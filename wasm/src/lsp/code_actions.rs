//! Code Actions for the LSP.
//!
//! Provides quick fixes and refactorings to improve code quality and fix errors.
//!
//! Architecture:
//! - The Checker identifies problems (Diagnostics)
//! - The CodeActionProvider identifies solutions (TextEdits)
//!
//! Current features:
//! - Extract Variable (selection-based refactoring)
//! - Organize Imports (sort-only)
//! - Remove Unused Import (diagnostic-based quick fix)
//!
//! Future features:
//! - Remove Unused Declarations (diagnostic-based quick fix)
//! - Add Missing Property (diagnostic-based quick fix)
//! - Add Missing Import (diagnostic-based quick fix)

use crate::parser::NodeIndex;
use crate::parser::thin_node::{NodeAccess, ThinNodeArena};
use crate::parser::syntax_kind_ext;
use crate::comments::get_leading_comments_from_cache;
use crate::thin_binder::ThinBinderState;
use crate::lsp::position::{Position, Range, LineMap};
use crate::lsp::diagnostics::LspDiagnostic;
use crate::lsp::rename::{WorkspaceEdit, TextEdit};
use crate::lsp::utils::find_node_at_offset;
use crate::scanner::SyntaxKind;
use serde::Serialize;

// =============================================================================
// Code Action Types
// =============================================================================

/// Kind of code action (matches LSP spec).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum CodeActionKind {
    /// Quick fix for an error or warning.
    #[serde(rename = "quickfix")]
    QuickFix,
    /// Generic refactoring action.
    #[serde(rename = "refactor")]
    Refactor,
    /// Extract to variable/constant/function.
    #[serde(rename = "refactor.extract")]
    RefactorExtract,
    /// Inline variable/function.
    #[serde(rename = "refactor.inline")]
    RefactorInline,
    /// Organize imports.
    #[serde(rename = "source.organizeImports")]
    SourceOrganizeImports,
}

/// A code action represents a change that can be performed in code.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeAction {
    /// A short, human-readable title for this code action.
    pub title: String,
    /// The kind of the code action.
    pub kind: CodeActionKind,
    /// The workspace edit to apply.
    pub edit: Option<WorkspaceEdit>,
    /// Marks this as a preferred action (shown first in UI).
    pub is_preferred: bool,
}

/// Context passed when requesting code actions.
#[derive(Debug, Clone)]
pub struct CodeActionContext {
    /// Diagnostics at the requested position (for quick fixes).
    pub diagnostics: Vec<LspDiagnostic>,
    /// Only return actions of these kinds (client filter).
    pub only: Option<Vec<CodeActionKind>>,
}

// =============================================================================
// Code Action Provider
// =============================================================================

/// Provides code actions for a given position/range in the source code.
pub struct CodeActionProvider<'a> {
    arena: &'a ThinNodeArena,
    binder: &'a ThinBinderState,
    line_map: &'a LineMap,
    file_name: String,
    source: &'a str,
}

impl<'a> CodeActionProvider<'a> {
    /// Create a new code action provider.
    pub fn new(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        line_map: &'a LineMap,
        file_name: String,
        source: &'a str,
    ) -> Self {
        Self {
            arena,
            binder,
            line_map,
            file_name,
            source,
        }
    }

    /// Provide code actions for a range in the source code.
    pub fn provide_code_actions(
        &self,
        root: NodeIndex,
        range: Range,
        context: CodeActionContext,
    ) -> Vec<CodeAction> {
        let mut actions = Vec::new();

        // Quick Fixes (diagnostic-based)
        let request_quickfix = context
            .only
            .as_ref()
            .map_or(true, |kinds| kinds.contains(&CodeActionKind::QuickFix));
        if request_quickfix {
            for diag in &context.diagnostics {
                if let Some(action) = self.unused_import_quickfix(diag) {
                    actions.push(action);
                }
            }
        }
        // TODO: Add Missing Property (2339)
        // TODO: Add Missing Import (2304)

        // Source Actions (file-level)
        let request_organize = context.only
            .as_ref()
            .map_or(true, |kinds| kinds.contains(&CodeActionKind::SourceOrganizeImports));
        if request_organize {
            if let Some(action) = self.organize_imports(root) {
                actions.push(action);
            }
        }

        // Refactorings (selection-based)
        // Only if the range is non-empty (user selected text)
        if range.start != range.end {
            if let Some(action) = self.extract_variable(root, range) {
                actions.push(action);
            }
        }

        actions
    }

