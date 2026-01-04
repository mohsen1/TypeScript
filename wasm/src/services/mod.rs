//! Language Service module for TypeScript IDE features.
//!
//! This module provides IDE-like functionality including:
//! - Code completions
//! - Go to definition
//! - Find all references
//! - Rename
//! - Signature help
//! - Quick info (hover)
//! - Code fixes and refactorings

use serde::{Deserialize, Serialize};
use crate::checker::{CheckerState, TypeId, Type};
use crate::parser::{NodeIndex, Node};
use crate::binder::SymbolId;
use crate::scanner::SyntaxKind;

// =============================================================================
// Text Span Types
// =============================================================================

/// A span of text in a source file, represented as start position and length.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TextSpan {
    pub start: u32,
    pub length: u32,
}

impl TextSpan {
    pub fn new(start: u32, length: u32) -> Self {
        Self { start, length }
    }

    pub fn end(&self) -> u32 {
        self.start + self.length
    }

    pub fn from_bounds(start: u32, end: u32) -> Self {
        Self::new(start, end.saturating_sub(start))
    }

    pub fn contains(&self, position: u32) -> bool {
        position >= self.start && position < self.end()
    }
}

/// A range of text in a source file, represented as start and end positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct TextRange {
    pub pos: u32,
    pub end: u32,
}

impl TextRange {
    pub fn new(pos: u32, end: u32) -> Self {
        Self { pos, end }
    }
}

/// A text change with the span to replace and the new text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextChange {
    pub span: TextSpan,
    pub new_text: String,
}

impl TextChange {
    pub fn new(span: TextSpan, new_text: String) -> Self {
        Self { span, new_text }
    }
}

// =============================================================================
// Definition Info
// =============================================================================

/// Information about a definition location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionInfo {
    pub file_name: String,
    pub text_span: TextSpan,
    pub kind: ScriptElementKind,
    pub name: String,
    pub container_name: Option<String>,
}

/// Kind of script element for display purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptElementKind {
    Unknown,
    Keyword,
    ScriptElement,
    ModuleElement,
    ClassElement,
    LocalClassElement,
    InterfaceElement,
    TypeElement,
    EnumElement,
    EnumMemberElement,
    VariableElement,
    LocalVariableElement,
    FunctionElement,
    LocalFunctionElement,
    MemberFunctionElement,
    MemberGetAccessorElement,
    MemberSetAccessorElement,
    MemberVariableElement,
    ConstructorImplementationElement,
    CallSignatureElement,
    IndexSignatureElement,
    ConstructSignatureElement,
    ParameterElement,
    TypeParameterElement,
    PrimitiveType,
    Label,
    Alias,
    ConstElement,
    LetElement,
    Directory,
    ExternalModuleName,
    JsxAttribute,
    String,
    Link,
    LinkName,
    LinkText,
}

// =============================================================================
// Quick Info
// =============================================================================

/// Quick info (hover) response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickInfo {
    pub kind: ScriptElementKind,
    pub kind_modifiers: String,
    pub text_span: TextSpan,
    pub display_parts: Vec<SymbolDisplayPart>,
    pub documentation: Vec<SymbolDisplayPart>,
    pub tags: Vec<JSDocTagInfo>,
}

/// A part of a symbol display string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDisplayPart {
    pub text: String,
    pub kind: String,
}

impl SymbolDisplayPart {
    pub fn new(text: impl Into<String>, kind: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: kind.into(),
        }
    }

    pub fn text(s: impl Into<String>) -> Self {
        Self::new(s, "text")
    }

    pub fn keyword(s: impl Into<String>) -> Self {
        Self::new(s, "keyword")
    }

    pub fn punctuation(s: impl Into<String>) -> Self {
        Self::new(s, "punctuation")
    }

    pub fn space() -> Self {
        Self::new(" ", "space")
    }
}

/// JSDoc tag information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSDocTagInfo {
    pub name: String,
    pub text: Option<Vec<SymbolDisplayPart>>,
}

// =============================================================================
// Completions
// =============================================================================

/// A completion entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionEntry {
    pub name: String,
    pub kind: ScriptElementKind,
    pub kind_modifiers: String,
    pub sort_text: String,
    pub insert_text: Option<String>,
    pub replacement_span: Option<TextSpan>,
    pub has_action: bool,
    pub source: Option<String>,
    pub is_recommended: bool,
    pub is_from_unchecked_file: bool,
}

/// Completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionInfo {
    pub is_global_completion: bool,
    pub is_member_completion: bool,
    pub is_new_identifier_location: bool,
    pub entries: Vec<CompletionEntry>,
}

// =============================================================================
// Signature Help
// =============================================================================

/// Signature help response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureHelpItems {
    pub items: Vec<SignatureHelpItem>,
    pub applicable_span: TextSpan,
    pub selected_item_index: usize,
    pub argument_index: usize,
    pub argument_count: usize,
}

