//! Project container for multi-file LSP operations.
//!
//! This provides a lightweight home for parsed files, binders, and line maps so
//! LSP features can be extended across multiple files.

use std::cmp::Ordering;
use std::path::{Component, Path, PathBuf};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::binder::SymbolId;
use crate::checker::TypeCache;
use crate::lsp::code_actions::{
    CodeAction, CodeActionContext, CodeActionKind, CodeActionProvider, ImportCandidate,
    ImportCandidateKind,
};
use crate::lsp::completions::{CompletionItem, Completions};
use crate::lsp::diagnostics::LspDiagnostic;
use crate::lsp::hover::{HoverInfo, HoverProvider};
use crate::lsp::signature_help::{SignatureHelp, SignatureHelpProvider};
use crate::lsp::utils::find_node_at_offset;
use crate::parser::thin_node::NodeAccess;
use crate::parser::{NodeIndex, syntax_kind_ext, thin_node::ThinNodeArena};
use crate::scanner::SyntaxKind;
use crate::solver::TypeInterner;
use crate::thin_binder::ThinBinderState;
use crate::thin_parser::ThinParserState;
use crate::lsp::definition::GoToDefinition;
use crate::lsp::references::FindReferences;
use crate::lsp::rename::TextEdit;
use crate::lsp::position::{LineMap, Position, Location, Range};

enum ImportKind {
    Named(String),
    Default,
    Namespace,
}

struct ImportTarget {
    module_specifier: String,
    kind: ImportKind,
}

struct NamespaceReexportTarget {
    file: String,
    namespace: String,
    member: String,
}

struct ExportMatch {
    kind: ImportCandidateKind,
    is_type_only: bool,
}

/// Parsed file state used by LSP features.
pub struct ProjectFile {
    file_name: String,
    root: NodeIndex,
    parser: ThinParserState,
    binder: ThinBinderState,
    line_map: LineMap,
    type_interner: TypeInterner,
    type_cache: Option<TypeCache>,
}

impl ProjectFile {
    /// Parse and bind a single source file for LSP queries.
    pub fn new(file_name: String, source_text: String) -> Self {
        let mut parser = ThinParserState::new(file_name.clone(), source_text);
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        let line_map = LineMap::build(parser.get_source_text());

        Self {
            file_name,
            root,
            parser,
            binder,
            line_map,
            type_interner: TypeInterner::new(),
            type_cache: None,
        }
    }

    /// File name used for LSP locations.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Root node of the parsed source file.
    pub fn root(&self) -> NodeIndex {
        self.root
    }

    /// Arena containing parsed ThinNodes.
    pub fn arena(&self) -> &ThinNodeArena {
        self.parser.get_arena()
    }

    /// Binder state for symbol lookup.
    pub fn binder(&self) -> &ThinBinderState {
        &self.binder
    }

    /// Line map for offset <-> position conversions.
    pub fn line_map(&self) -> &LineMap {
        &self.line_map
    }

    /// Original source text for this file.
    pub fn source_text(&self) -> &str {
        self.parser.get_source_text()
    }

    pub fn get_hover(&mut self, position: Position) -> Option<HoverInfo> {
        let provider = HoverProvider::new(
            self.parser.get_arena(),
            &self.binder,
            &self.line_map,
            &self.type_interner,
            self.parser.get_source_text(),
            self.file_name.clone(),
        );

        provider.get_hover(self.root, position, &mut self.type_cache)
    }

    pub fn get_signature_help(&mut self, position: Position) -> Option<SignatureHelp> {
        let provider = SignatureHelpProvider::new(
            self.parser.get_arena(),
            &self.binder,
            &self.line_map,
            &self.type_interner,
            self.parser.get_source_text(),
            self.file_name.clone(),
        );

        provider.get_signature_help(self.root, position, &mut self.type_cache)
    }

    pub fn get_completions(&mut self, position: Position) -> Option<Vec<CompletionItem>> {
        let provider = Completions::new_with_types(
            self.parser.get_arena(),
            &self.binder,
            &self.line_map,
            &self.type_interner,
            self.parser.get_source_text(),
            self.file_name.clone(),
        );

        provider.get_completions_with_cache(self.root, position, &mut self.type_cache)
    }

    fn node_location(&self, node_idx: NodeIndex) -> Option<Location> {
        let node = self.arena().get(node_idx)?;
        let start = self.line_map.offset_to_position(node.pos, self.source_text());
        let end = self.line_map.offset_to_position(node.end, self.source_text());
        Some(Location {
            file_path: self.file_name.clone(),
            range: Range::new(start, end),
        })
    }

    fn resolve_symbol(&self, node_idx: NodeIndex) -> Option<SymbolId> {
        if node_idx.is_none() {
            return None;
        }

        if let Some(&sym_id) = self.binder.node_symbols.get(&node_idx.0) {
            return Some(sym_id);
        }

        self.binder.resolve_identifier(self.arena(), node_idx)
    }

    fn export_locations(&self, export_name: &str) -> Vec<Location> {
        self.export_nodes(export_name)
            .into_iter()
            .filter_map(|node| self.node_location(node))
            .collect()
    }

    fn export_nodes(&self, export_name: &str) -> Vec<NodeIndex> {
        let arena = self.arena();
        let binder = self.binder();
        let mut nodes = Vec::new();

        let Some(root_node) = arena.get(self.root()) else { return Vec::new(); };
        let Some(source_file) = arena.get_source_file(root_node) else { return Vec::new(); };

        for &stmt_idx in &source_file.statements.nodes {
            let Some(stmt_node) = arena.get(stmt_idx) else { continue; };
            if stmt_node.kind != syntax_kind_ext::EXPORT_DECLARATION {
                continue;
            }
            let Some(export) = arena.get_export_decl(stmt_node) else { continue; };
            if !export.module_specifier.is_none() {
                continue;
            }

            if export.is_default_export {
                if export_name == "default" {
                    self.push_default_export_nodes(export.export_clause, &mut nodes);
                }
                continue;
            }

            if export_name == "default" || export.export_clause.is_none() {
                continue;
            }

            let Some(clause_node) = arena.get(export.export_clause) else { continue; };
            if clause_node.kind == syntax_kind_ext::NAMED_EXPORTS {
                self.push_named_export_nodes(export.export_clause, export_name, &mut nodes);
                continue;
            }

            if !self.declaration_has_name(export.export_clause, export_name) {
                continue;
            }

            if let Some(sym_id) = binder.file_locals.get(export_name) {
                self.push_symbol_decls(sym_id, &mut nodes);
            } else {
                nodes.push(export.export_clause);
            }
        }

        nodes.sort_by_key(|node| node.0);
        nodes.dedup();
        nodes
    }