    fn unused_import_quickfix(&self, diag: &LspDiagnostic) -> Option<CodeAction> {
        let code = diag.code?;
        if code != crate::checker::types::diagnostics::diagnostic_codes::UNUSED_IMPORT {
            return None;
        }

        let start_offset = self.line_map.position_to_offset(diag.range.start, self.source)?;
        let node_idx = find_node_at_offset(self.arena, start_offset);
        if node_idx.is_none() {
            return None;
        }

        let (import_decl, removal) = self.import_removal_target(node_idx)?;
        let (edit, title) = self.build_import_removal_edit(import_decl, removal)?;

        let mut changes = std::collections::HashMap::new();
        changes.insert(self.file_name.clone(), vec![edit]);

        Some(CodeAction {
            title,
            kind: CodeActionKind::QuickFix,
            edit: Some(WorkspaceEdit { changes }),
            is_preferred: true,
        })
    }

    /// Organize imports: sort contiguous import blocks by module specifier.
    fn organize_imports(&self, root: NodeIndex) -> Option<CodeAction> {
        let root_node = self.arena.get(root)?;
        let source_file = self.arena.get_source_file(root_node)?;

        let mut edits = Vec::new();
        let statements = &source_file.statements.nodes;
        let mut i = 0;

        while i < statements.len() {
            let start_idx = i;
            while i < statements.len() && self.is_import_declaration(statements[i]) {
                i += 1;
            }
            let end_idx = i;

            if end_idx > start_idx + 1 {
                if let Some(edit) = self.sort_imports_range(&statements[start_idx..end_idx], &source_file.comments) {
                    edits.push(edit);
                }
            }

            while i < statements.len() && !self.is_import_declaration(statements[i]) {
                i += 1;
            }
        }

        if edits.is_empty() {
            return None;
        }

        let mut changes = std::collections::HashMap::new();
        changes.insert(self.file_name.clone(), edits);

        Some(CodeAction {
            title: "Organize Imports".to_string(),
            kind: CodeActionKind::SourceOrganizeImports,
            edit: Some(WorkspaceEdit { changes }),
            is_preferred: false,
        })
    }

    fn is_import_declaration(&self, node_idx: NodeIndex) -> bool {
        self.arena
            .get(node_idx)
            .map_or(false, |node| node.kind == syntax_kind_ext::IMPORT_DECLARATION)
    }

    fn sort_imports_range(
        &self,
        import_nodes: &[NodeIndex],
        comments: &[crate::comments::CommentRange],
    ) -> Option<TextEdit> {
        #[derive(Clone)]
        struct ImportInfo {
            start: u32,
            end: u32,
            text: String,
            module_specifier: String,
            is_side_effect: bool,
        }

        let mut imports = Vec::new();
        let mut block_start = u32::MAX;
        let mut block_end = 0u32;

        for &node_idx in import_nodes {
            let node = self.arena.get(node_idx)?;
            let leading = get_leading_comments_from_cache(comments, node.pos, self.source);
            let start = leading.first().map(|c| c.pos).unwrap_or(node.pos);

            block_start = block_start.min(start);
            block_end = block_end.max(node.end);

            let import_decl = self.arena.get_import_decl(node)?;
            let is_side_effect = import_decl.import_clause.is_none();
            let specifier = self.get_module_specifier(node_idx).unwrap_or_default();
            let text = self.source.get(start as usize..node.end as usize)?.to_string();
            imports.push(ImportInfo {
                start,
                end: node.end,
                text,
                module_specifier: specifier,
                is_side_effect,
            });
        }

        if imports.is_empty() {
            return None;
        }

        let mut groups: Vec<Vec<ImportInfo>> = Vec::new();
        let mut separators: Vec<String> = Vec::new();
        let mut current = Vec::new();

        for idx in 0..imports.len() {
            let mut info = imports[idx].clone();
            if idx + 1 < imports.len() {
                let next_start = imports[idx + 1].start;
                let between = self.source.get(info.end as usize..next_start as usize).unwrap_or("");
                let has_blank_line = between.contains("\n\n")
                    || between.contains("\r\n\r\n")
                    || between.contains("\r\r");
                if has_blank_line {
                    current.push(info);
                    groups.push(std::mem::take(&mut current));
                    separators.push(between.to_string());
                    continue;
                }
                info.text.push_str(between);
                info.end = next_start;
            }
            current.push(info);
        }
        if !current.is_empty() {
            groups.push(current);
        }

        let mut new_text = String::new();
        for (group_idx, group) in groups.into_iter().enumerate() {
            let mut new_chunks = Vec::new();
            let mut pending = Vec::new();
            for info in group {
                if info.is_side_effect {
                    pending.sort_by(|a: &ImportInfo, b: &ImportInfo| a.module_specifier.cmp(&b.module_specifier));
                    for sorted in pending.drain(..) {
                        new_chunks.push(sorted.text);
                    }
                    new_chunks.push(info.text);
                } else {
                    pending.push(info);
                }
            }
            if !pending.is_empty() {
                pending.sort_by(|a, b| a.module_specifier.cmp(&b.module_specifier));
                for sorted in pending {
                    new_chunks.push(sorted.text);
                }
            }

            if group_idx > 0 {
                if let Some(sep) = separators.get(group_idx - 1) {
                    new_text.push_str(sep);
                } else {
                    new_text.push('\n');
                }
            }

            if !new_chunks.is_empty() {
                if !new_text.is_empty() && !new_text.ends_with('\n') {
                    new_text.push('\n');
                }
                for chunk in new_chunks {
                    new_text.push_str(&chunk);
                    if !chunk.ends_with('\n') && !chunk.ends_with('\r') {
                        new_text.push('\n');
                    }
                }
            }
        }

        let original = self.source.get(block_start as usize..block_end as usize)?;
        if original == new_text {
            return None;
        }

        let start_pos = self.line_map.offset_to_position(block_start, self.source);
        let end_pos = self.line_map.offset_to_position(block_end, self.source);

        Some(TextEdit {
            range: Range::new(start_pos, end_pos),
            new_text,
        })
    }