/// A signature help item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureHelpItem {
    pub is_variadic: bool,
    pub prefix_display_parts: Vec<SymbolDisplayPart>,
    pub suffix_display_parts: Vec<SymbolDisplayPart>,
    pub separator_display_parts: Vec<SymbolDisplayPart>,
    pub parameters: Vec<SignatureHelpParameter>,
    pub documentation: Vec<SymbolDisplayPart>,
    pub tags: Vec<JSDocTagInfo>,
}

/// A signature help parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureHelpParameter {
    pub name: String,
    pub documentation: Vec<SymbolDisplayPart>,
    pub display_parts: Vec<SymbolDisplayPart>,
    pub is_optional: bool,
    pub is_rest: bool,
}

// =============================================================================
// Rename
// =============================================================================

/// Rename info response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameInfo {
    pub can_rename: bool,
    pub display_name: String,
    pub full_display_name: String,
    pub kind: ScriptElementKind,
    pub kind_modifiers: String,
    pub trigger_span: TextSpan,
}

/// A rename location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameLocation {
    pub text_span: TextSpan,
    pub file_name: String,
    pub prefix_text: Option<String>,
    pub suffix_text: Option<String>,
}

// =============================================================================
// Document Highlights
// =============================================================================

/// Document highlight kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HighlightSpanKind {
    None,
    Definition,
    Reference,
    WrittenReference,
}

/// A highlight span.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSpan {
    pub text_span: TextSpan,
    pub kind: HighlightSpanKind,
}

/// Document highlights response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHighlights {
    pub file_name: String,
    pub highlight_spans: Vec<HighlightSpan>,
}

// =============================================================================
// Find All References
// =============================================================================

/// A reference entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceEntry {
    pub text_span: TextSpan,
    pub file_name: String,
    pub is_write_access: bool,
    pub is_definition: bool,
}

/// Referenced symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferencedSymbol {
    pub definition: DefinitionInfo,
    pub references: Vec<ReferenceEntry>,
}

// =============================================================================
// Navigation
// =============================================================================

/// Navigation bar item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationBarItem {
    pub text: String,
    pub kind: ScriptElementKind,
    pub kind_modifiers: String,
    pub spans: Vec<TextSpan>,
    pub child_items: Vec<NavigationBarItem>,
    pub indent: usize,
    pub bolded: bool,
    pub grayed: bool,
}

/// Navigation tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationTree {
    pub text: String,
    pub kind: ScriptElementKind,
    pub kind_modifiers: String,
    pub spans: Vec<TextSpan>,
    pub name_span: Option<TextSpan>,
    pub child_items: Vec<NavigationTree>,
}

// =============================================================================
// Code Fixes
// =============================================================================

/// A code action (fix or refactoring).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAction {
    pub description: String,
    pub changes: Vec<FileTextChanges>,
    pub commands: Vec<CodeActionCommand>,
}

/// File text changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTextChanges {
    pub file_name: String,
    pub text_changes: Vec<TextChange>,
    pub is_new_file: bool,
}

/// A command associated with a code action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeActionCommand {
    pub command: String,
    pub title: String,
    pub arguments: Vec<String>,
}

/// Code fix response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFixAction {
    pub fix_name: String,
    pub fix_id: Option<String>,
    pub fix_all_description: Option<String>,
    pub changes: Vec<FileTextChanges>,
    pub commands: Vec<CodeActionCommand>,
    pub description: String,
}

// =============================================================================
// Inlay Hints
// =============================================================================

/// Inlay hint kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InlayHintKind {
    Type,
    Parameter,
    Enum,
}

/// An inlay hint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlayHint {
    pub text: String,
    pub position: u32,
    pub kind: InlayHintKind,
    pub whitespace_before: bool,
    pub whitespace_after: bool,
}

// =============================================================================
// Call Hierarchy
// =============================================================================

/// Call hierarchy item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallHierarchyItem {
    pub name: String,
    pub kind: ScriptElementKind,
    pub kind_modifiers: String,
    pub file: String,
    pub span: TextSpan,
    pub selection_span: TextSpan,
}

/// Incoming call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallHierarchyIncomingCall {
    pub from: CallHierarchyItem,
    pub from_spans: Vec<TextSpan>,
}

/// Outgoing call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallHierarchyOutgoingCall {
    pub to: CallHierarchyItem,
    pub from_spans: Vec<TextSpan>,
}

// =============================================================================
// Outlining
// =============================================================================

/// Outlining span kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutliningSpanKind {
    Comment,
    Region,
    Code,
    Imports,
}

/// An outlining span for code folding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutliningSpan {
    pub text_span: TextSpan,
    pub hint_span: TextSpan,
    pub banner_text: String,
    pub auto_collapse: bool,
    pub kind: OutliningSpanKind,
}

// =============================================================================
// Formatting
// =============================================================================