    fn exported_names_for_symbol(&self, sym_id: SymbolId) -> Vec<String> {
        let mut names = Vec::new();
        let arena = self.arena();
        let Some(symbol) = self.binder.symbols.get(sym_id) else { return names; };
        let local_name = symbol.escaped_name.as_str();

        let Some(root_node) = arena.get(self.root()) else { return names; };
        let Some(source_file) = arena.get_source_file(root_node) else { return names; };

        for &stmt_idx in &source_file.statements.nodes {
            let Some(stmt_node) = arena.get(stmt_idx) else { continue; };
            if stmt_node.kind != syntax_kind_ext::EXPORT_DECLARATION {
                continue;
            }
            let Some(export) = arena.get_export_decl(stmt_node) else { continue; };
            if !export.module_specifier.is_none() {
                continue;
            }

            if export.is_default_export {
                if !export.export_clause.is_none() && self.resolve_symbol(export.export_clause) == Some(sym_id) {
                    names.push("default".to_string());
                }
                continue;
            }

            if export.export_clause.is_none() {
                continue;
            }

            let Some(clause_node) = arena.get(export.export_clause) else { continue; };
            if clause_node.kind == syntax_kind_ext::NAMED_EXPORTS {
                if let Some(named) = arena.get_named_imports(clause_node) {
                    for &spec_idx in &named.elements.nodes {
                        let Some(spec_node) = arena.get(spec_idx) else { continue; };
                        let Some(spec) = arena.get_specifier(spec_node) else { continue; };

                        let local_ident = if !spec.property_name.is_none() {
                            spec.property_name
                        } else {
                            spec.name
                        };
                        if self.resolve_symbol(local_ident) != Some(sym_id) {
                            continue;
                        }

                        let export_ident = if !spec.name.is_none() {
                            spec.name
                        } else {
                            spec.property_name
                        };
                        if let Some(export_text) = arena.get_identifier_text(export_ident) {
                            names.push(export_text.to_string());
                        }
                    }
                }
                continue;
            }

            if self.declaration_has_name(export.export_clause, local_name) {
                names.push(local_name.to_string());
            }
        }

        names.sort();
        names.dedup();
        names
    }

    fn import_targets_for_local(&self, local_name: &str) -> Vec<ImportTarget> {
        let mut targets = Vec::new();
        let arena = self.arena();

        let Some(root_node) = arena.get(self.root()) else { return targets; };
        let Some(source_file) = arena.get_source_file(root_node) else { return targets; };

        for &stmt_idx in &source_file.statements.nodes {
            let Some(stmt_node) = arena.get(stmt_idx) else { continue; };
            if stmt_node.kind != syntax_kind_ext::IMPORT_DECLARATION
                && stmt_node.kind != syntax_kind_ext::IMPORT_EQUALS_DECLARATION {
                continue;
            }
            let Some(import) = arena.get_import_decl(stmt_node) else { continue; };
            let Some(module_specifier) = arena.get_literal_text(import.module_specifier) else { continue; };
            let module_specifier = module_specifier.to_string();

            if import.import_clause.is_none() {
                continue;
            }

            let Some(clause_node) = arena.get(import.import_clause) else { continue; };
            let Some(clause) = arena.get_import_clause(clause_node) else { continue; };

            if !clause.name.is_none() {
                if let Some(name) = arena.get_identifier_text(clause.name) {
                    if name == local_name {
                        targets.push(ImportTarget {
                            module_specifier: module_specifier.clone(),
                            kind: ImportKind::Default,
                        });
                    }
                }
            }

            if clause.named_bindings.is_none() {
                continue;
            }

            let Some(bindings_node) = arena.get(clause.named_bindings) else { continue; };
            if bindings_node.kind == SyntaxKind::Identifier as u16 {
                if let Some(name) = arena.get_identifier_text(clause.named_bindings) {
                    if name == local_name {
                        targets.push(ImportTarget {
                            module_specifier: module_specifier.clone(),
                            kind: ImportKind::Namespace,
                        });
                    }
                }
                continue;
            }
            let Some(named) = arena.get_named_imports(bindings_node) else { continue; };

            if !named.name.is_none() {
                if let Some(name) = arena.get_identifier_text(named.name) {
                    if name == local_name {
                        targets.push(ImportTarget {
                            module_specifier: module_specifier.clone(),
                            kind: ImportKind::Namespace,
                        });
                    }
                }
            }

            for &spec_idx in &named.elements.nodes {
                let Some(spec_node) = arena.get(spec_idx) else { continue; };
                let Some(spec) = arena.get_specifier(spec_node) else { continue; };

                let local_ident = if !spec.name.is_none() {
                    spec.name
                } else {
                    spec.property_name
                };
                let Some(local_text) = arena.get_identifier_text(local_ident) else { continue; };
                if local_text != local_name {
                    continue;
                }

                let export_ident = if !spec.property_name.is_none() {
                    spec.property_name
                } else {
                    spec.name
                };
                let Some(export_text) = arena.get_identifier_text(export_ident) else { continue; };

                targets.push(ImportTarget {
                    module_specifier: module_specifier.clone(),
                    kind: ImportKind::Named(export_text.to_string()),
                });
            }
        }

        targets
    }

    fn push_default_export_nodes(&self, clause_idx: NodeIndex, nodes: &mut Vec<NodeIndex>) {
        if clause_idx.is_none() {
            return;
        }

        if let Some(&sym_id) = self.binder.node_symbols.get(&clause_idx.0) {
            self.push_symbol_decls(sym_id, nodes);
            return;
        }

        if let Some(sym_id) = self.binder.resolve_identifier(self.arena(), clause_idx) {
            self.push_symbol_decls(sym_id, nodes);
            return;
        }

        nodes.push(clause_idx);
    }