    fn get_module_specifier(&self, import_idx: NodeIndex) -> Option<String> {
        let node = self.arena.get(import_idx)?;
        let import_decl = self.arena.get_import_decl(node)?;
        let spec_idx = import_decl.module_specifier;
        let text = self.arena.get_literal_text(spec_idx)?;
        Some(text.to_string())
    }

    fn import_removal_target(&self, node_idx: NodeIndex) -> Option<(NodeIndex, ImportRemoval)> {
        let mut current = node_idx;
        while !current.is_none() {
            let node = self.arena.get(current)?;
            if node.kind == syntax_kind_ext::IMPORT_SPECIFIER {
                let name = self.specifier_local_name(current)?;
                let import_decl = self.find_import_decl(current)?;
                return Some((import_decl, ImportRemoval::Named { specifier: current, name }));
            }

            if node.kind == SyntaxKind::Identifier as u16 {
                let parent = self.arena.get_extended(current)?.parent;
                if parent.is_none() {
                    return None;
                }

                let parent_node = self.arena.get(parent)?;
                if parent_node.kind == syntax_kind_ext::IMPORT_CLAUSE {
                    let clause = self.arena.get_import_clause(parent_node)?;
                    if clause.name == current {
                        let name = self.arena.get_identifier_text(current)?.to_string();
                        let import_decl = self.find_import_decl(parent)?;
                        return Some((import_decl, ImportRemoval::Default { name }));
                    }
                    if clause.named_bindings == current {
                        let name = self.arena.get_identifier_text(current)?.to_string();
                        let import_decl = self.find_import_decl(parent)?;
                        return Some((import_decl, ImportRemoval::Namespace { name }));
                    }
                }
            }

            current = self.arena.get_extended(current)?.parent;
        }

        None
    }

    fn find_import_decl(&self, start: NodeIndex) -> Option<NodeIndex> {
        let mut current = start;
        while !current.is_none() {
            let node = self.arena.get(current)?;
            if node.kind == syntax_kind_ext::IMPORT_DECLARATION {
                return Some(current);
            }
            current = self.arena.get_extended(current)?.parent;
        }
        None
    }

    fn specifier_local_name(&self, spec_idx: NodeIndex) -> Option<String> {
        let spec_node = self.arena.get(spec_idx)?;
        let spec = self.arena.get_specifier(spec_node)?;
        let local_ident = if !spec.name.is_none() {
            spec.name
        } else {
            spec.property_name
        };
        self.arena.get_identifier_text(local_ident).map(|name| name.to_string())
    }

