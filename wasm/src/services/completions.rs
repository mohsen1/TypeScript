//! Code Completions implementation.
//!
//! This module provides IntelliSense completions including:
//! - Property and method completions
//! - Variable and function completions
//! - Import completions (auto-import)
//! - Keyword completions
//! - Snippet completions

use std::collections::HashSet;
use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::{TypeId, CheckerState};
use crate::parser::{Node, NodeIndex, NodeArena};
use crate::scanner::SyntaxKind;

use super::symbol_display::{SymbolDisplayPart, DisplayPartsBuilder};
use super::text_span::TextSpan;
use super::utilities::ScriptElementKind;
use super::export_info_map::{ExportInfoMap, SymbolExportInfo};

// =============================================================================
// Completion Types
// =============================================================================

/// The result of a completion request.
#[derive(Debug, Clone)]
pub struct CompletionInfo {
    /// Whether this is a global completion.
    pub is_global_completion: bool,
    /// Whether this is a member completion.
    pub is_member_completion: bool,
    /// Whether this is a new identifier location.
    pub is_new_identifier_location: bool,
    /// The completion entries.
    pub entries: Vec<CompletionEntry>,
    /// Optional span to replace.
    pub optional_replacement_span: Option<TextSpan>,
}

impl CompletionInfo {
    pub fn new(entries: Vec<CompletionEntry>) -> Self {
        Self {
            is_global_completion: false,
            is_member_completion: false,
            is_new_identifier_location: false,
            entries,
            optional_replacement_span: None,
        }
    }

    pub fn global(entries: Vec<CompletionEntry>) -> Self {
        Self {
            is_global_completion: true,
            is_member_completion: false,
            is_new_identifier_location: true,
            entries,
            optional_replacement_span: None,
        }
    }

    pub fn member(entries: Vec<CompletionEntry>) -> Self {
        Self {
            is_global_completion: false,
            is_member_completion: true,
            is_new_identifier_location: false,
            entries,
            optional_replacement_span: None,
        }
    }
}

/// A single completion entry.
#[derive(Debug, Clone)]
pub struct CompletionEntry {
    /// The name to insert.
    pub name: String,
    /// The kind of completion.
    pub kind: ScriptElementKind,
    /// Kind modifiers.
    pub kind_modifiers: String,
    /// Sort text for ordering.
    pub sort_text: String,
    /// Text to insert (if different from name).
    pub insert_text: Option<String>,
    /// Whether insert text is a snippet.
    pub is_snippet: Option<bool>,
    /// Replacement span.
    pub replacement_span: Option<TextSpan>,
    /// Whether this has action (e.g., auto-import).
    pub has_action: Option<bool>,
    /// Source module (for auto-imports).
    pub source: Option<String>,
    /// Whether this is recommended.
    pub is_recommended: Option<bool>,
    /// Whether this is from an unchecked file.
    pub is_from_unchecked_file: Option<bool>,
    /// Label details.
    pub label_details: Option<CompletionEntryLabelDetails>,
    /// Data for resolve.
    pub data: Option<CompletionEntryData>,
}

impl CompletionEntry {
    pub fn new(name: String, kind: ScriptElementKind) -> Self {
        Self {
            name,
            kind,
            kind_modifiers: String::new(),
            sort_text: String::new(),
            insert_text: None,
            is_snippet: None,
            replacement_span: None,
            has_action: None,
            source: None,
            is_recommended: None,
            is_from_unchecked_file: None,
            label_details: None,
            data: None,
        }
    }

    pub fn with_sort_text(mut self, sort_text: String) -> Self {
        self.sort_text = sort_text;
        self
    }

    pub fn with_insert_text(mut self, text: String) -> Self {
        self.insert_text = Some(text);
        self
    }

    pub fn with_snippet(mut self, snippet: String) -> Self {
        self.insert_text = Some(snippet);
        self.is_snippet = Some(true);
        self
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self.has_action = Some(true);
        self
    }

    pub fn recommended(mut self) -> Self {
        self.is_recommended = Some(true);
        self
    }
}

/// Label details for completion entry.
#[derive(Debug, Clone)]
pub struct CompletionEntryLabelDetails {
    /// Detail text (shown after the label).
    pub detail: Option<String>,
    /// Description text (shown in a different color).
    pub description: Option<String>,
}

/// Data for resolving a completion entry.
#[derive(Debug, Clone)]
pub struct CompletionEntryData {
    /// The file name.
    pub file_name: String,
    /// The position.
    pub position: u32,
    /// The entry name.
    pub name: String,
    /// The source module.
    pub source: Option<String>,
    /// Export info for auto-imports.
    pub export_info: Option<SymbolExportInfo>,
}