    fn push_named_export_nodes(&self, clause_idx: NodeIndex, export_name: &str, nodes: &mut Vec<NodeIndex>) {
        let arena = self.arena();
        let binder = self.binder();

        let Some(clause_node) = arena.get(clause_idx) else { return; };
        let Some(named) = arena.get_named_imports(clause_node) else { return; };

        for &spec_idx in &named.elements.nodes {
            let Some(spec_node) = arena.get(spec_idx) else { continue; };
            let Some(spec) = arena.get_specifier(spec_node) else { continue; };

            let export_ident = if !spec.name.is_none() {
                spec.name
            } else {
                spec.property_name
            };
            let Some(export_text) = arena.get_identifier_text(export_ident) else { continue; };
            if export_text != export_name {
                continue;
            }

            let local_ident = if !spec.property_name.is_none() {
                spec.property_name
            } else {
                spec.name
            };
            if let Some(local_text) = arena.get_identifier_text(local_ident) {
                if let Some(sym_id) = binder.file_locals.get(local_text) {
                    self.push_symbol_decls(sym_id, nodes);
                } else {
                    nodes.push(spec_idx);
                }
            }
        }
    }

    fn push_symbol_decls(&self, sym_id: SymbolId, nodes: &mut Vec<NodeIndex>) {
        if let Some(symbol) = self.binder.symbols.get(sym_id) {
            nodes.extend(symbol.declarations.iter().copied());
        }
    }

    fn declaration_has_name(&self, decl_idx: NodeIndex, export_name: &str) -> bool {
        let arena = self.arena();
        let Some(node) = arena.get(decl_idx) else { return false; };

        match node.kind {
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => arena
                .get_function(node)
                .and_then(|func| arena.get_identifier_text(func.name))
                .map_or(false, |name| name == export_name),
            k if k == syntax_kind_ext::CLASS_DECLARATION => arena
                .get_class(node)
                .and_then(|class| arena.get_identifier_text(class.name))
                .map_or(false, |name| name == export_name),
            k if k == syntax_kind_ext::INTERFACE_DECLARATION => arena
                .get_interface(node)
                .and_then(|iface| arena.get_identifier_text(iface.name))
                .map_or(false, |name| name == export_name),
            k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => arena
                .get_type_alias(node)
                .and_then(|alias| arena.get_identifier_text(alias.name))
                .map_or(false, |name| name == export_name),
            k if k == syntax_kind_ext::ENUM_DECLARATION => arena
                .get_enum(node)
                .and_then(|enm| arena.get_identifier_text(enm.name))
                .map_or(false, |name| name == export_name),
            k if k == syntax_kind_ext::MODULE_DECLARATION => arena
                .get_module(node)
                .and_then(|module| arena.get_identifier_text(module.name))
                .map_or(false, |name| name == export_name),
            k if k == syntax_kind_ext::VARIABLE_STATEMENT
                || k == syntax_kind_ext::VARIABLE_DECLARATION_LIST
                || k == syntax_kind_ext::VARIABLE_DECLARATION => {
                let mut decls = Vec::new();
                self.collect_variable_declarations(decl_idx, &mut decls);
                decls.into_iter().any(|decl_idx| {
                    let Some(decl_node) = arena.get(decl_idx) else { return false; };
                    arena
                        .get_variable_declaration(decl_node)
                        .and_then(|decl| arena.get_identifier_text(decl.name))
                        .map_or(false, |name| name == export_name)
                })
            }
            _ => false,
        }
    }

    fn collect_variable_declarations(&self, node_idx: NodeIndex, output: &mut Vec<NodeIndex>) {
        let arena = self.arena();
        let Some(node) = arena.get(node_idx) else { return; };

        if node.kind == syntax_kind_ext::VARIABLE_DECLARATION {
            output.push(node_idx);
            return;
        }

        if node.kind == syntax_kind_ext::VARIABLE_STATEMENT || node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
            if let Some(var) = arena.get_variable(node) {
                for &child in &var.declarations.nodes {
                    self.collect_variable_declarations(child, output);
                }
            }
        }
    }
}

fn apply_text_edits(source: &str, line_map: &LineMap, edits: &[TextEdit]) -> Option<String> {
    let mut edits_with_offsets = Vec::with_capacity(edits.len());
    for edit in edits {
        let start = line_map.position_to_offset(edit.range.start, source)? as usize;
        let end = line_map.position_to_offset(edit.range.end, source)? as usize;
        if start > end || end > source.len() {
            return None;
        }
        edits_with_offsets.push((start, end, edit));
    }

    edits_with_offsets.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));

    let mut result = source.to_string();
    for (start, end, edit) in edits_with_offsets {
        result.replace_range(start..end, &edit.new_text);
    }

    Some(result)
}

/// Multi-file container for LSP operations.
pub struct Project {
    files: FxHashMap<String, ProjectFile>,
}

impl Project {
    /// Create a new empty project.
    pub fn new() -> Self {
        Self {
            files: FxHashMap::default(),
        }
    }

    /// Total number of files tracked by the project.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Add or replace a file, re-parsing and re-binding its contents.
    pub fn set_file(&mut self, file_name: String, source_text: String) {
        let file = ProjectFile::new(file_name.clone(), source_text);
        self.files.insert(file_name, file);
    }

    /// Update an existing file by applying incremental text edits.
    pub fn update_file(&mut self, file_name: &str, edits: &[TextEdit]) -> Option<()> {
        if edits.is_empty() {
            return Some(());
        }

        let (updated_source, unchanged) = {
            let file = self.files.get(file_name)?;
            let source = file.source_text();
            let updated = apply_text_edits(source, file.line_map(), edits)?;
            let unchanged = updated == source;
            (updated, unchanged)
        };

        if unchanged {
            return Some(());
        }

        let file = ProjectFile::new(file_name.to_string(), updated_source);
        self.files.insert(file_name.to_string(), file);
        Some(())
    }

    /// Remove a file from the project.
    pub fn remove_file(&mut self, file_name: &str) -> Option<ProjectFile> {
        self.files.remove(file_name)
    }

    /// Fetch a file by name.
    pub fn file(&self, file_name: &str) -> Option<&ProjectFile> {
        self.files.get(file_name)
    }