    fn build_import_removal_edit(
        &self,
        import_decl: NodeIndex,
        removal: ImportRemoval,
    ) -> Option<(TextEdit, String)> {
        let import_node = self.arena.get(import_decl)?;
        let import_data = self.arena.get_import_decl(import_node)?;
        if import_data.import_clause.is_none() {
            return None;
        }

        let clause_node = self.arena.get(import_data.import_clause)?;
        let clause = self.arena.get_import_clause(clause_node)?;

        let mut default_name = if !clause.name.is_none() {
            self.arena.get_identifier_text(clause.name).map(|name| name.to_string())
        } else {
            None
        };

        let mut namespace_name = None;
        let mut named_specs = Vec::new();

        if !clause.named_bindings.is_none() {
            let bindings_node = self.arena.get(clause.named_bindings)?;
            if bindings_node.kind == SyntaxKind::Identifier as u16 {
                namespace_name = self
                    .arena
                    .get_identifier_text(clause.named_bindings)
                    .map(|name| name.to_string());
            } else if let Some(named) = self.arena.get_named_imports(bindings_node) {
                for &spec_idx in &named.elements.nodes {
                    let spec_node = self.arena.get(spec_idx)?;
                    let spec = self.arena.get_specifier(spec_node)?;
                    let import_ident = if !spec.property_name.is_none() {
                        spec.property_name
                    } else {
                        spec.name
                    };
                    let local_ident = if !spec.name.is_none() {
                        spec.name
                    } else {
                        spec.property_name
                    };
                    let import_name = self.arena.get_identifier_text(import_ident)?.to_string();
                    let local_name = self.arena.get_identifier_text(local_ident)?.to_string();
                    named_specs.push(NamedImportSpec {
                        specifier: spec_idx,
                        import_name,
                        local_name,
                        is_type_only: spec.is_type_only,
                    });
                }
            }
        }

        let removed_name = removal.name().to_string();
        match removal {
            ImportRemoval::Default { .. } => default_name = None,
            ImportRemoval::Namespace { .. } => namespace_name = None,
            ImportRemoval::Named { specifier, .. } => {
                named_specs.retain(|spec| spec.specifier != specifier);
            }
        }

        let has_named = !named_specs.is_empty();
        let has_namespace = namespace_name.is_some();
        let has_default = default_name.is_some();

        let (range, trailing) = self.import_decl_range(import_node);
        if !has_named && !has_namespace && !has_default {
            let edit = TextEdit {
                range,
                new_text: String::new(),
            };
            let title = format!("Remove unused import '{}'", removed_name);
            return Some((edit, title));
        }

        let mut parts = Vec::new();
        if let Some(default_name) = default_name {
            parts.push(default_name);
        }
        if let Some(namespace_name) = namespace_name {
            parts.push(format!("* as {}", namespace_name));
        }
        if has_named {
            let mut items = Vec::new();
            for spec in named_specs {
                let mut item = String::new();
                if spec.is_type_only && !clause.is_type_only {
                    item.push_str("type ");
                }
                if spec.import_name == spec.local_name {
                    item.push_str(&spec.import_name);
                } else {
                    item.push_str(&format!("{} as {}", spec.import_name, spec.local_name));
                }
                items.push(item);
            }
            parts.push(format!("{{ {} }}", items.join(", ")));
        }

        let module_node = self.arena.get(import_data.module_specifier)?;
        let module_text = self
            .source
            .get(module_node.pos as usize..module_node.end as usize)?
            .to_string();

        let mut new_text = String::new();
        new_text.push_str("import ");
        if clause.is_type_only {
            new_text.push_str("type ");
        }
        new_text.push_str(&parts.join(", "));
        new_text.push_str(" from ");
        new_text.push_str(&module_text);
        new_text.push(';');
        new_text.push_str(&trailing);

        let edit = TextEdit { range, new_text };
        let title = format!("Remove unused import '{}'", removed_name);

        Some((edit, title))
    }

    fn import_decl_range(&self, node: &crate::parser::thin_node::ThinNode) -> (Range, String) {
        let mut end = node.end;
        let mut trailing = String::new();
        if let Some(rest) = self.source.get(end as usize..) {
            if rest.starts_with("\r\n") {
                end += 2;
                trailing = "\r\n".to_string();
            } else if rest.starts_with('\n') {
                end += 1;
                trailing = "\n".to_string();
            } else if rest.starts_with('\r') {
                end += 1;
                trailing = "\r".to_string();
            }
        }

        let start_pos = self.line_map.offset_to_position(node.pos, self.source);
        let end_pos = self.line_map.offset_to_position(end, self.source);
        (Range::new(start_pos, end_pos), trailing)
    }