/// Formatting options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatCodeSettings {
    pub indent_size: usize,
    pub tab_size: usize,
    pub new_line_character: String,
    pub convert_tabs_to_spaces: bool,
    pub indent_style: IndentStyle,
    pub insert_space_after_comma_delimiter: bool,
    pub insert_space_after_semicolon_in_for_statements: bool,
    pub insert_space_before_and_after_binary_operators: bool,
    pub insert_space_after_constructor: bool,
    pub insert_space_after_keywords_in_control_flow_statements: bool,
    pub insert_space_after_function_keyword_for_anonymous_functions: bool,
    pub insert_space_after_opening_and_before_closing_nonempty_parenthesis: bool,
    pub insert_space_after_opening_and_before_closing_nonempty_brackets: bool,
    pub insert_space_after_opening_and_before_closing_nonempty_braces: bool,
    pub insert_space_after_opening_and_before_closing_template_string_braces: bool,
    pub insert_space_after_opening_and_before_closing_jsx_expression_braces: bool,
    pub insert_space_after_type_assertion: bool,
    pub place_open_brace_on_new_line_for_functions: bool,
    pub place_open_brace_on_new_line_for_control_blocks: bool,
    pub semicolons: SemicolonPreference,
}

/// Indent style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndentStyle {
    None,
    Block,
    Smart,
}

/// Semicolon preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemicolonPreference {
    Ignore,
    Insert,
    Remove,
}

impl Default for FormatCodeSettings {
    fn default() -> Self {
        Self {
            indent_size: 4,
            tab_size: 4,
            new_line_character: "\n".to_string(),
            convert_tabs_to_spaces: true,
            indent_style: IndentStyle::Smart,
            insert_space_after_comma_delimiter: true,
            insert_space_after_semicolon_in_for_statements: true,
            insert_space_before_and_after_binary_operators: true,
            insert_space_after_constructor: false,
            insert_space_after_keywords_in_control_flow_statements: true,
            insert_space_after_function_keyword_for_anonymous_functions: false,
            insert_space_after_opening_and_before_closing_nonempty_parenthesis: false,
            insert_space_after_opening_and_before_closing_nonempty_brackets: false,
            insert_space_after_opening_and_before_closing_nonempty_braces: true,
            insert_space_after_opening_and_before_closing_template_string_braces: false,
            insert_space_after_opening_and_before_closing_jsx_expression_braces: false,
            insert_space_after_type_assertion: false,
            place_open_brace_on_new_line_for_functions: false,
            place_open_brace_on_new_line_for_control_blocks: false,
            semicolons: SemicolonPreference::Ignore,
        }
    }
}

// =============================================================================
// Language Service Mode
// =============================================================================

/// Language service mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanguageServiceMode {
    /// Full semantic services
    Semantic,
    /// Partial - used during project loading
    PartialSemantic,
    /// Syntax only
    Syntactic,
}

// =============================================================================
// Language Service
// =============================================================================

/// The TypeScript Language Service implementation in Rust.
///
/// This provides IDE features like:
/// - Go to definition
/// - Hover/quick info
/// - Find all references
/// - Completions
/// - etc.
pub struct LanguageService<'a> {
    /// Reference to the type checker state
    checker: &'a CheckerState<'a>,
    /// The source file name
    file_name: String,
    /// The root node index (source file)
    root: NodeIndex,
}

impl<'a> LanguageService<'a> {
    /// Create a new language service for a checked file.
    pub fn new(checker: &'a CheckerState<'a>, file_name: String, root: NodeIndex) -> Self {
        Self {
            checker,
            file_name,
            root,
        }
    }

    /// Get definition at a position.
    /// Returns definition locations for the symbol at the given position.
    pub fn get_definition_at_position(&self, position: u32) -> Vec<DefinitionInfo> {
        // Find the node at this position
        let Some(node_idx) = self.checker.get_node_at_position(self.root, position) else {
            return Vec::new();
        };

        // Get the symbol at this node
        let Some(symbol_id) = self.checker.get_symbol_at_location(node_idx) else {
            return Vec::new();
        };

        // Get declarations for this symbol
        let declarations = self.checker.get_symbol_declarations(symbol_id);
        let symbol_name = self.checker.get_symbol_name(symbol_id).unwrap_or_default();
        let symbol_flags = self.checker.get_symbol_flags(symbol_id);

        declarations
            .into_iter()
            .filter_map(|decl_idx| {
                let (start, end) = self.checker.get_node_span(decl_idx)?;
                let kind = self.symbol_flags_to_script_element_kind(symbol_flags);
                Some(DefinitionInfo {
                    file_name: self.file_name.clone(),
                    text_span: TextSpan::from_bounds(start, end),
                    kind,
                    name: symbol_name.clone(),
                    container_name: None, // TODO: get containing scope name
                })
            })
            .collect()
    }

