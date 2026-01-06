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
//!
//! Future features:
//! - Remove Unused Declaration (diagnostic-based quick fix)
//! - Add Missing Property (diagnostic-based quick fix)
//! - Organize Imports (source action)

use crate::parser::NodeIndex;
use crate::parser::thin_node::ThinNodeArena;
use crate::parser::syntax_kind_ext;
use crate::thin_binder::ThinBinderState;
use crate::lsp::position::{Position, Range, LineMap};
use crate::lsp::rename::{WorkspaceEdit, TextEdit};
use crate::scanner::SyntaxKind;
use serde::Serialize;

// =============================================================================
// Code Action Types
// =============================================================================

/// Kind of code action (matches LSP spec).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CodeActionKind {
    /// Quick fix for an error or warning.
    QuickFix,
    /// Generic refactoring action.
    Refactor,
    /// Extract to variable/constant/function.
    RefactorExtract,
    /// Inline variable/function.
    RefactorInline,
    /// Organize imports.
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
    /// For now, this is empty since we don't integrate diagnostics yet.
    pub diagnostics: Vec<String>, // TODO: Use real Diagnostic type
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
        _context: CodeActionContext,
    ) -> Vec<CodeAction> {
        let mut actions = Vec::new();

        // Quick Fixes (diagnostic-based)
        // TODO: Implement when diagnostics are integrated
        // - Remove Unused Declaration (6133)
        // - Add Missing Property (2339)
        // - Add Missing Import (2304)

        // Refactorings (selection-based)
        // Only if the range is non-empty (user selected text)
        if range.start != range.end {
            if let Some(action) = self.extract_variable(root, range) {
                actions.push(action);
            }
        }

        actions
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
        let start_offset = self.line_map.position_to_offset(range.start);
        let end_offset = self.line_map.position_to_offset(range.end);

        // 2. Find the expression node that matches this range
        let expr_idx = self.find_expression_at_range(root, start_offset, end_offset)?;

        // 3. Verify it's an expression (not a statement or declaration)
        let expr_node = self.arena.get(expr_idx)?;
        if !self.is_extractable_expression(expr_node.kind) {
            return None;
        }

        // 4. Find the enclosing statement to determine where to insert the variable
        let stmt_idx = self.find_enclosing_statement(expr_idx)?;
        let stmt_node = self.arena.get(stmt_idx)?;

        // 5. Generate a unique variable name (simple version: use "extracted")
        let var_name = "extracted";

        // 6. Extract the selected text
        let selected_text = &self.source[start_offset as usize..end_offset as usize];

        // 7. Create text edits:
        //    a) Insert variable declaration before the statement
        //    b) Replace the selected expression with the variable name

        // Get the position to insert the variable declaration
        let insert_pos = self.line_map.offset_to_position(stmt_node.pos);

        // Calculate indentation by looking at the statement's line
        let indent = self.get_indentation_at_position(&insert_pos);

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
            range: range.clone(),
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
    fn find_expression_at_range(
        &self,
        root: NodeIndex,
        start: u32,
        end: u32,
    ) -> Option<NodeIndex> {
        let mut best_match: Option<(NodeIndex, u32)> = None;

        self.traverse(root, &mut |idx| {
            let node = self.arena.get(idx)?;
            let node_start = node.pos;
            let node_end = node.end;

            // Check if this node's range matches the selection
            if node_start == start && node_end == end {
                if self.is_expression(node.kind) {
                    // Found an exact match
                    let size = node_end - node_start;
                    match best_match {
                        None => best_match = Some((idx, size)),
                        Some((_, best_size)) => {
                            // Prefer smallest exact match (most specific)
                            if size < best_size {
                                best_match = Some((idx, size));
                            }
                        }
                    }
                }
            }

            Some(())
        });

        best_match.map(|(idx, _)| idx)
    }

    /// Find the enclosing statement for a given node.
    fn find_enclosing_statement(&self, node_idx: NodeIndex) -> Option<NodeIndex> {
        // Walk up the tree until we find a statement
        // Note: ThinNodeArena doesn't have parent pointers, so we need to traverse from root
        // For simplicity, we'll use a helper that tracks parents during traversal

        // Simple approach: find the statement that contains this node
        // by checking if the node is within a statement's range
        let target_node = self.arena.get(node_idx)?;
        let target_start = target_node.pos;

        // Start from source file and find the first statement that contains the target
        // This is a simplified approach - in a real implementation, we'd build a parent map
        let mut enclosing_stmt = None;

        // For now, we'll just traverse the entire tree and find the smallest statement
        // that contains our target node
        self.traverse(NodeIndex(0), &mut |idx| {
            let node = self.arena.get(idx)?;

            if self.is_statement(node.kind) && node.pos <= target_start && node.end >= target_node.end {
                match enclosing_stmt {
                    None => enclosing_stmt = Some((idx, node.end - node.pos)),
                    Some((_, size)) => {
                        let new_size = node.end - node.pos;
                        if new_size < size {
                            enclosing_stmt = Some((idx, new_size));
                        }
                    }
                }
            }

            Some(())
        });

        enclosing_stmt.map(|(idx, _)| idx)
    }

    /// Traverse the AST, calling visitor for each node.
    fn traverse<F>(&self, node_idx: NodeIndex, visitor: &mut F)
    where
        F: FnMut(NodeIndex) -> Option<()>,
    {
        visitor(node_idx);

        if let Some(node) = self.arena.get(node_idx) {
            // Simple child traversal for common node types
            match node.kind {
                k if k == syntax_kind_ext::SOURCE_FILE => {
                    if let Some(sf) = self.arena.get_source_file(node) {
                        for &stmt in &sf.statements.nodes {
                            self.traverse(stmt, visitor);
                        }
                    }
                }
                k if k == syntax_kind_ext::BLOCK => {
                    if let Some(block) = self.arena.get_block(node) {
                        for &stmt in &block.statements.nodes {
                            self.traverse(stmt, visitor);
                        }
                    }
                }
                k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                    if let Some(var) = self.arena.get_variable(node) {
                        for &decl_list in &var.declarations.nodes {
                            self.traverse(decl_list, visitor);
                        }
                    }
                }
                k if k == syntax_kind_ext::VARIABLE_DECLARATION_LIST => {
                    if let Some(list) = self.arena.get_variable(node) {
                        for &decl in &list.declarations.nodes {
                            self.traverse(decl, visitor);
                        }
                    }
                }
                k if k == syntax_kind_ext::VARIABLE_DECLARATION => {
                    if let Some(decl) = self.arena.get_variable_declaration(node) {
                        self.traverse(decl.name, visitor);
                        if !decl.initializer.is_none() {
                            self.traverse(decl.initializer, visitor);
                        }
                    }
                }
                k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                    if let Some(expr_stmt) = self.arena.get_expression_statement(node) {
                        self.traverse(expr_stmt.expression, visitor);
                    }
                }
                k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                    if let Some(binary) = self.arena.get_binary_expr(node) {
                        self.traverse(binary.left, visitor);
                        self.traverse(binary.right, visitor);
                    }
                }
                k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION => {
                    if let Some(access) = self.arena.get_access_expr(node) {
                        self.traverse(access.expression, visitor);
                        self.traverse(access.name_or_argument, visitor);
                    }
                }
                k if k == syntax_kind_ext::CALL_EXPRESSION => {
                    if let Some(call) = self.arena.get_call_expr(node) {
                        self.traverse(call.expression, visitor);
                        if let Some(ref args) = call.arguments {
                            for &arg in &args.nodes {
                                self.traverse(arg, visitor);
                            }
                        }
                    }
                }
                _ => {
                    // For other nodes, no children to traverse
                }
            }
        }
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
            || kind == SyntaxKind::StringLiteral as u16
            || kind == SyntaxKind::NumericLiteral as u16
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

    /// Get the indentation (leading whitespace) at a given position.
    fn get_indentation_at_position(&self, pos: &Position) -> String {
        // Find the start of the line
        let line_start = self.source
            .lines()
            .nth(pos.line as usize)
            .unwrap_or("");

        // Count leading whitespace
        let mut indent = String::new();
        for ch in line_start.chars() {
            if ch == ' ' || ch == '\t' {
                indent.push(ch);
            } else {
                break;
            }
        }

        indent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;

    #[test]
    fn test_extract_variable_property_access() {
        let source = "const x = foo.bar.baz + 1;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(source);
        let provider = CodeActionProvider::new(
            arena,
            &binder,
            &line_map,
            "test.ts".to_string(),
            source,
        );

        // Select "foo.bar.baz" (positions 10 to 21)
        let range = Range {
            start: Position::new(0, 10),
            end: Position::new(0, 21),
        };

        let actions = provider.provide_code_actions(root, range, CodeActionContext {
            diagnostics: Vec::new(),
            only: None,
        });

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].title, "Extract to constant 'extracted'");
        assert_eq!(actions[0].kind, CodeActionKind::RefactorExtract);
        assert!(actions[0].is_preferred);

        // Check the edit
        let edit = actions[0].edit.as_ref().unwrap();
        let edits = edit.changes.get("test.ts").unwrap();
        assert_eq!(edits.len(), 2);

        // First edit should insert the declaration
        assert!(edits[0].new_text.contains("const extracted = foo.bar.baz;"));

        // Second edit should replace the expression
        assert_eq!(edits[1].new_text, "extracted");
    }

    #[test]
    fn test_extract_variable_no_action_for_simple_literal() {
        let source = "const x = 42;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(source);
        let provider = CodeActionProvider::new(
            arena,
            &binder,
            &line_map,
            "test.ts".to_string(),
            source,
        );

        // Select "42"
        let range = Range {
            start: Position::new(0, 10),
            end: Position::new(0, 12),
        };

        let actions = provider.provide_code_actions(root, range, CodeActionContext {
            diagnostics: Vec::new(),
            only: None,
        });

        // Should not extract simple literals
        assert_eq!(actions.len(), 0);
    }

    #[test]
    fn test_extract_variable_empty_range() {
        let source = "const x = foo.bar.baz;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(source);
        let provider = CodeActionProvider::new(
            arena,
            &binder,
            &line_map,
            "test.ts".to_string(),
            source,
        );

        // Empty range (no selection)
        let range = Range {
            start: Position::new(0, 10),
            end: Position::new(0, 10),
        };

        let actions = provider.provide_code_actions(root, range, CodeActionContext {
            diagnostics: Vec::new(),
            only: None,
        });

        // Should not provide refactorings for empty ranges
        assert_eq!(actions.len(), 0);
    }
}