    /// Go to definition within a single file.
    pub fn get_definition(&self, file_name: &str, position: Position) -> Option<Vec<Location>> {
        let file = self.files.get(file_name)?;
        if let Some(definitions) = self.definition_from_import(file, position) {
            return Some(definitions);
        }
        let goto_def = GoToDefinition::new(
            file.arena(),
            file.binder(),
            file.line_map(),
            file.file_name().to_string(),
            file.source_text(),
        );
        goto_def.get_definition(file.root(), position)
    }

    /// Hover within a single file.
    pub fn get_hover(&mut self, file_name: &str, position: Position) -> Option<HoverInfo> {
        let file = self.files.get_mut(file_name)?;
        file.get_hover(position)
    }

    /// Signature help within a single file.
    pub fn get_signature_help(&mut self, file_name: &str, position: Position) -> Option<SignatureHelp> {
        let file = self.files.get_mut(file_name)?;
        file.get_signature_help(position)
    }

    /// Completions within a single file.
    pub fn get_completions(&mut self, file_name: &str, position: Position) -> Option<Vec<CompletionItem>> {
        let file = self.files.get_mut(file_name)?;
        file.get_completions(position)
    }

    /// Code actions for a file (project-aware).
    pub fn get_code_actions(
        &self,
        file_name: &str,
        range: Range,
        diagnostics: Vec<LspDiagnostic>,
        only: Option<Vec<CodeActionKind>>,
    ) -> Option<Vec<CodeAction>> {
        let file = self.files.get(file_name)?;
        let import_candidates = self.import_candidates_for_diagnostics(file, &diagnostics);

        let provider = CodeActionProvider::new(
            file.arena(),
            file.binder(),
            file.line_map(),
            file.file_name().to_string(),
            file.source_text(),
        );

        let actions = provider.provide_code_actions(
            file.root(),
            range,
            CodeActionContext {
                diagnostics,
                only,
                import_candidates,
            },
        );

        if actions.is_empty() {
            None
        } else {
            Some(actions)
        }
    }

    fn collect_file_references(&self, file: &ProjectFile, node_idx: NodeIndex, output: &mut Vec<Location>) {
        if node_idx.is_none() {
            return;
        }

        let find_refs = FindReferences::new(
            file.arena(),
            file.binder(),
            file.line_map(),
            file.file_name().to_string(),
            file.source_text(),
        );

        if let Some(mut refs) = find_refs.find_references_for_node(file.root(), node_idx) {
            output.append(&mut refs);
        }
    }

    fn import_binding_nodes(&self, file: &ProjectFile, target_file: &str, export_name: &str) -> Vec<NodeIndex> {
        let mut bindings = Vec::new();
        let arena = file.arena();

        let Some(root_node) = arena.get(file.root()) else { return bindings; };
        let Some(source_file) = arena.get_source_file(root_node) else { return bindings; };

        for &stmt_idx in &source_file.statements.nodes {
            let Some(stmt_node) = arena.get(stmt_idx) else { continue; };
            if stmt_node.kind != syntax_kind_ext::IMPORT_DECLARATION
                && stmt_node.kind != syntax_kind_ext::IMPORT_EQUALS_DECLARATION {
                continue;
            }

            let Some(import) = arena.get_import_decl(stmt_node) else { continue; };
            let Some(module_specifier) = arena.get_literal_text(import.module_specifier) else { continue; };
            let Some(resolved) = self.resolve_module_specifier(file.file_name(), module_specifier) else { continue; };
            if resolved != target_file {
                continue;
            }

            if import.import_clause.is_none() {
                continue;
            }

            let Some(clause_node) = arena.get(import.import_clause) else { continue; };
            let Some(clause) = arena.get_import_clause(clause_node) else { continue; };

            if export_name == "default" && !clause.name.is_none() {
                bindings.push(clause.name);
            }

            if clause.named_bindings.is_none() {
                continue;
            }

            let Some(bindings_node) = arena.get(clause.named_bindings) else { continue; };
            let Some(named) = arena.get_named_imports(bindings_node) else { continue; };

            for &spec_idx in &named.elements.nodes {
                let Some(spec_node) = arena.get(spec_idx) else { continue; };
                let Some(spec) = arena.get_specifier(spec_node) else { continue; };

                let export_ident = if !spec.property_name.is_none() {
                    spec.property_name
                } else {
                    spec.name
                };
                let Some(imported_name) = arena.get_identifier_text(export_ident) else { continue; };
                if imported_name != export_name {
                    continue;
                }

                bindings.push(spec_idx);
            }
        }

        bindings
    }

    fn named_import_local_names(&self, file: &ProjectFile, target_file: &str, export_name: &str) -> Vec<String> {
        let mut locals = Vec::new();
        let arena = file.arena();

        let Some(root_node) = arena.get(file.root()) else { return locals; };
        let Some(source_file) = arena.get_source_file(root_node) else { return locals; };

        for &stmt_idx in &source_file.statements.nodes {
            let Some(stmt_node) = arena.get(stmt_idx) else { continue; };
            if stmt_node.kind != syntax_kind_ext::IMPORT_DECLARATION
                && stmt_node.kind != syntax_kind_ext::IMPORT_EQUALS_DECLARATION {
                continue;
            }

            let Some(import) = arena.get_import_decl(stmt_node) else { continue; };
            let Some(module_specifier) = arena.get_literal_text(import.module_specifier) else { continue; };
            let Some(resolved) = self.resolve_module_specifier(file.file_name(), module_specifier) else { continue; };
            if resolved != target_file {
                continue;
            }

            if import.import_clause.is_none() {
                continue;
            }

            let Some(clause_node) = arena.get(import.import_clause) else { continue; };
            let Some(clause) = arena.get_import_clause(clause_node) else { continue; };

            if clause.named_bindings.is_none() {
                continue;
            }

            let Some(bindings_node) = arena.get(clause.named_bindings) else { continue; };
            if bindings_node.kind == SyntaxKind::Identifier as u16 {
                continue;
            }

            let Some(named) = arena.get_named_imports(bindings_node) else { continue; };

            for &spec_idx in &named.elements.nodes {
                let Some(spec_node) = arena.get(spec_idx) else { continue; };
                let Some(spec) = arena.get_specifier(spec_node) else { continue; };

                let export_ident = if !spec.property_name.is_none() {
                    spec.property_name
                } else {
                    spec.name
                };
                let Some(export_text) = arena.get_identifier_text(export_ident) else { continue; };
                if export_text != export_name {
                    continue;
                }

                let local_ident = if !spec.name.is_none() {
                    spec.name
                } else {
                    spec.property_name
                };
                let Some(local_text) = arena.get_identifier_text(local_ident) else { continue; };
                locals.push(local_text.to_string());
            }
        }

        locals
    }