    /// Get quick info (hover) at a position.
    pub fn get_quick_info_at_position(&self, position: u32) -> Option<QuickInfo> {
        // Find the node at this position
        let node_idx = self.checker.get_node_at_position(self.root, position)?;
        let (start, end) = self.checker.get_node_span(node_idx)?;

        // Try to get the symbol
        let symbol_id = self.checker.get_symbol_at_location(node_idx);

        // Get the cached type if we have a symbol
        // Note: get_type_of_node requires &mut self, so we use cached types
        let type_id = symbol_id.and_then(|sid| self.checker.get_cached_type_of_symbol(sid));

        let mut display_parts = Vec::new();
        let mut kind = ScriptElementKind::Unknown;
        let kind_modifiers = String::new();

        if let Some(sid) = symbol_id {
            let symbol_name = self.checker.get_symbol_name(sid).unwrap_or_default();
            let flags = self.checker.get_symbol_flags(sid);
            kind = self.symbol_flags_to_script_element_kind(flags);

            // Build display parts
            display_parts.push(SymbolDisplayPart::keyword(self.get_kind_keyword(&kind)));
            display_parts.push(SymbolDisplayPart::space());
            display_parts.push(SymbolDisplayPart::text(symbol_name));

            // Add type annotation if we have a type
            if let Some(tid) = type_id {
                if !tid.is_none() {
                    display_parts.push(SymbolDisplayPart::punctuation(":"));
                    display_parts.push(SymbolDisplayPart::space());
                    display_parts.push(SymbolDisplayPart::text(self.type_to_string(tid)));
                }
            }
        } else {
            // Check if this is a keyword
            if let Some(node_kind) = self.checker.get_node_kind(node_idx) {
                if let Some(keyword_info) = self.get_keyword_quick_info(node_kind) {
                    return Some(QuickInfo {
                        kind: ScriptElementKind::Keyword,
                        kind_modifiers: String::new(),
                        text_span: TextSpan::from_bounds(start, end),
                        display_parts: vec![SymbolDisplayPart::keyword(keyword_info.0)],
                        documentation: vec![SymbolDisplayPart::text(keyword_info.1)],
                        tags: Vec::new(),
                    });
                }
            }
            return None;
        }

        Some(QuickInfo {
            kind,
            kind_modifiers,
            text_span: TextSpan::from_bounds(start, end),
            display_parts,
            documentation: Vec::new(), // TODO: extract JSDoc
            tags: Vec::new(),
        })
    }

    /// Get references to the symbol at the given position.
    pub fn get_references_at_position(&self, position: u32) -> Vec<ReferenceEntry> {
        // Find the node at this position
        let Some(node_idx) = self.checker.get_node_at_position(self.root, position) else {
            return Vec::new();
        };

        // Get the symbol at this node
        let Some(target_symbol) = self.checker.get_symbol_at_location(node_idx) else {
            return Vec::new();
        };

        // Find all references to this symbol
        self.find_all_references_in_file(target_symbol)
    }

    /// Find all references to a symbol in the current file.
    fn find_all_references_in_file(&self, target_symbol: SymbolId) -> Vec<ReferenceEntry> {
        let mut references = Vec::new();
        self.collect_references_recursive(self.root, target_symbol, &mut references);
        references
    }

    fn collect_references_recursive(
        &self,
        node_idx: NodeIndex,
        target_symbol: SymbolId,
        references: &mut Vec<ReferenceEntry>,
    ) {
        // Check if this node references the target symbol
        if let Some(symbol) = self.checker.get_symbol_at_location(node_idx) {
            if symbol == target_symbol {
                if let Some((start, end)) = self.checker.get_node_span(node_idx) {
                    // Check if this is a definition or just a reference
                    let declarations = self.checker.get_symbol_declarations(target_symbol);
                    let is_definition = declarations.contains(&node_idx);

                    // Check if this is a write access
                    let is_write = self.is_write_access(node_idx);

                    references.push(ReferenceEntry {
                        text_span: TextSpan::from_bounds(start, end),
                        file_name: self.file_name.clone(),
                        is_write_access: is_write,
                        is_definition,
                    });
                }
            }
        }

        // Recurse into children
        for child_idx in self.checker.get_node_children(node_idx) {
            self.collect_references_recursive(child_idx, target_symbol, references);
        }
    }

    /// Check if a node represents a write access to a variable.
    fn is_write_access(&self, node_idx: NodeIndex) -> bool {
        // This is a simplification - a full implementation would check the parent
        // to see if it's the LHS of an assignment
        let declarations = self.checker.get_symbol_declarations(
            self.checker.get_symbol_at_location(node_idx).unwrap_or(SymbolId(u32::MAX))
        );
        declarations.contains(&node_idx)
    }