// =============================================================================
// Completion Entry Details
// =============================================================================

/// Detailed information about a completion entry.
#[derive(Debug, Clone)]
pub struct CompletionEntryDetails {
    /// The name of the completion.
    pub name: String,
    /// The kind of completion.
    pub kind: ScriptElementKind,
    /// Kind modifiers.
    pub kind_modifiers: String,
    /// Display parts showing the signature.
    pub display_parts: Vec<SymbolDisplayPart>,
    /// Documentation.
    pub documentation: Vec<SymbolDisplayPart>,
    /// JSDoc tags.
    pub tags: Vec<JSDocTagInfo>,
    /// Code actions to apply (e.g., for auto-import).
    pub code_actions: Vec<CodeAction>,
    /// Source module.
    pub source: Option<Vec<SymbolDisplayPart>>,
    /// Source display for auto-imports.
    pub source_display: Option<Vec<SymbolDisplayPart>>,
}

impl CompletionEntryDetails {
    pub fn new(name: String, kind: ScriptElementKind) -> Self {
        Self {
            name,
            kind,
            kind_modifiers: String::new(),
            display_parts: Vec::new(),
            documentation: Vec::new(),
            tags: Vec::new(),
            code_actions: Vec::new(),
            source: None,
            source_display: None,
        }
    }
}

/// JSDoc tag information.
#[derive(Debug, Clone)]
pub struct JSDocTagInfo {
    pub name: String,
    pub text: Option<Vec<SymbolDisplayPart>>,
}

/// A code action to apply.
#[derive(Debug, Clone)]
pub struct CodeAction {
    pub description: String,
    pub changes: Vec<FileTextChanges>,
}

/// Text changes for a file.
#[derive(Debug, Clone)]
pub struct FileTextChanges {
    pub file_name: String,
    pub text_changes: Vec<TextChange>,
    pub is_new_file: Option<bool>,
}

/// A single text change.
#[derive(Debug, Clone)]
pub struct TextChange {
    pub span: TextSpan,
    pub new_text: String,
}

// =============================================================================
// Completion Options and Context
// =============================================================================

/// Trigger kind for completions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionTriggerKind {
    /// Completion was invoked manually.
    Invoked,
    /// Completion was triggered by a character.
    TriggerCharacter,
    /// Completion was re-triggered.
    TriggerForIncompleteCompletions,
}

/// Options for completions.
#[derive(Debug, Clone, Default)]
pub struct GetCompletionsAtPositionOptions {
    /// Whether to include auto-imports.
    pub include_automatic_optional_chain_completions: Option<bool>,
    /// Whether to include completions with insertText.
    pub include_completions_with_insert_text: Option<bool>,
    /// Whether to include completions for module exports.
    pub include_completions_for_module_exports: Option<bool>,
    /// Whether to include completions with snippet text.
    pub include_completions_with_snippet_text: Option<bool>,
    /// Whether to include completions with class member snippets.
    pub include_completions_with_class_member_snippets: Option<bool>,
    /// Whether to include completions with object literal method snippets.
    pub include_completions_with_object_literal_method_snippets: Option<bool>,
    /// Whether to use label details in completions.
    pub use_label_details_in_completion_entries: Option<bool>,
    /// Import statement to use.
    pub allow_incomplete_completions: Option<bool>,
    /// Provide prefix and suffix text.
    pub provide_prefix_and_suffix_text_for_rename: Option<bool>,
    /// Trigger character.
    pub trigger_character: Option<char>,
    /// Trigger kind.
    pub trigger_kind: Option<CompletionTriggerKind>,
}

/// Context for completion operations.
pub struct CompletionsContext<'a> {
    pub arena: &'a NodeArena,
    pub checker: &'a CheckerState,
    pub source_text: &'a str,
    pub file_name: &'a str,
    pub export_map: Option<&'a ExportInfoMap>,
    pub options: GetCompletionsAtPositionOptions,
}

// =============================================================================
// Get Completions
// =============================================================================