    fn reexport_targets_for(
        &self,
        source_file: &str,
        export_name: &str,
        refs: &mut Vec<Location>,
    ) -> (Vec<(String, String)>, Vec<NamespaceReexportTarget>) {
        let mut targets = Vec::new();
        let mut namespace_targets = Vec::new();

        for (file_name, file) in &self.files {
            let arena = file.arena();
            let Some(root_node) = arena.get(file.root()) else { continue; };
            let Some(source_file_node) = arena.get_source_file(root_node) else { continue; };

            for &stmt_idx in &source_file_node.statements.nodes {
                let Some(stmt_node) = arena.get(stmt_idx) else { continue; };
                if stmt_node.kind != syntax_kind_ext::EXPORT_DECLARATION {
                    continue;
                }

                let Some(export) = arena.get_export_decl(stmt_node) else { continue; };
                if export.module_specifier.is_none() {
                    continue;
                }

                let Some(module_specifier) = arena.get_literal_text(export.module_specifier) else { continue; };
                let Some(resolved) = self.resolve_module_specifier(file.file_name(), module_specifier) else {
                    continue;
                };
                if resolved != source_file {
                    continue;
                }

                if export.export_clause.is_none() {
                    if export_name != "default" {
                        targets.push((file_name.clone(), export_name.to_string()));
                    }
                    continue;
                }

                let Some(clause_node) = arena.get(export.export_clause) else { continue; };
                if clause_node.kind != syntax_kind_ext::NAMED_EXPORTS {
                    if clause_node.kind == SyntaxKind::Identifier as u16 {
                        if let Some(ns_name) = arena.get_identifier_text(export.export_clause) {
                            namespace_targets.push(NamespaceReexportTarget {
                                file: file_name.clone(),
                                namespace: ns_name.to_string(),
                                member: export_name.to_string(),
                            });
                        }
                    }
                    continue;
                }

                let Some(named) = arena.get_named_imports(clause_node) else { continue; };
                for &spec_idx in &named.elements.nodes {
                    let Some(spec_node) = arena.get(spec_idx) else { continue; };
                    let Some(spec) = arena.get_specifier(spec_node) else { continue; };

                    let import_ident = if !spec.property_name.is_none() {
                        spec.property_name
                    } else {
                        spec.name
                    };
                    let Some(import_text) = arena.get_identifier_text(import_ident) else { continue; };
                    if import_text != export_name {
                        continue;
                    }

                    if let Some(location) = file.node_location(import_ident) {
                        refs.push(location);
                    }

                    let export_ident = if !spec.name.is_none() {
                        spec.name
                    } else {
                        spec.property_name
                    };
                    if let Some(export_text) = arena.get_identifier_text(export_ident) {
                        targets.push((file_name.clone(), export_text.to_string()));
                    }
                }
            }
        }

        (targets, namespace_targets)
    }

    fn namespace_import_names(&self, file: &ProjectFile, target_file: &str) -> Vec<String> {
        let mut names = Vec::new();
        let arena = file.arena();

        let Some(root_node) = arena.get(file.root()) else { return names; };
        let Some(source_file) = arena.get_source_file(root_node) else { return names; };

        for &stmt_idx in &source_file.statements.nodes {
            let Some(stmt_node) = arena.get(stmt_idx) else { continue; };
            if stmt_node.kind != syntax_kind_ext::IMPORT_DECLARATION
                && stmt_node.kind != syntax_kind_ext::IMPORT_EQUALS_DECLARATION {
                continue;
            }

            let Some(import) = arena.get_import_decl(stmt_node) else { continue; };
            let Some(module_specifier) = arena.get_literal_text(import.module_specifier) else { continue; };
            let Some(resolved) = self.resolve_module_specifier(file.file_name(), module_specifier) else { continue; };
            if resolved != target_file {
                continue;
            }

            if import.import_clause.is_none() {
                continue;
            }

            let Some(clause_node) = arena.get(import.import_clause) else { continue; };
            let Some(clause) = arena.get_import_clause(clause_node) else { continue; };

            if clause.named_bindings.is_none() {
                continue;
            }

            let Some(bindings_node) = arena.get(clause.named_bindings) else { continue; };
            if bindings_node.kind != syntax_kind_ext::NAMESPACE_IMPORT {
                continue;
            }

            let Some(bindings) = arena.get_named_imports(bindings_node) else { continue; };
            if let Some(name) = arena.get_identifier_text(bindings.name) {
                names.push(name.to_string());
            }
        }

        names
    }

    fn collect_namespace_member_locations(
        &self,
        file: &ProjectFile,
        namespace_name: &str,
        export_name: &str,
        output: &mut Vec<Location>,
    ) {
        let arena = file.arena();
        let expected_symbol = file.binder().file_locals.get(namespace_name);

        for node in arena.nodes.iter() {
            if node.kind != syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION
                && node.kind != syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION {
                continue;
            }

            let Some(access) = arena.get_access_expr(node) else { continue; };
            let expr_idx = access.expression;
            let Some(expr_node) = arena.get(expr_idx) else { continue; };
            if expr_node.kind != SyntaxKind::Identifier as u16 {
                continue;
            }

            let Some(expr_text) = arena.get_identifier_text(expr_idx) else { continue; };
            if expr_text != namespace_name {
                continue;
            }

            if let Some(sym_id) = expected_symbol {
                if file.binder().resolve_identifier(arena, expr_idx) != Some(sym_id) {
                    continue;
                }
            }

            let member_idx = access.name_or_argument;
            let matches = if node.kind == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION {
                arena.get_identifier_text(member_idx).map_or(false, |name| name == export_name)
            } else {
                arena.get_literal_text(member_idx).map_or(false, |name| name == export_name)
            };

            if !matches {
                continue;
            }

            if let Some(location) = file.node_location(member_idx) {
                output.push(location);
            }
        }
    }