    /// Get completions at a position.
    pub fn get_completions_at_position(&self, position: u32) -> Option<CompletionInfo> {
        // For now, provide all file-level symbols as completions
        let mut entries = Vec::new();

        for (name, symbol_id) in self.checker.get_file_symbols() {
            let flags = self.checker.get_symbol_flags(*symbol_id);
            let kind = self.symbol_flags_to_script_element_kind(flags);

            entries.push(CompletionEntry {
                name: name.clone(),
                kind,
                kind_modifiers: String::new(),
                sort_text: name.clone(),
                insert_text: None,
                replacement_span: None,
                has_action: false,
                source: None,
                is_recommended: false,
                is_from_unchecked_file: false,
            });
        }

        // Add TypeScript keywords
        for keyword in self.get_common_keywords() {
            entries.push(CompletionEntry {
                name: keyword.to_string(),
                kind: ScriptElementKind::Keyword,
                kind_modifiers: String::new(),
                sort_text: format!("~{}", keyword), // Sort keywords after symbols
                insert_text: None,
                replacement_span: None,
                has_action: false,
                source: None,
                is_recommended: false,
                is_from_unchecked_file: false,
            });
        }

        Some(CompletionInfo {
            is_global_completion: true,
            is_member_completion: false,
            is_new_identifier_location: true,
            entries,
        })
    }

    /// Get document highlights at a position.
    pub fn get_document_highlights(
        &self,
        position: u32,
        _files_to_search: &[String],
    ) -> Vec<DocumentHighlights> {
        // Find the node at this position
        let Some(node_idx) = self.checker.get_node_at_position(self.root, position) else {
            return Vec::new();
        };

        // Get the symbol at this node
        let Some(target_symbol) = self.checker.get_symbol_at_location(node_idx) else {
            return Vec::new();
        };

        // Find all references in this file
        let references = self.find_all_references_in_file(target_symbol);
        let declarations = self.checker.get_symbol_declarations(target_symbol);

        let highlight_spans: Vec<HighlightSpan> = references
            .into_iter()
            .map(|r| {
                let kind = if r.is_definition {
                    HighlightSpanKind::Definition
                } else if r.is_write_access {
                    HighlightSpanKind::WrittenReference
                } else {
                    HighlightSpanKind::Reference
                };
                HighlightSpan {
                    text_span: r.text_span,
                    kind,
                }
            })
            .collect();

        if highlight_spans.is_empty() {
            Vec::new()
        } else {
            vec![DocumentHighlights {
                file_name: self.file_name.clone(),
                highlight_spans,
            }]
        }
    }

    /// Get rename info at a position.
    pub fn get_rename_info(&self, position: u32) -> RenameInfo {
        // Find the node at this position
        let Some(node_idx) = self.checker.get_node_at_position(self.root, position) else {
            return RenameInfo {
                can_rename: false,
                display_name: String::new(),
                full_display_name: String::new(),
                kind: ScriptElementKind::Unknown,
                kind_modifiers: String::new(),
                trigger_span: TextSpan::default(),
            };
        };

        // Get the symbol at this node
        let Some(symbol_id) = self.checker.get_symbol_at_location(node_idx) else {
            return RenameInfo {
                can_rename: false,
                display_name: String::new(),
                full_display_name: String::new(),
                kind: ScriptElementKind::Unknown,
                kind_modifiers: String::new(),
                trigger_span: TextSpan::default(),
            };
        };

        let symbol_name = self.checker.get_symbol_name(symbol_id).unwrap_or_default();
        let flags = self.checker.get_symbol_flags(symbol_id);
        let (start, end) = self.checker.get_node_span(node_idx).unwrap_or((0, 0));

        RenameInfo {
            can_rename: true,
            display_name: symbol_name.clone(),
            full_display_name: symbol_name,
            kind: self.symbol_flags_to_script_element_kind(flags),
            kind_modifiers: String::new(),
            trigger_span: TextSpan::from_bounds(start, end),
        }
    }

    /// Find rename locations at a position.
    pub fn find_rename_locations(&self, position: u32) -> Vec<RenameLocation> {
        self.get_references_at_position(position)
            .into_iter()
            .map(|r| RenameLocation {
                text_span: r.text_span,
                file_name: r.file_name,
                prefix_text: None,
                suffix_text: None,
            })
            .collect()
    }

    /// Get navigation bar items (outline) for the file.
    pub fn get_navigation_bar_items(&self) -> Vec<NavigationBarItem> {
        let mut items = Vec::new();
        self.collect_navigation_items(self.root, 0, &mut items);
        items
    }