    /// Extract the selected expression to a new variable.
    ///
    /// Example: Selecting `foo.bar.baz` produces:
    /// ```typescript
    /// const extracted = foo.bar.baz;
    /// // ... use extracted here
    /// ```
    fn extract_variable(&self, root: NodeIndex, range: Range) -> Option<CodeAction> {
        // 1. Convert range to offsets
        let start_offset = self.line_map.position_to_offset(range.start, self.source)?;
        let end_offset = self.line_map.position_to_offset(range.end, self.source)?;

        // 2. Find the expression node that matches this range
        let expr_idx = self.find_expression_at_range(root, start_offset, end_offset)?;

        // 3. Verify it's an expression (not a statement or declaration)
        let expr_node = self.arena.get(expr_idx)?;
        if !self.is_extractable_expression(expr_node.kind) {
            return None;
        }

        // 4. Find the enclosing statement to determine where to insert the variable
        let stmt_idx = self.find_enclosing_statement(root, expr_idx)?;
        let stmt_node = self.arena.get(stmt_idx)?;
        if !self.statement_allows_lexical_insertion(stmt_idx) {
            return None;
        }
        // TODO: Validate that extracted expressions don't capture out-of-scope identifiers.

        // TODO: Validate name collisions in scope before inserting.
        // 5. Generate a unique variable name (simple version: use "extracted")
        let var_name = "extracted";

        // TODO: Preserve operator precedence (wrap in parentheses when needed).
        // 6. Extract the selected text (snap to node boundaries)
        let node_start = expr_node.pos;
        let node_end = expr_node.end;
        let selected_text = self.source.get(node_start as usize..node_end as usize)?;
        let replacement_range = Range::new(
            self.line_map.offset_to_position(node_start, self.source),
            self.line_map.offset_to_position(node_end, self.source),
        );

        // 7. Create text edits:
        //    a) Insert variable declaration before the statement
        //    b) Replace the selected expression with the variable name

        // Get the position to insert the variable declaration
        let stmt_pos = self.line_map.offset_to_position(stmt_node.pos, self.source);
        let insert_pos = Position::new(stmt_pos.line, 0);

        // Calculate indentation by looking at the statement's line
        let indent = self.get_indentation_at_position(&stmt_pos);

        let declaration = format!("{}const {} = {};\n", indent, var_name, selected_text);

        let mut edits = Vec::new();

        // Insert the declaration
        edits.push(TextEdit {
            range: Range {
                start: insert_pos,
                end: insert_pos,
            },
            new_text: declaration,
        });

        // Replace the expression with the variable name
        edits.push(TextEdit {
            range: replacement_range,
            new_text: var_name.to_string(),
        });

        // Create the workspace edit
        let mut changes = std::collections::HashMap::new();
        changes.insert(self.file_name.clone(), edits);

        Some(CodeAction {
            title: format!("Extract to constant '{}'", var_name),
            kind: CodeActionKind::RefactorExtract,
            edit: Some(WorkspaceEdit { changes }),
            is_preferred: true,
        })
    }

    /// Find an expression node that matches the given range.
    /// Finds the smallest expression node that contains the selection.
    fn find_expression_at_range(
        &self,
        _root: NodeIndex,
        start: u32,
        end: u32,
    ) -> Option<NodeIndex> {
        let mut current = find_node_at_offset(self.arena, start);
        while !current.is_none() {
            let node = self.arena.get(current)?;
            if node.pos <= start && node.end >= end && self.is_expression(node.kind) {
                return Some(current);
            }
            let ext = self.arena.get_extended(current)?;
            current = ext.parent;
        }

        None
    }

    /// Find the enclosing statement for a given node.
    fn find_enclosing_statement(&self, _root: NodeIndex, node_idx: NodeIndex) -> Option<NodeIndex> {
        let mut current = node_idx;
        while !current.is_none() {
            let node = self.arena.get(current)?;
            if self.is_statement(node.kind) {
                return Some(current);
            }
            let ext = self.arena.get_extended(current)?;
            current = ext.parent;
        }
        None
    }