    /// Find references within a single file.
    pub fn find_references(&self, file_name: &str, position: Position) -> Option<Vec<Location>> {
        let file = self.files.get(file_name)?;
        let offset = file.line_map().position_to_offset(position, file.source_text())?;
        let node_idx = find_node_at_offset(file.arena(), offset);
        if node_idx.is_none() {
            return None;
        }

        let symbol_id = file.resolve_symbol(node_idx)?;
        let symbol = file.binder().symbols.get(symbol_id)?;
        let local_name = symbol.escaped_name.clone();

        let mut locations = Vec::new();
        self.collect_file_references(file, node_idx, &mut locations);

        let mut cross_targets: Vec<(String, String)> = Vec::new();
        let import_targets = file.import_targets_for_local(&local_name);

        if !import_targets.is_empty() {
            for target in import_targets {
                let Some(resolved) = self.resolve_module_specifier(file.file_name(), &target.module_specifier) else {
                    continue;
                };
                match target.kind {
                    ImportKind::Named(name) => cross_targets.push((resolved, name)),
                    ImportKind::Default => cross_targets.push((resolved, "default".to_string())),
                    ImportKind::Namespace => {}
                }
            }
        } else {
            for export_name in file.exported_names_for_symbol(symbol_id) {
                cross_targets.push((file.file_name().to_string(), export_name));
            }
        }

        let mut expanded_targets = Vec::new();
        let mut pending = cross_targets;
        let mut seen_targets: FxHashSet<(String, String)> = FxHashSet::default();
        let mut namespace_targets = Vec::new();

        while let Some((def_file, export_name)) = pending.pop() {
            if !seen_targets.insert((def_file.clone(), export_name.clone())) {
                continue;
            }
            expanded_targets.push((def_file.clone(), export_name.clone()));

            let mut reexport_refs = Vec::new();
            let (reexports, reexport_namespaces) =
                self.reexport_targets_for(&def_file, &export_name, &mut reexport_refs);
            locations.extend(reexport_refs);
            pending.extend(reexports);
            namespace_targets.extend(reexport_namespaces);
        }

        for (def_file, export_name) in expanded_targets {
            if let Some(target_file) = self.files.get(&def_file) {
                let export_nodes = target_file.export_nodes(&export_name);
                for node in export_nodes {
                    self.collect_file_references(target_file, node, &mut locations);
                }
            }

            for (other_name, other_file) in &self.files {
                if other_name == &def_file {
                    continue;
                }

                let binding_nodes = self.import_binding_nodes(other_file, &def_file, &export_name);
                for node in binding_nodes {
                    self.collect_file_references(other_file, node, &mut locations);
                }

                for namespace_name in self.namespace_import_names(other_file, &def_file) {
                    self.collect_namespace_member_locations(other_file, &namespace_name, &export_name, &mut locations);
                }
            }
        }

        let mut seen_namespace_targets: FxHashSet<(String, String, String)> = FxHashSet::default();
        for target in namespace_targets {
            if !seen_namespace_targets.insert((
                target.file.clone(),
                target.namespace.clone(),
                target.member.clone(),
            )) {
                continue;
            }

            for (other_name, other_file) in &self.files {
                if other_name == &target.file {
                    continue;
                }

                let local_names = self.named_import_local_names(other_file, &target.file, &target.namespace);
                for local_name in local_names {
                    self.collect_namespace_member_locations(other_file, &local_name, &target.member, &mut locations);
                }
            }
        }

        if locations.is_empty() {
            return None;
        }

        locations.sort_by(|a, b| {
            let file_cmp = a.file_path.cmp(&b.file_path);
            if file_cmp != Ordering::Equal {
                return file_cmp;
            }
            let start_cmp = (a.range.start.line, a.range.start.character)
                .cmp(&(b.range.start.line, b.range.start.character));
            if start_cmp != Ordering::Equal {
                return start_cmp;
            }
            (a.range.end.line, a.range.end.character)
                .cmp(&(b.range.end.line, b.range.end.character))
        });
        locations.dedup_by(|a, b| a.file_path == b.file_path && a.range == b.range);

        Some(locations)
    }

    fn definition_from_import(&self, file: &ProjectFile, position: Position) -> Option<Vec<Location>> {
        let target = self.import_target_at_position(file, position)?;
        let resolved = self.resolve_module_specifier(file.file_name(), &target.module_specifier)?;
        let target_file = self.files.get(&resolved)?;

        match target.kind {
            ImportKind::Namespace => {
                let location = target_file.node_location(target_file.root())?;
                Some(vec![location])
            }
            ImportKind::Default => {
                let locations = target_file.export_locations("default");
                if locations.is_empty() { None } else { Some(locations) }
            }
            ImportKind::Named(name) => {
                let locations = target_file.export_locations(&name);
                if locations.is_empty() { None } else { Some(locations) }
            }
        }
    }

    fn import_candidates_for_diagnostics(
        &self,
        file: &ProjectFile,
        diagnostics: &[LspDiagnostic],
    ) -> Vec<ImportCandidate> {
        let mut candidates = Vec::new();
        let mut seen = FxHashSet::default();

        for diag in diagnostics {
            if diag.code != Some(crate::checker::types::diagnostics::diagnostic_codes::CANNOT_FIND_NAME) {
                continue;
            }

            let Some(missing_name) = self.identifier_at_range(file, diag.range) else {
                continue;
            };

            self.collect_import_candidates_for_name(file, &missing_name, &mut candidates, &mut seen);
        }

        candidates
    }