    fn collect_navigation_items(
        &self,
        node_idx: NodeIndex,
        indent: usize,
        items: &mut Vec<NavigationBarItem>,
    ) {
        let Some(node) = self.checker.node_arena.get(node_idx) else {
            return;
        };

        // Check if this node should be in the navigation bar
        let (text, kind) = match node {
            Node::FunctionDeclaration(fd) => {
                let name = if !fd.name.is_none() {
                    self.get_node_text(fd.name)
                } else {
                    "<anonymous>".to_string()
                };
                (name, ScriptElementKind::FunctionElement)
            }
            Node::ClassDeclaration(cd) => {
                let name = if !cd.name.is_none() {
                    self.get_node_text(cd.name)
                } else {
                    "<anonymous>".to_string()
                };
                (name, ScriptElementKind::ClassElement)
            }
            Node::InterfaceDeclaration(id) => {
                let name = self.get_node_text(id.name);
                (name, ScriptElementKind::InterfaceElement)
            }
            Node::TypeAliasDeclaration(tad) => {
                let name = self.get_node_text(tad.name);
                (name, ScriptElementKind::TypeElement)
            }
            Node::EnumDeclaration(ed) => {
                let name = self.get_node_text(ed.name);
                (name, ScriptElementKind::EnumElement)
            }
            Node::VariableDeclaration(vd) => {
                let name = self.get_node_text(vd.name);
                (name, ScriptElementKind::VariableElement)
            }
            Node::MethodDeclaration(md) => {
                let name = self.get_node_text(md.name);
                (name, ScriptElementKind::MemberFunctionElement)
            }
            _ => {
                // Not a navigation item, but recurse into children
                for child_idx in self.checker.get_node_children(node_idx) {
                    self.collect_navigation_items(child_idx, indent, items);
                }
                return;
            }
        };

        let (start, end) = self.checker.get_node_span(node_idx).unwrap_or((0, 0));
        let mut child_items = Vec::new();

        // Collect children
        for child_idx in self.checker.get_node_children(node_idx) {
            self.collect_navigation_items(child_idx, indent + 1, &mut child_items);
        }

        items.push(NavigationBarItem {
            text,
            kind,
            kind_modifiers: String::new(),
            spans: vec![TextSpan::from_bounds(start, end)],
            child_items,
            indent,
            bolded: false,
            grayed: false,
        });
    }

    /// Get outlining spans for code folding.
    pub fn get_outlining_spans(&self) -> Vec<OutliningSpan> {
        let mut spans = Vec::new();
        self.collect_outlining_spans(self.root, &mut spans);
        spans
    }

    fn collect_outlining_spans(&self, node_idx: NodeIndex, spans: &mut Vec<OutliningSpan>) {
        let Some(node) = self.checker.node_arena.get(node_idx) else {
            return;
        };

        // Check if this node should be foldable
        match node {
            Node::FunctionDeclaration(_)
            | Node::ClassDeclaration(_)
            | Node::InterfaceDeclaration(_)
            | Node::Block(_)
            | Node::ObjectLiteralExpression(_)
            | Node::ArrayLiteralExpression(_) => {
                if let Some((start, end)) = self.checker.get_node_span(node_idx) {
                    // Only add if the span is meaningful (more than 1 line would be ideal)
                    if end > start + 10 {
                        spans.push(OutliningSpan {
                            text_span: TextSpan::from_bounds(start, end),
                            hint_span: TextSpan::from_bounds(start, start.saturating_add(20).min(end)),
                            banner_text: "...".to_string(),
                            auto_collapse: false,
                            kind: OutliningSpanKind::Code,
                        });
                    }
                }
            }
            _ => {}
        }

        // Recurse
        for child_idx in self.checker.get_node_children(node_idx) {
            self.collect_outlining_spans(child_idx, spans);
        }
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    /// Convert symbol flags to ScriptElementKind.
    fn symbol_flags_to_script_element_kind(&self, flags: u32) -> ScriptElementKind {
        use crate::binder::symbol_flags;

        if flags & symbol_flags::CLASS != 0 {
            ScriptElementKind::ClassElement
        } else if flags & symbol_flags::INTERFACE != 0 {
            ScriptElementKind::InterfaceElement
        } else if flags & symbol_flags::TYPE_ALIAS != 0 {
            ScriptElementKind::TypeElement
        } else if flags & symbol_flags::ENUM != 0 {
            ScriptElementKind::EnumElement
        } else if flags & symbol_flags::ENUM_MEMBER != 0 {
            ScriptElementKind::EnumMemberElement
        } else if flags & symbol_flags::FUNCTION != 0 {
            ScriptElementKind::FunctionElement
        } else if flags & symbol_flags::METHOD != 0 {
            ScriptElementKind::MemberFunctionElement
        } else if flags & symbol_flags::PROPERTY != 0 {
            ScriptElementKind::MemberVariableElement
        } else if flags & symbol_flags::VARIABLE != 0 {
            ScriptElementKind::VariableElement
        } else if flags & symbol_flags::TYPE_PARAMETER != 0 {
            ScriptElementKind::TypeParameterElement
        } else {
            ScriptElementKind::Unknown
        }
    }

    /// Get the keyword for display parts.
    fn get_kind_keyword(&self, kind: &ScriptElementKind) -> &'static str {
        match kind {
            ScriptElementKind::ClassElement => "class",
            ScriptElementKind::InterfaceElement => "interface",
            ScriptElementKind::TypeElement => "type",
            ScriptElementKind::EnumElement => "enum",
            ScriptElementKind::FunctionElement => "function",
            ScriptElementKind::MemberFunctionElement => "method",
            ScriptElementKind::MemberVariableElement => "property",
            ScriptElementKind::VariableElement => "var",
            ScriptElementKind::ConstElement => "const",
            ScriptElementKind::LetElement => "let",
            ScriptElementKind::ParameterElement => "parameter",
            ScriptElementKind::TypeParameterElement => "type parameter",
            _ => "",
        }
    }