/// Get completions at a position.
pub fn get_completions_at_position(
    ctx: &CompletionsContext,
    position: u32,
) -> Option<CompletionInfo> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;
    let node = ctx.arena.get(node_id)?;

    // Determine the completion context
    let completion_kind = get_completion_kind(ctx.arena, node_id, position);

    match completion_kind {
        CompletionKind::Member => get_member_completions(ctx, node_id),
        CompletionKind::String => get_string_completions(ctx, node_id),
        CompletionKind::Keyword => get_keyword_completions(ctx, node_id),
        CompletionKind::Global => get_global_completions(ctx, node_id),
        CompletionKind::Import => get_import_completions(ctx, node_id),
        CompletionKind::Type => get_type_completions(ctx, node_id),
        CompletionKind::None => None,
    }
}

/// Get completion entry details.
pub fn get_completion_entry_details(
    ctx: &CompletionsContext,
    position: u32,
    entry_name: &str,
    source: Option<&str>,
) -> Option<CompletionEntryDetails> {
    // Find the node at the position
    let node_id = find_node_at_position(ctx.arena, position)?;

    // Look up the symbol by name
    let symbol = find_symbol_by_name(ctx, node_id, entry_name)?;

    // Build completion details
    let kind = super::utilities::get_symbol_kind(&symbol);
    let mut details = CompletionEntryDetails::new(entry_name.to_string(), kind);

    // Add display parts
    details.display_parts = super::symbol_display::get_symbol_display_parts(&symbol, true);

    // Add documentation
    details.documentation = super::symbol_display::get_symbol_documentation(&symbol);

    // Add JSDoc tags
    for tag in super::symbol_display::get_symbol_jsdoc_tags(&symbol) {
        details.tags.push(JSDocTagInfo {
            name: tag.name,
            text: tag.text,
        });
    }

    // If this is an auto-import, add the code action
    if let Some(source_module) = source {
        let import_action = create_import_action(ctx, entry_name, source_module);
        details.code_actions.push(import_action);

        details.source_display = Some(vec![
            SymbolDisplayPart::text(source_module),
        ]);
    }

    Some(details)
}

// =============================================================================
// Completion Kinds
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompletionKind {
    Member,
    String,
    Keyword,
    Global,
    Import,
    Type,
    None,
}

fn get_completion_kind(
    arena: &NodeArena,
    node_id: NodeIndex,
    position: u32,
) -> CompletionKind {
    let node = match arena.get(node_id) {
        Some(n) => n,
        None => return CompletionKind::None,
    };

    // Check the node kind and context
    match node.kind() {
        SyntaxKind::PropertyAccessExpression => CompletionKind::Member,
        SyntaxKind::ElementAccessExpression => CompletionKind::Member,
        SyntaxKind::StringLiteral | SyntaxKind::NoSubstitutionTemplateLiteral => {
            CompletionKind::String
        }
        SyntaxKind::ImportDeclaration | SyntaxKind::ImportClause => {
            CompletionKind::Import
        }
        SyntaxKind::TypeReference | SyntaxKind::TypeQuery => {
            CompletionKind::Type
        }
        _ => {
            // Check if we're in a position that expects identifiers
            if is_identifier_position(arena, node_id) {
                CompletionKind::Global
            } else if is_keyword_position(arena, node_id) {
                CompletionKind::Keyword
            } else {
                CompletionKind::None
            }
        }
    }
}

// =============================================================================
// Completion Implementations
// =============================================================================

fn get_member_completions(
    ctx: &CompletionsContext,
    node_id: NodeIndex,
) -> Option<CompletionInfo> {
    let node = ctx.arena.get(node_id)?;

    // Get the expression being accessed
    let expression_id = get_expression_for_member_access(ctx.arena, node_id)?;

    // Get the type of the expression
    let type_id = ctx.checker.get_type_at_location(expression_id)?;

    // Get properties of the type
    let properties = ctx.checker.get_properties_of_type(type_id);

    let mut entries = Vec::new();

    for prop in properties {
        let kind = super::utilities::get_symbol_kind(&prop);
        let entry = CompletionEntry::new(prop.escaped_name.clone(), kind);
        entries.push(entry);
    }

    Some(CompletionInfo::member(entries))
}

fn get_string_completions(
    ctx: &CompletionsContext,
    node_id: NodeIndex,
) -> Option<CompletionInfo> {
    // Delegate to string_completions module
    super::string_completions::get_string_literal_completions(ctx, node_id)
}

fn get_keyword_completions(
    ctx: &CompletionsContext,
    node_id: NodeIndex,
) -> Option<CompletionInfo> {
    let mut entries = Vec::new();

    // Add applicable keywords based on context
    let keywords = get_applicable_keywords(ctx.arena, node_id);

    for keyword in keywords {
        let entry = CompletionEntry::new(keyword.to_string(), ScriptElementKind::Keyword)
            .with_sort_text("15".to_string()); // Keywords sort after other completions
        entries.push(entry);
    }

    if entries.is_empty() {
        None
    } else {
        Some(CompletionInfo::new(entries))
    }
}