    fn collect_import_candidates_for_name(
        &self,
        from_file: &ProjectFile,
        missing_name: &str,
        output: &mut Vec<ImportCandidate>,
        seen: &mut FxHashSet<(String, String, String, bool)>,
    ) {
        for (file_name, _file) in &self.files {
            if file_name == from_file.file_name() {
                continue;
            }

            let Some(module_specifier) = self.module_specifier_from_files(from_file.file_name(), file_name) else {
                continue;
            };

            let mut visited = FxHashSet::default();
            let matches = self.matching_exports_in_file(file_name, missing_name, &mut visited);

            for export_match in matches {
                let candidate = ImportCandidate {
                    module_specifier: module_specifier.clone(),
                    local_name: missing_name.to_string(),
                    kind: export_match.kind,
                    is_type_only: export_match.is_type_only,
                };

                let kind_key = match &candidate.kind {
                    ImportCandidateKind::Named { export_name } => format!("named:{}", export_name),
                    ImportCandidateKind::Default => "default".to_string(),
                    ImportCandidateKind::Namespace => "namespace".to_string(),
                };

                if seen.insert((
                    candidate.module_specifier.clone(),
                    candidate.local_name.clone(),
                    kind_key,
                    candidate.is_type_only,
                )) {
                    output.push(candidate);
                }
            }
        }
    }

    fn matching_exports_in_file(
        &self,
        file_name: &str,
        export_name: &str,
        visited: &mut FxHashSet<String>,
    ) -> Vec<ExportMatch> {
        if !visited.insert(file_name.to_string()) {
            return Vec::new();
        }

        let Some(file) = self.files.get(file_name) else { return Vec::new(); };
        let arena = file.arena();
        let Some(root_node) = arena.get(file.root()) else { return Vec::new(); };
        let Some(source_file) = arena.get_source_file(root_node) else { return Vec::new(); };

        let mut matches = Vec::new();

        for &stmt_idx in &source_file.statements.nodes {
            let Some(stmt_node) = arena.get(stmt_idx) else { continue; };
            if stmt_node.kind != syntax_kind_ext::EXPORT_DECLARATION {
                continue;
            }

            let Some(export) = arena.get_export_decl(stmt_node) else { continue; };

            if export.is_default_export {
                matches.push(ExportMatch {
                    kind: ImportCandidateKind::Default,
                    is_type_only: export.is_type_only,
                });
                continue;
            }

            if export.module_specifier.is_none() {
                if export.export_clause.is_none() {
                    continue;
                }

                let Some(clause_node) = arena.get(export.export_clause) else { continue; };
                if clause_node.kind == syntax_kind_ext::NAMED_EXPORTS {
                    let Some(named) = arena.get_named_imports(clause_node) else { continue; };
                    for &spec_idx in &named.elements.nodes {
                        let Some(spec_node) = arena.get(spec_idx) else { continue; };
                        let Some(spec) = arena.get_specifier(spec_node) else { continue; };

                        let export_ident = if !spec.name.is_none() {
                            spec.name
                        } else {
                            spec.property_name
                        };
                        let Some(export_text) = arena.get_identifier_text(export_ident) else { continue; };
                        if export_text == "default" {
                            matches.push(ExportMatch {
                                kind: ImportCandidateKind::Default,
                                is_type_only: export.is_type_only || spec.is_type_only,
                            });
                        }
                        if export_text != export_name {
                            continue;
                        }

                        matches.push(ExportMatch {
                            kind: ImportCandidateKind::Named {
                                export_name: export_text.to_string(),
                            },
                            is_type_only: export.is_type_only || spec.is_type_only,
                        });
                    }
                } else if file.declaration_has_name(export.export_clause, export_name) {
                    matches.push(ExportMatch {
                        kind: ImportCandidateKind::Named {
                            export_name: export_name.to_string(),
                        },
                        is_type_only: export.is_type_only,
                    });
                }

                continue;
            }

            let module_specifier = match arena.get_literal_text(export.module_specifier) {
                Some(text) => text,
                None => continue,
            };
            let resolved = match self.resolve_module_specifier(file.file_name(), module_specifier) {
                Some(path) => path,
                None => continue,
            };

            if export.export_clause.is_none() {
                if export_name == "default" {
                    continue;
                }

                if self.file_exports_named(&resolved, export_name, visited) {
                    matches.push(ExportMatch {
                        kind: ImportCandidateKind::Named {
                            export_name: export_name.to_string(),
                        },
                        is_type_only: export.is_type_only,
                    });
                }

                continue;
            }

            let Some(clause_node) = arena.get(export.export_clause) else { continue; };
            if clause_node.kind == syntax_kind_ext::NAMED_EXPORTS {
                let Some(named) = arena.get_named_imports(clause_node) else { continue; };
                for &spec_idx in &named.elements.nodes {
                    let Some(spec_node) = arena.get(spec_idx) else { continue; };
                    let Some(spec) = arena.get_specifier(spec_node) else { continue; };

                    let export_ident = if !spec.name.is_none() {
                        spec.name
                    } else {
                        spec.property_name
                    };
                    let Some(export_text) = arena.get_identifier_text(export_ident) else { continue; };
                    if export_text == "default" {
                        matches.push(ExportMatch {
                            kind: ImportCandidateKind::Default,
                            is_type_only: export.is_type_only || spec.is_type_only,
                        });
                    }
                    if export_text != export_name {
                        continue;
                    }

                    matches.push(ExportMatch {
                        kind: ImportCandidateKind::Named {
                            export_name: export_text.to_string(),
                        },
                        is_type_only: export.is_type_only || spec.is_type_only,
                    });
                }
            } else if clause_node.kind == SyntaxKind::Identifier as u16 {
                if let Some(export_text) = arena.get_identifier_text(export.export_clause) {
                    if export_text == export_name {
                        matches.push(ExportMatch {
                            kind: ImportCandidateKind::Named {
                                export_name: export_text.to_string(),
                            },
                            is_type_only: export.is_type_only,
                        });
                    }
                }
            }
        }

        matches
    }

    fn file_exports_named(
        &self,
        file_name: &str,
        export_name: &str,
        visited: &mut FxHashSet<String>,
    ) -> bool {
        self.matching_exports_in_file(file_name, export_name, visited)
            .iter()
            .any(|export_match| matches!(export_match.kind, ImportCandidateKind::Named { .. }))
    }

    fn identifier_at_range(&self, file: &ProjectFile, range: Range) -> Option<String> {
        let offset = file.line_map().position_to_offset(range.start, file.source_text())?;
        let node_idx = find_node_at_offset(file.arena(), offset);
        if node_idx.is_none() {
            return None;
        }

        let node = file.arena().get(node_idx)?;
        if node.kind != SyntaxKind::Identifier as u16 {
            return None;
        }

        file.arena()
            .get_identifier_text(node_idx)
            .map(|text| text.to_string())
    }