    /// Convert a type to a display string.
    fn type_to_string(&self, type_id: TypeId) -> String {
        if type_id.is_none() {
            return "unknown".to_string();
        }

        let Some(ty) = self.checker.types.get(type_id) else {
            return "unknown".to_string();
        };

        match ty {
            Type::Intrinsic(intrinsic) => intrinsic.intrinsic_name.clone(),
            Type::Literal(lit) => {
                match &lit.value {
                    crate::checker::LiteralValue::String(s) => format!("\"{}\"", s),
                    crate::checker::LiteralValue::Number(n) => format!("{}", n),
                    crate::checker::LiteralValue::Boolean(b) => format!("{}", b),
                    crate::checker::LiteralValue::BigInt(s) => format!("{}n", s),
                }
            }
            Type::Union(u) => {
                let parts: Vec<String> = u.types.iter()
                    .map(|t| self.type_to_string(*t))
                    .collect();
                parts.join(" | ")
            }
            Type::Intersection(i) => {
                let parts: Vec<String> = i.types.iter()
                    .map(|t| self.type_to_string(*t))
                    .collect();
                parts.join(" & ")
            }
            Type::Object(obj) => {
                if obj.members.is_empty() {
                    "{}".to_string()
                } else {
                    // Object members is a SymbolTable, not a simple HashMap of types
                    // For now, just show the number of members
                    format!("{{ {} member(s) }}", obj.members.len())
                }
            }
            Type::Function(func) => {
                // FunctionType has parameter_types and parameter_names
                let params: Vec<String> = func.parameter_names.iter()
                    .zip(func.parameter_types.iter())
                    .map(|(name, type_id)| format!("{}: {}", name, self.type_to_string(*type_id)))
                    .collect();
                format!("({}) => {}", params.join(", "), self.type_to_string(func.return_type))
            }
            Type::Array(arr) => {
                format!("{}[]", self.type_to_string(arr.element_type))
            }
            Type::Tuple(tup) => {
                let elements: Vec<String> = tup.element_types.iter()
                    .map(|t| self.type_to_string(*t))
                    .collect();
                format!("[{}]", elements.join(", "))
            }
            Type::TypeParameter(tp) => {
                // TypeParameter uses symbol, not name directly
                // Get name from the symbol if possible
                if let Some(sym_name) = self.checker.get_symbol_name(tp.symbol) {
                    sym_name
                } else {
                    "T".to_string()
                }
            }
            Type::TypeReference(tr) => {
                // Get the symbol name for the target type
                let name = if let Some(sym_name) = self.checker.get_symbol_name(tr.symbol) {
                    sym_name
                } else {
                    // Fall back to the target type name
                    "Type".to_string()
                };

                if tr.type_arguments.is_empty() {
                    name
                } else {
                    let args: Vec<String> = tr.type_arguments.iter()
                        .map(|t| self.type_to_string(*t))
                        .collect();
                    format!("{}<{}>", name, args.join(", "))
                }
            }
            _ => "unknown".to_string(),
        }
    }

    /// Get the text of an identifier node.
    fn get_node_text(&self, node_idx: NodeIndex) -> String {
        if node_idx.is_none() {
            return String::new();
        }

        if let Some(Node::Identifier(id)) = self.checker.node_arena.get(node_idx) {
            return id.escaped_text.to_string();
        }

        // For other nodes, get the symbol name
        if let Some(symbol_id) = self.checker.get_symbol_at_location(node_idx) {
            return self.checker.get_symbol_name(symbol_id).unwrap_or_default();
        }

        String::new()
    }