fn get_global_completions(
    ctx: &CompletionsContext,
    node_id: NodeIndex,
) -> Option<CompletionInfo> {
    let mut entries = Vec::new();
    let mut seen_names = HashSet::new();

    // Get symbols from the current scope
    if let Some(scope_symbols) = get_symbols_in_scope(ctx, node_id) {
        for symbol in scope_symbols {
            if seen_names.insert(symbol.escaped_name.clone()) {
                let kind = super::utilities::get_symbol_kind(&symbol);
                let entry = CompletionEntry::new(symbol.escaped_name.clone(), kind);
                entries.push(entry);
            }
        }
    }

    // Add auto-import completions if enabled
    if ctx.options.include_completions_for_module_exports.unwrap_or(true) {
        if let Some(export_map) = ctx.export_map {
            for name in export_map.get_all_names() {
                if seen_names.insert(name.clone()) {
                    if let Some(exports) = export_map.get_exports_for_name(name) {
                        if let Some(export_info) = exports.first() {
                            let kind = super::utilities::get_kind_from_flags(export_info.target_flags);
                            let entry = CompletionEntry::new(name.clone(), kind)
                                .with_source(export_info.module_file_name.clone().unwrap_or_default());
                            entries.push(entry);
                        }
                    }
                }
            }
        }
    }

    // Add keywords
    let keywords = get_applicable_keywords(ctx.arena, node_id);
    for keyword in keywords {
        let entry = CompletionEntry::new(keyword.to_string(), ScriptElementKind::Keyword)
            .with_sort_text("15".to_string());
        entries.push(entry);
    }

    if entries.is_empty() {
        None
    } else {
        Some(CompletionInfo::global(entries))
    }
}

fn get_import_completions(
    ctx: &CompletionsContext,
    node_id: NodeIndex,
) -> Option<CompletionInfo> {
    let mut entries = Vec::new();

    // Get available modules
    if let Some(export_map) = ctx.export_map {
        let mut seen_modules = HashSet::new();

        export_map.for_each(|name, export| {
            if let Some(module_name) = &export.module_file_name {
                if seen_modules.insert(module_name.clone()) {
                    let entry = CompletionEntry::new(
                        module_name.clone(),
                        ScriptElementKind::ModuleElement,
                    );
                    entries.push(entry);
                }
            }
        });
    }

    if entries.is_empty() {
        None
    } else {
        Some(CompletionInfo::new(entries))
    }
}