    fn import_target_at_position(&self, file: &ProjectFile, position: Position) -> Option<ImportTarget> {
        let offset = file.line_map().position_to_offset(position, file.source_text())?;
        let node_idx = find_node_at_offset(file.arena(), offset);
        if node_idx.is_none() {
            return None;
        }
        self.import_target_from_node(file, node_idx)
    }

    fn import_target_from_node(&self, file: &ProjectFile, node_idx: NodeIndex) -> Option<ImportTarget> {
        let arena = file.arena();
        let mut current = node_idx;
        let mut import_specifier = None;
        let mut import_clause = None;
        let mut import_decl = None;

        while !current.is_none() {
            let node = arena.get(current)?;
            match node.kind {
                k if k == syntax_kind_ext::IMPORT_SPECIFIER => {
                    import_specifier = Some(current);
                }
                k if k == syntax_kind_ext::IMPORT_CLAUSE => {
                    import_clause = Some(current);
                }
                k if k == syntax_kind_ext::IMPORT_DECLARATION
                    || k == syntax_kind_ext::IMPORT_EQUALS_DECLARATION => {
                    import_decl = Some(current);
                    break;
                }
                _ => {}
            }
            current = arena.get_extended(current)?.parent;
        }

        let import_decl_idx = import_decl?;
        let import_decl_node = arena.get(import_decl_idx)?;
        let import_decl = arena.get_import_decl(import_decl_node)?;
        let module_specifier = arena.get_literal_text(import_decl.module_specifier)?.to_string();

        let kind = if let Some(spec_idx) = import_specifier {
            let spec_node = arena.get(spec_idx)?;
            let spec = arena.get_specifier(spec_node)?;
            let export_ident = if !spec.property_name.is_none() {
                spec.property_name
            } else {
                spec.name
            };
            let export_name = arena.get_identifier_text(export_ident)?.to_string();
            ImportKind::Named(export_name)
        } else if let Some(clause_idx) = import_clause {
            let clause_node = arena.get(clause_idx)?;
            let clause = arena.get_import_clause(clause_node)?;

            if clause.name == node_idx {
                ImportKind::Default
            } else if clause.named_bindings == node_idx {
                ImportKind::Namespace
            } else if import_decl.module_specifier == node_idx {
                ImportKind::Namespace
            } else {
                return None;
            }
        } else if import_decl.module_specifier == node_idx {
            ImportKind::Namespace
        } else {
            return None;
        };

        Some(ImportTarget {
            module_specifier,
            kind,
        })
    }

    fn resolve_module_specifier(&self, from_file: &str, module_specifier: &str) -> Option<String> {
        let candidates = self.module_specifier_candidates(from_file, module_specifier);
        candidates
            .into_iter()
            .find(|candidate| self.files.contains_key(candidate))
    }

    fn module_specifier_from_files(&self, from_file: &str, target_file: &str) -> Option<String> {
        let from_dir = Path::new(from_file).parent().unwrap_or_else(|| Path::new(""));
        let target_path = strip_ts_extension(Path::new(target_file));
        let relative = relative_path(from_dir, &target_path);

        let mut spec = path_to_string(&relative).replace('\\', "/");
        if spec.is_empty() {
            return None;
        }
        if !spec.starts_with('.') {
            spec = format!("./{}", spec);
        }
        Some(spec)
    }

    fn module_specifier_candidates(&self, from_file: &str, module_specifier: &str) -> Vec<String> {
        let mut candidates = Vec::new();

        if module_specifier.starts_with('.') {
            let base_dir = Path::new(from_file).parent().unwrap_or_else(|| Path::new(""));
            let joined = normalize_path(&base_dir.join(module_specifier));

            if joined.extension().is_some() {
                candidates.push(path_to_string(&joined));
            } else {
                for ext in TS_EXTENSION_CANDIDATES {
                    candidates.push(path_to_string(&joined.with_extension(ext)));
                }
                for ext in TS_EXTENSION_CANDIDATES {
                    candidates.push(path_to_string(&joined.join("index").with_extension(ext)));
                }
            }
        } else {
            candidates.push(module_specifier.to_string());
            if Path::new(module_specifier).extension().is_none() {
                for ext in TS_EXTENSION_CANDIDATES {
                    candidates.push(format!("{}.{}", module_specifier, ext));
                }
            }
        }

        candidates
    }
}

const TS_EXTENSION_CANDIDATES: [&str; 7] = ["ts", "tsx", "d.ts", "mts", "cts", "d.mts", "d.cts"];
const TS_EXTENSION_SUFFIXES: [&str; 7] = [".d.ts", ".d.mts", ".d.cts", ".ts", ".tsx", ".mts", ".cts"];

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Normal(_) | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    normalized
}

fn strip_ts_extension(path: &Path) -> PathBuf {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return path.to_path_buf();
    };

    for suffix in TS_EXTENSION_SUFFIXES {
        if file_name.ends_with(suffix) {
            let base_name = &file_name[..file_name.len() - suffix.len()];
            if base_name.is_empty() {
                return path.to_path_buf();
            }
            let mut base = PathBuf::new();
            if let Some(parent) = path.parent() {
                base.push(parent);
            }
            base.push(base_name);
            return base;
        }
    }

    path.to_path_buf()
}

fn relative_path(from: &Path, to: &Path) -> PathBuf {
    let from_components: Vec<_> = from
        .components()
        .filter(|c| *c != Component::CurDir)
        .collect();
    let to_components: Vec<_> = to
        .components()
        .filter(|c| *c != Component::CurDir)
        .collect();

    let mut common = 0;
    while common < from_components.len()
        && common < to_components.len()
        && from_components[common] == to_components[common]
    {
        common += 1;
    }

    let mut result = PathBuf::new();
    for _ in common..from_components.len() {
        result.push("..");
    }
    for component in &to_components[common..] {
        result.push(component.as_os_str());
    }

    if result.as_os_str().is_empty() {
        result.push(".");
    }

    result
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