    /// Get quick info for a keyword.
    fn get_keyword_quick_info(&self, kind: SyntaxKind) -> Option<(&'static str, &'static str)> {
        match kind {
            SyntaxKind::LetKeyword => Some(("let", "Declares a block-scoped variable.")),
            SyntaxKind::ConstKeyword => Some(("const", "Declares a block-scoped read-only constant.")),
            SyntaxKind::VarKeyword => Some(("var", "Declares a function-scoped or globally-scoped variable.")),
            SyntaxKind::FunctionKeyword => Some(("function", "Declares a function.")),
            SyntaxKind::ClassKeyword => Some(("class", "Declares a class.")),
            SyntaxKind::InterfaceKeyword => Some(("interface", "Declares an interface.")),
            SyntaxKind::TypeKeyword => Some(("type", "Declares a type alias.")),
            SyntaxKind::EnumKeyword => Some(("enum", "Declares an enumeration.")),
            SyntaxKind::ReturnKeyword => Some(("return", "Exits a function and optionally returns a value.")),
            SyntaxKind::IfKeyword => Some(("if", "Conditionally executes a statement.")),
            SyntaxKind::ElseKeyword => Some(("else", "Alternative branch for if statement.")),
            SyntaxKind::WhileKeyword => Some(("while", "Creates a loop that executes while condition is true.")),
            SyntaxKind::ForKeyword => Some(("for", "Creates a loop.")),
            SyntaxKind::BreakKeyword => Some(("break", "Terminates the current loop or switch statement.")),
            SyntaxKind::ContinueKeyword => Some(("continue", "Terminates the current loop iteration.")),
            SyntaxKind::ThrowKeyword => Some(("throw", "Throws an exception.")),
            SyntaxKind::TryKeyword => Some(("try", "Defines a block of code to try for errors.")),
            SyntaxKind::CatchKeyword => Some(("catch", "Handles errors from try block.")),
            SyntaxKind::FinallyKeyword => Some(("finally", "Executes after try/catch regardless of outcome.")),
            SyntaxKind::AsyncKeyword => Some(("async", "Declares an asynchronous function.")),
            SyntaxKind::AwaitKeyword => Some(("await", "Waits for a Promise to resolve.")),
            SyntaxKind::ExportKeyword => Some(("export", "Exports a declaration for use in other modules.")),
            SyntaxKind::ImportKeyword => Some(("import", "Imports declarations from other modules.")),
            SyntaxKind::NewKeyword => Some(("new", "Creates a new instance of a class.")),
            SyntaxKind::ThisKeyword => Some(("this", "References the current object.")),
            SyntaxKind::SuperKeyword => Some(("super", "References the parent class.")),
            SyntaxKind::ExtendsKeyword => Some(("extends", "Extends a class or type.")),
            SyntaxKind::ImplementsKeyword => Some(("implements", "Implements an interface.")),
            SyntaxKind::PrivateKeyword => Some(("private", "Restricts member access to the class.")),
            SyntaxKind::ProtectedKeyword => Some(("protected", "Restricts member access to class and subclasses.")),
            SyntaxKind::PublicKeyword => Some(("public", "Allows unrestricted member access.")),
            SyntaxKind::StaticKeyword => Some(("static", "Defines a static member.")),
            SyntaxKind::ReadonlyKeyword => Some(("readonly", "Marks a property as read-only.")),
            SyntaxKind::AbstractKeyword => Some(("abstract", "Defines an abstract class or member.")),
            _ => None,
        }
    }

    /// Get common keywords for completions.
    fn get_common_keywords(&self) -> &'static [&'static str] {
        &[
            "const", "let", "var", "function", "class", "interface", "type", "enum",
            "if", "else", "for", "while", "do", "switch", "case", "default",
            "return", "break", "continue", "throw", "try", "catch", "finally",
            "import", "export", "from", "as", "async", "await",
            "new", "this", "super", "extends", "implements",
            "public", "private", "protected", "static", "readonly", "abstract",
            "true", "false", "null", "undefined", "void", "never", "any", "unknown",
        ]
    }
}

// Backwards-compatible stateless LanguageService wrapper
pub struct StatelessLanguageService;

impl StatelessLanguageService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StatelessLanguageService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_span_basics() {
        let span = TextSpan::new(10, 5);
        assert_eq!(span.start, 10);
        assert_eq!(span.length, 5);
        assert_eq!(span.end(), 15);
        assert!(span.contains(10));
        assert!(span.contains(14));
        assert!(!span.contains(15));
    }

    #[test]
    fn test_text_span_from_bounds() {
        let span = TextSpan::from_bounds(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.length, 10);
    }

    #[test]
    fn test_stateless_language_service_create() {
        let _ls = StatelessLanguageService::new();
    }

    #[test]
    fn test_language_service_with_checker() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::checker::CheckerState;

        // Parse a simple TypeScript file
        let source = r#"
const x = 42;
function add(a: number, b: number): number {
    return a + b;
}
const y = add(x, 10);
"#;
        let mut parser = ParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        // Bind the file
        let mut binder = BinderState::new();
        binder.bind_source_file(&parser.arena, root);

        // Type check
        let mut checker = CheckerState::new(
            &parser.arena,
            &binder.symbols,
            &binder.file_locals,
            "test.ts".to_string(),
        );
        checker.check_source_file(root);

        // Create language service
        let ls = LanguageService::new(&checker, "test.ts".to_string(), root);

        // Test completions (should include our symbols)
        let completions = ls.get_completions_at_position(0).unwrap();
        assert!(!completions.entries.is_empty());

        // Test navigation bar (should find function and variables)
        let nav_items = ls.get_navigation_bar_items();
        assert!(!nav_items.is_empty());

        // Test outlining spans (should find the function block)
        let outlining = ls.get_outlining_spans();
        assert!(!outlining.is_empty());
    }
}