fn get_type_completions(
    ctx: &CompletionsContext,
    node_id: NodeIndex,
) -> Option<CompletionInfo> {
    let mut entries = Vec::new();

    // Get type symbols from scope
    if let Some(symbols) = get_type_symbols_in_scope(ctx, node_id) {
        for symbol in symbols {
            let kind = super::utilities::get_symbol_kind(&symbol);
            let entry = CompletionEntry::new(symbol.escaped_name.clone(), kind);
            entries.push(entry);
        }
    }

    // Add built-in types
    let builtin_types = [
        "string", "number", "boolean", "object", "symbol", "bigint",
        "undefined", "null", "void", "never", "unknown", "any",
    ];

    for type_name in builtin_types {
        let entry = CompletionEntry::new(
            type_name.to_string(),
            ScriptElementKind::PrimitiveType,
        );
        entries.push(entry);
    }

    if entries.is_empty() {
        None
    } else {
        Some(CompletionInfo::new(entries))
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn find_node_at_position(arena: &NodeArena, position: u32) -> Option<NodeIndex> {
    let root = arena.root()?;
    find_deepest_node_at_position(arena, root, position)
}

fn find_deepest_node_at_position(
    arena: &NodeArena,
    node_id: NodeIndex,
    position: u32,
) -> Option<NodeIndex> {
    let node = arena.get(node_id)?;

    if position < node.pos() || position >= node.end() {
        return None;
    }

    for child_id in node.children() {
        if let Some(deeper) = find_deepest_node_at_position(arena, child_id, position) {
            return Some(deeper);
        }
    }

    Some(node_id)
}

fn is_identifier_position(arena: &NodeArena, node_id: NodeIndex) -> bool {
    let node = match arena.get(node_id) {
        Some(n) => n,
        None => return false,
    };

    matches!(
        node.kind(),
        SyntaxKind::Identifier | SyntaxKind::SourceFile
    )
}

fn is_keyword_position(arena: &NodeArena, node_id: NodeIndex) -> bool {
    // Check if we're at a position where keywords are expected
    false // Simplified
}

fn get_expression_for_member_access(
    arena: &NodeArena,
    node_id: NodeIndex,
) -> Option<NodeIndex> {
    // In a full implementation, this would navigate to the expression part
    // of a property access or element access
    None
}

fn get_symbols_in_scope(
    ctx: &CompletionsContext,
    node_id: NodeIndex,
) -> Option<Vec<Symbol>> {
    // In a full implementation, this would get all symbols visible
    // from the current scope
    ctx.checker.get_symbols_in_scope(node_id)
}

fn get_type_symbols_in_scope(
    ctx: &CompletionsContext,
    node_id: NodeIndex,
) -> Option<Vec<Symbol>> {
    // In a full implementation, this would filter to only type symbols
    get_symbols_in_scope(ctx, node_id).map(|symbols| {
        symbols
            .into_iter()
            .filter(|s| {
                s.flags.contains(SymbolFlags::Type)
                    || s.flags.contains(SymbolFlags::Interface)
                    || s.flags.contains(SymbolFlags::Class)
                    || s.flags.contains(SymbolFlags::TypeAlias)
            })
            .collect()
    })
}

fn find_symbol_by_name(
    ctx: &CompletionsContext,
    node_id: NodeIndex,
    name: &str,
) -> Option<Symbol> {
    let symbols = get_symbols_in_scope(ctx, node_id)?;
    symbols.into_iter().find(|s| s.escaped_name == name)
}

fn get_applicable_keywords(
    arena: &NodeArena,
    node_id: NodeIndex,
) -> Vec<&'static str> {
    // Return keywords based on context
    // This is a simplified version - full implementation would analyze context
    vec![
        "const", "let", "var", "function", "class", "interface", "type",
        "if", "else", "for", "while", "do", "switch", "case", "default",
        "return", "throw", "try", "catch", "finally", "break", "continue",
        "import", "export", "from", "as", "async", "await", "yield",
        "new", "this", "super", "typeof", "instanceof", "in", "delete",
        "void", "null", "true", "false",
    ]
}

fn create_import_action(
    ctx: &CompletionsContext,
    name: &str,
    source_module: &str,
) -> CodeAction {
    // Create the import statement to add
    let import_text = format!("import {{ {} }} from \"{}\";\n", name, source_module);

    CodeAction {
        description: format!("Add import from \"{}\"", source_module),
        changes: vec![FileTextChanges {
            file_name: ctx.file_name.to_string(),
            text_changes: vec![TextChange {
                span: TextSpan::new(0, 0), // Insert at beginning
                new_text: import_text,
            }],
            is_new_file: None,
        }],
    }
}

/// Get trigger characters for completions.
pub fn get_completion_trigger_characters() -> &'static [char] {
    &['.', '"', '\'', '`', '/', '@', '<', '#', ' ']
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_entry() {
        let entry = CompletionEntry::new(
            "foo".to_string(),
            ScriptElementKind::FunctionElement,
        )
        .with_sort_text("10".to_string());

        assert_eq!(entry.name, "foo");
        assert_eq!(entry.kind, ScriptElementKind::FunctionElement);
        assert_eq!(entry.sort_text, "10");
    }

    #[test]
    fn test_completion_entry_with_snippet() {
        let entry = CompletionEntry::new(
            "forEach".to_string(),
            ScriptElementKind::MemberFunctionElement,
        )
        .with_snippet("forEach((${1:element}) => {\n\t$0\n})");

        assert_eq!(entry.insert_text, Some("forEach((${1:element}) => {\n\t$0\n})".to_string()));
        assert_eq!(entry.is_snippet, Some(true));
    }

    #[test]
    fn test_completion_info() {
        let entries = vec![
            CompletionEntry::new("foo".to_string(), ScriptElementKind::VariableElement),
            CompletionEntry::new("bar".to_string(), ScriptElementKind::FunctionElement),
        ];

        let info = CompletionInfo::global(entries);

        assert!(info.is_global_completion);
        assert!(!info.is_member_completion);
        assert_eq!(info.entries.len(), 2);
    }

    #[test]
    fn test_trigger_characters() {
        let triggers = get_completion_trigger_characters();
        assert!(triggers.contains(&'.'));
        assert!(triggers.contains(&'"'));
        assert!(triggers.contains(&'/'));
    }
}