    /// Check if a syntax kind is an expression.
    fn is_expression(&self, kind: u16) -> bool {
        // Check both token kinds (from scanner) and expression kinds (from parser)
        kind == SyntaxKind::Identifier as u16
            || kind == SyntaxKind::StringLiteral as u16
            || kind == SyntaxKind::NumericLiteral as u16
            || kind == SyntaxKind::TrueKeyword as u16
            || kind == SyntaxKind::FalseKeyword as u16
            || kind == SyntaxKind::NullKeyword as u16
            || kind == SyntaxKind::ThisKeyword as u16
            || kind == SyntaxKind::SuperKeyword as u16
            || kind == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION
            || kind == syntax_kind_ext::CALL_EXPRESSION
            || kind == syntax_kind_ext::BINARY_EXPRESSION
            || kind == syntax_kind_ext::ARRAY_LITERAL_EXPRESSION
            || kind == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION
            || kind == syntax_kind_ext::FUNCTION_EXPRESSION
            || kind == syntax_kind_ext::ARROW_FUNCTION
            || kind == syntax_kind_ext::CLASS_EXPRESSION
            || kind == syntax_kind_ext::NEW_EXPRESSION
            || kind == syntax_kind_ext::TEMPLATE_EXPRESSION
            || kind == syntax_kind_ext::TAGGED_TEMPLATE_EXPRESSION
            || kind == syntax_kind_ext::AWAIT_EXPRESSION
            || kind == syntax_kind_ext::YIELD_EXPRESSION
            || kind == syntax_kind_ext::CONDITIONAL_EXPRESSION
            || kind == syntax_kind_ext::PARENTHESIZED_EXPRESSION
            || kind == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION
            || kind == syntax_kind_ext::PREFIX_UNARY_EXPRESSION
            || kind == syntax_kind_ext::POSTFIX_UNARY_EXPRESSION
    }

    /// Check if an expression is extractable (not all expressions should be extracted).
    fn is_extractable_expression(&self, kind: u16) -> bool {
        // Don't extract simple literals or identifiers - not useful
        !(kind == SyntaxKind::Identifier as u16
            || kind == SyntaxKind::TrueKeyword as u16
            || kind == SyntaxKind::FalseKeyword as u16
            || kind == SyntaxKind::NullKeyword as u16)
    }

    /// Check if a syntax kind is a statement.
    fn is_statement(&self, kind: u16) -> bool {
        matches!(
            kind,
            syntax_kind_ext::VARIABLE_STATEMENT
                | syntax_kind_ext::EXPRESSION_STATEMENT
                | syntax_kind_ext::IF_STATEMENT
                | syntax_kind_ext::FOR_STATEMENT
                | syntax_kind_ext::FOR_IN_STATEMENT
                | syntax_kind_ext::FOR_OF_STATEMENT
                | syntax_kind_ext::WHILE_STATEMENT
                | syntax_kind_ext::DO_STATEMENT
                | syntax_kind_ext::RETURN_STATEMENT
                | syntax_kind_ext::BREAK_STATEMENT
                | syntax_kind_ext::CONTINUE_STATEMENT
                | syntax_kind_ext::THROW_STATEMENT
                | syntax_kind_ext::TRY_STATEMENT
                | syntax_kind_ext::SWITCH_STATEMENT
                | syntax_kind_ext::BLOCK
        )
    }

    fn statement_allows_lexical_insertion(&self, stmt_idx: NodeIndex) -> bool {
        let parent = match self.arena.get_extended(stmt_idx) {
            Some(ext) => ext.parent,
            None => return false,
        };
        if parent.is_none() {
            return true;
        }

        let parent_node = match self.arena.get(parent) {
            Some(node) => node,
            None => return false,
        };

        matches!(
            parent_node.kind,
            k if k == syntax_kind_ext::SOURCE_FILE
                || k == syntax_kind_ext::BLOCK
                || k == syntax_kind_ext::MODULE_BLOCK
                || k == syntax_kind_ext::CASE_CLAUSE
                || k == syntax_kind_ext::DEFAULT_CLAUSE
        )
    }

    /// Get the indentation (leading whitespace) at a given position.
    fn get_indentation_at_position(&self, pos: &Position) -> String {
        let line_start = self.line_map.line_start(pos.line as usize).unwrap_or(0);
        let slice = self.source.get(line_start as usize..).unwrap_or("");
        slice
            .chars()
            .take_while(|ch| *ch == ' ' || *ch == '\t')
            .collect()
    }
}

#[derive(Clone, Debug)]
struct NamedImportSpec {
    specifier: NodeIndex,
    import_name: String,
    local_name: String,
    is_type_only: bool,
}

#[derive(Clone, Debug)]
enum ImportRemoval {
    Default { name: String },
    Namespace { name: String },
    Named { specifier: NodeIndex, name: String },
}

impl ImportRemoval {
    fn name(&self) -> &str {
        match self {
            ImportRemoval::Default { name }
            | ImportRemoval::Namespace { name }
            | ImportRemoval::Named { name, .. } => name,
        }
    }
}
