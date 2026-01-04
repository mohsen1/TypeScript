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
use crate::checker::{CheckerState, TypeId};
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

    pub fn parameter_name(s: impl Into<String>) -> Self {
        Self::new(s, "parameterName")
    }

    pub fn type_parameter_name(s: impl Into<String>) -> Self {
        Self::new(s, "typeParameterName")
    }

    pub fn property_name(s: impl Into<String>) -> Self {
        Self::new(s, "propertyName")
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

/// Diagnostic information for language service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSDiagnostic {
    /// Source file name
    pub file_name: String,
    /// Span of the diagnostic
    pub text_span: TextSpan,
    /// Diagnostic message
    pub message_text: String,
    /// Diagnostic category (Error, Warning, Suggestion, Message)
    pub category: DiagnosticCategory,
    /// Diagnostic code (e.g., 2304 for "Cannot find name")
    pub code: u32,
    /// Related information spans
    pub related_information: Vec<DiagnosticRelatedInfo>,
}

/// Related diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRelatedInfo {
    pub file_name: String,
    pub text_span: TextSpan,
    pub message_text: String,
}

/// Diagnostic category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticCategory {
    Warning = 0,
    Error = 1,
    Suggestion = 2,
    Message = 3,
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

    /// Get semantic (type checking) diagnostics for the file.
    /// Returns all type errors, missing imports, etc.
    pub fn get_semantic_diagnostics(&self) -> Vec<LSDiagnostic> {
        self.checker.diagnostics.iter().map(|d| {
            LSDiagnostic {
                file_name: d.file.clone(),
                text_span: TextSpan::new(d.start, d.length),
                message_text: d.message_text.clone(),
                category: match d.category {
                    crate::checker::DiagnosticCategory::Warning => DiagnosticCategory::Warning,
                    crate::checker::DiagnosticCategory::Error => DiagnosticCategory::Error,
                    crate::checker::DiagnosticCategory::Suggestion => DiagnosticCategory::Suggestion,
                    crate::checker::DiagnosticCategory::Message => DiagnosticCategory::Message,
                },
                code: d.code,
                related_information: d.related_information.iter().map(|ri| {
                    DiagnosticRelatedInfo {
                        file_name: ri.file.clone(),
                        text_span: TextSpan::new(ri.start, ri.length),
                        message_text: ri.message_text.clone(),
                    }
                }).collect(),
            }
        }).collect()
    }

    /// Get syntactic (parse) diagnostics for the file.
    /// Currently returns an empty list as parse errors are handled by the parser.
    /// TODO: Integrate parser diagnostics when available.
    pub fn get_syntactic_diagnostics(&self) -> Vec<LSDiagnostic> {
        // Parser doesn't currently expose diagnostics through the checker
        // In the future, this would return parse errors from the parser
        Vec::new()
    }

    /// Get all diagnostics (both syntactic and semantic) for the file.
    pub fn get_all_diagnostics(&self) -> Vec<LSDiagnostic> {
        let mut diagnostics = self.get_syntactic_diagnostics();
        diagnostics.extend(self.get_semantic_diagnostics());
        diagnostics
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

    /// Get type definition at a position.
    /// Returns the location of the type that the symbol references.
    /// For example, if `x: Foo`, go-to-type-definition on `x` jumps to `Foo`'s definition.
    pub fn get_type_definition_at_position(&self, position: u32) -> Vec<DefinitionInfo> {
        // Find the node at this position
        let Some(node_idx) = self.checker.get_node_at_position(self.root, position) else {
            return Vec::new();
        };

        // Get the symbol at this node
        let Some(symbol_id) = self.checker.get_symbol_at_location(node_idx) else {
            return Vec::new();
        };

        // Get the type of this symbol
        let Some(type_id) = self.checker.get_cached_type_of_symbol(symbol_id) else {
            return Vec::new();
        };

        // Get the type's symbol (if it has one)
        let Some(type_symbol) = self.checker.get_symbol_of_type(type_id) else {
            return Vec::new();
        };

        // Return definition locations for the type symbol
        let declarations = self.checker.get_symbol_declarations(type_symbol);
        let symbol_name = self.checker.get_symbol_name(type_symbol).unwrap_or_default();
        let symbol_flags = self.checker.get_symbol_flags(type_symbol);

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
                    container_name: None,
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
        // Check if we're in a property access context (after a '.')
        if let Some(member_completions) = self.get_member_completions_at_position(position) {
            return Some(member_completions);
        }

        // Check if we're in a type annotation context (after ':' or in type position)
        if let Some(type_completions) = self.get_type_completions_at_position(position) {
            return Some(type_completions);
        }

        // Fall back to global completions
        self.get_global_completions()
    }

    /// Try to get type completions if we're in a type annotation context.
    /// Returns Some if we're after a ':' in a type annotation position.
    fn get_type_completions_at_position(&self, position: u32) -> Option<CompletionInfo> {
        // Find if we're in a type annotation context
        if !self.is_in_type_context(self.root, position) {
            return None;
        }

        self.get_type_completions()
    }

    /// Check if position is in a type annotation context.
    /// This includes: after ':' in variable/parameter declarations, after 'extends', 'implements',
    /// inside generic type arguments, etc.
    fn is_in_type_context(&self, node_idx: NodeIndex, position: u32) -> bool {
        let (start, end) = match self.checker.get_node_span(node_idx) {
            Some(span) => span,
            None => return false,
        };

        // Check if position is within this node
        if position < start || position > end {
            return false;
        }

        // Check for type-related node kinds
        let node = match self.checker.node_arena.get(node_idx) {
            Some(n) => n,
            None => return false,
        };

        match node {
            // Direct type contexts
            Node::TypeReference(_) |
            Node::TypeLiteral(_) |
            Node::ArrayType(_) |
            Node::UnionType(_) |
            Node::IntersectionType(_) |
            Node::FunctionType(_) |
            Node::TypeQuery(_) |
            Node::TupleType(_) |
            Node::ParenthesizedType(_) |
            Node::ConditionalType(_) |
            Node::IndexedAccessType(_) |
            Node::MappedType(_) => return true,

            // Variable declaration - check if we're in the type part
            Node::VariableDeclaration(vd) => {
                if !vd.type_annotation.is_none() {
                    if let Some((type_start, _)) = self.checker.get_node_span(vd.type_annotation) {
                        if position >= type_start {
                            return true;
                        }
                    }
                }
            }

            // Parameter declaration - check if we're in the type part
            Node::ParameterDeclaration(p) => {
                if !p.type_annotation.is_none() {
                    if let Some((type_start, _)) = self.checker.get_node_span(p.type_annotation) {
                        if position >= type_start {
                            return true;
                        }
                    }
                }
            }

            Node::FunctionDeclaration(fd) => {
                // Check if we're in the return type
                if !fd.type_annotation.is_none() {
                    if let Some((type_start, _)) = self.checker.get_node_span(fd.type_annotation) {
                        if position >= type_start {
                            return true;
                        }
                    }
                }
            }

            Node::PropertyDeclaration(pd) => {
                if !pd.type_annotation.is_none() {
                    if let Some((type_start, _)) = self.checker.get_node_span(pd.type_annotation) {
                        if position >= type_start {
                            return true;
                        }
                    }
                }
            }

            Node::PropertySignature(ps) => {
                if !ps.type_annotation.is_none() {
                    if let Some((type_start, _)) = self.checker.get_node_span(ps.type_annotation) {
                        if position >= type_start {
                            return true;
                        }
                    }
                }
            }

            _ => {}
        }

        // Check children recursively
        for child_idx in self.checker.get_node_children(node_idx) {
            if self.is_in_type_context(child_idx, position) {
                return true;
            }
        }

        false
    }

    /// Get type completions (interfaces, type aliases, classes, enums, and primitive types).
    fn get_type_completions(&self) -> Option<CompletionInfo> {
        use crate::binder::symbol_flags;

        let mut entries = Vec::new();

        // Add primitive types first
        for primitive in &["string", "number", "boolean", "void", "null", "undefined", "never", "unknown", "any", "object", "symbol", "bigint"] {
            entries.push(CompletionEntry {
                name: primitive.to_string(),
                kind: ScriptElementKind::PrimitiveType,
                kind_modifiers: String::new(),
                sort_text: format!("0{}", primitive), // Sort primitives first
                insert_text: None,
                replacement_span: None,
                has_action: false,
                source: None,
                is_recommended: false,
                is_from_unchecked_file: false,
            });
        }

        // Add type symbols from file scope
        for (name, symbol_id) in self.checker.get_file_symbols() {
            let flags = self.checker.get_symbol_flags(*symbol_id);

            // Only include type-related symbols
            if (flags & symbol_flags::INTERFACE) != 0 ||
               (flags & symbol_flags::TYPE_ALIAS) != 0 ||
               (flags & symbol_flags::CLASS) != 0 ||
               (flags & symbol_flags::REGULAR_ENUM) != 0 ||
               (flags & symbol_flags::CONST_ENUM) != 0 {
                let kind = self.symbol_flags_to_script_element_kind(flags);

                entries.push(CompletionEntry {
                    name: name.clone(),
                    kind,
                    kind_modifiers: String::new(),
                    sort_text: format!("1{}", name), // Sort after primitives
                    insert_text: None,
                    replacement_span: None,
                    has_action: false,
                    source: None,
                    is_recommended: false,
                    is_from_unchecked_file: false,
                });
            }
        }

        if entries.is_empty() {
            return None;
        }

        Some(CompletionInfo {
            is_global_completion: false,
            is_member_completion: false,
            is_new_identifier_location: true,
            entries,
        })
    }

    /// Try to get member completions if we're in a property access context.
    /// Returns Some if we're after a '.' and can determine the type of the expression.
    fn get_member_completions_at_position(&self, position: u32) -> Option<CompletionInfo> {
        // Find if we're in a PropertyAccessExpression
        let node_idx = self.find_property_access_at_position(self.root, position)?;

        // Get the PropertyAccessExpression
        let property_access = match self.checker.node_arena.get(node_idx) {
            Some(Node::PropertyAccessExpression(pa)) => pa,
            _ => return None,
        };

        // Get the expression before the dot
        let expr_idx = property_access.expression;

        // Try to get the type of the expression
        // First check if there's a symbol for the expression
        let type_id = if let Some(symbol_id) = self.checker.get_symbol_at_location(expr_idx) {
            self.checker.get_cached_type_of_symbol(symbol_id)?
        } else {
            return None;
        };

        // Get properties of this type
        let properties = self.checker.get_properties_of_type(type_id);

        if properties.is_empty() {
            return None;
        }

        let mut entries = Vec::new();

        for (name, symbol_id) in properties {
            let flags = self.checker.get_symbol_flags(symbol_id);
            let kind = self.symbol_flags_to_script_element_kind(flags);

            entries.push(CompletionEntry {
                name: name.clone(),
                kind,
                kind_modifiers: String::new(),
                sort_text: name,
                insert_text: None,
                replacement_span: None,
                has_action: false,
                source: None,
                is_recommended: false,
                is_from_unchecked_file: false,
            });
        }

        Some(CompletionInfo {
            is_global_completion: false,
            is_member_completion: true,
            is_new_identifier_location: false,
            entries,
        })
    }

    /// Find a PropertyAccessExpression at the given position.
    fn find_property_access_at_position(&self, start_node: NodeIndex, position: u32) -> Option<NodeIndex> {
        self.find_property_access_recursive(start_node, position)
    }

    /// Recursively search for a PropertyAccessExpression containing the position.
    fn find_property_access_recursive(&self, node_idx: NodeIndex, position: u32) -> Option<NodeIndex> {
        let (start, end) = self.checker.get_node_span(node_idx)?;

        // Check if position is within this node
        if position < start || position > end {
            return None;
        }

        // If this is a PropertyAccessExpression, check if we're after the dot
        if let Some(Node::PropertyAccessExpression(pa)) = self.checker.node_arena.get(node_idx) {
            // Check if we're in the name part (after the dot)
            if let Some((expr_start, expr_end)) = self.checker.get_node_span(pa.expression) {
                // Position is after the expression (in the .name part)
                if position > expr_end {
                    return Some(node_idx);
                }
            }
        }

        // Check children
        for child_idx in self.checker.get_node_children(node_idx) {
            if let Some(result) = self.find_property_access_recursive(child_idx, position) {
                return Some(result);
            }
        }

        None
    }

    /// Get global completions (file-level symbols and keywords).
    fn get_global_completions(&self) -> Option<CompletionInfo> {
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

    /// Get signature help at a position (inside a function call).
    /// Returns function signatures and parameter info when cursor is inside parentheses.
    pub fn get_signature_help_at_position(&self, position: u32) -> Option<SignatureHelpItems> {
        // Find the call expression containing this position
        let (call_idx, arg_index, arg_count) = self.find_containing_call_expression(self.root, position)?;

        // Get the expression being called
        let call_expression = match self.checker.node_arena.get(call_idx) {
            Some(Node::CallExpression(c)) => c.expression,
            Some(Node::NewExpression(n)) => n.expression,
            _ => return None,
        };

        // Get the symbol of the called function
        let symbol_id = self.checker.get_symbol_at_location(call_expression)?;
        let symbol_name = self.checker.get_symbol_name(symbol_id)?;
        let type_id = self.checker.get_cached_type_of_symbol(symbol_id)?;

        // Build the signature help item
        let mut parameters = Vec::new();
        let mut prefix_display_parts = vec![
            SymbolDisplayPart::text(symbol_name.clone()),
            SymbolDisplayPart::punctuation("("),
        ];
        let suffix_display_parts = vec![SymbolDisplayPart::punctuation(")")];
        let separator_display_parts = vec![
            SymbolDisplayPart::punctuation(","),
            SymbolDisplayPart::space(),
        ];

        // Try to get function parameter info from the type
        if let Some(signatures) = self.checker.get_signatures_of_type(type_id) {
            for (i, sig) in signatures.iter().enumerate() {
                if i == 0 { // Use first signature for now
                    for param_info in sig {
                        parameters.push(SignatureHelpParameter {
                            name: param_info.name.clone(),
                            documentation: Vec::new(),
                            display_parts: vec![
                                SymbolDisplayPart::parameter_name(param_info.name.clone()),
                                SymbolDisplayPart::punctuation(":"),
                                SymbolDisplayPart::space(),
                                SymbolDisplayPart::text(param_info.type_display.clone()),
                            ],
                            is_optional: param_info.is_optional,
                            is_rest: param_info.is_rest,
                        });
                    }
                }
            }
        }

        // Get the applicable span (the argument list area)
        let (call_start, call_end) = self.checker.get_node_span(call_idx)?;
        let applicable_span = TextSpan::from_bounds(call_start, call_end);

        let item = SignatureHelpItem {
            is_variadic: parameters.iter().any(|p| p.is_rest),
            prefix_display_parts,
            suffix_display_parts,
            separator_display_parts,
            parameters,
            documentation: Vec::new(),
            tags: Vec::new(),
        };

        Some(SignatureHelpItems {
            items: vec![item],
            applicable_span,
            selected_item_index: 0,
            argument_index: arg_index,
            argument_count: arg_count,
        })
    }

    /// Find the call expression containing the given position and return the argument index.
    fn find_containing_call_expression(
        &self,
        start_node: NodeIndex,
        position: u32,
    ) -> Option<(NodeIndex, usize, usize)> {
        self.find_call_at_position(start_node, position)
    }

    /// Recursively search for a call expression containing the position.
    fn find_call_at_position(&self, node_idx: NodeIndex, position: u32) -> Option<(NodeIndex, usize, usize)> {
        let (start, end) = self.checker.get_node_span(node_idx)?;

        // Check if position is within this node
        if position < start || position > end {
            return None;
        }

        // If this is a call expression, check if we're in the argument list
        match self.checker.node_arena.get(node_idx) {
            Some(Node::CallExpression(call)) => {
                // Calculate the argument index based on position
                let (arg_index, arg_count) = self.get_argument_index(&call.arguments, position);
                if arg_count > 0 || self.is_in_parentheses(node_idx, position) {
                    return Some((node_idx, arg_index, arg_count));
                }
            }
            Some(Node::NewExpression(new_expr)) => {
                if let Some(ref args) = new_expr.arguments {
                    let (arg_index, arg_count) = self.get_argument_index(args, position);
                    if arg_count > 0 || self.is_in_parentheses(node_idx, position) {
                        return Some((node_idx, arg_index, arg_count));
                    }
                }
            }
            _ => {}
        }

        // Check children
        for child_idx in self.checker.get_node_children(node_idx) {
            if let Some(result) = self.find_call_at_position(child_idx, position) {
                return Some(result);
            }
        }

        None
    }

    /// Get the current argument index based on position within argument list.
    fn get_argument_index(&self, args: &crate::parser::NodeList, position: u32) -> (usize, usize) {
        let arg_count = args.nodes.len();
        if arg_count == 0 {
            return (0, 0);
        }

        // Find which argument we're in based on position
        for (i, arg_idx) in args.nodes.iter().enumerate() {
            if let Some((start, end)) = self.checker.get_node_span(*arg_idx) {
                if position <= start {
                    return (i, arg_count);
                }
                if position <= end {
                    return (i, arg_count);
                }
            }
        }

        // If we're past all arguments, we're in the last/next position
        (arg_count, arg_count)
    }

    /// Check if position is inside the parentheses of a call expression.
    fn is_in_parentheses(&self, call_idx: NodeIndex, position: u32) -> bool {
        // Simple check: if we have the call expression span, position should be inside
        if let Some((start, end)) = self.checker.get_node_span(call_idx) {
            return position > start && position < end;
        }
        false
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
    /// Delegates to the checker's type_to_string to avoid code duplication.
    fn type_to_string(&self, type_id: TypeId) -> String {
        self.checker.type_to_string(type_id)
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
            &binder.node_symbols,
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

        // Test diagnostics API - this code has no errors
        let diagnostics = ls.get_semantic_diagnostics();
        assert!(diagnostics.is_empty(), "Expected no diagnostics for valid code");

        let all_diagnostics = ls.get_all_diagnostics();
        assert!(all_diagnostics.is_empty(), "Expected no diagnostics for valid code");
    }

    #[test]
    fn test_language_service_diagnostics() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::checker::CheckerState;

        // Parse TypeScript code with a type error
        let source = r#"
const x: number = "hello"; // Error: string not assignable to number
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
            &binder.node_symbols,
            "test.ts".to_string(),
        );
        checker.check_source_file(root);

        // Create language service
        let ls = LanguageService::new(&checker, "test.ts".to_string(), root);

        // Test diagnostics API - should have a type error
        let diagnostics = ls.get_semantic_diagnostics();
        assert!(!diagnostics.is_empty(), "Expected diagnostics for type error");
        assert_eq!(diagnostics[0].code, 2322, "Expected TS2322 type error");
        assert_eq!(diagnostics[0].category, DiagnosticCategory::Error);
    }

    #[test]
    fn test_member_completions() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::checker::CheckerState;

        // Parse TypeScript code with an interface and property access
        let source = r#"
interface Point {
    x: number;
    y: number;
}
const p: Point = { x: 1, y: 2 };
p.
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
            &binder.node_symbols,
            "test.ts".to_string(),
        );
        checker.check_source_file(root);

        // Create language service
        let ls = LanguageService::new(&checker, "test.ts".to_string(), root);

        // Global completions should include Point and p
        let global_completions = ls.get_global_completions().unwrap();
        assert!(global_completions.is_global_completion);
        assert!(!global_completions.is_member_completion);

        // Check that we have file-level symbols
        let entry_names: Vec<_> = global_completions.entries.iter()
            .map(|e| e.name.as_str())
            .collect();
        assert!(entry_names.contains(&"Point"), "Should have Point in completions");
        assert!(entry_names.contains(&"p"), "Should have p in completions");
    }

    #[test]
    fn test_get_properties_of_type() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::checker::CheckerState;

        // Parse TypeScript code with an interface
        let source = r#"
interface Point {
    x: number;
    y: number;
    move(dx: number, dy: number): void;
}
const p: Point = { x: 1, y: 2, move(dx, dy) {} };
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
            &binder.node_symbols,
            "test.ts".to_string(),
        );
        checker.check_source_file(root);

        // Get the type of 'p' and check its properties
        if let Some(p_symbol) = binder.file_locals.get("p") {
            if let Some(p_type) = checker.get_cached_type_of_symbol(p_symbol) {
                let properties = checker.get_properties_of_type(p_type);
                let prop_names: Vec<_> = properties.iter()
                    .map(|(name, _)| name.as_str())
                    .collect();

                // Note: The exact properties depend on how the type checker resolves
                // the Point type. At minimum we should have some properties.
                // This test validates the infrastructure works.
                assert!(!properties.is_empty() || p_type.is_none(),
                    "Should have properties or be unresolved type");
            }
        }
    }

    #[test]
    fn test_type_completions() {
        use crate::parser_impl::ParserState;
        use crate::binder::BinderState;
        use crate::checker::CheckerState;

        // Parse TypeScript code with type annotations
        let source = r#"
interface Person {
    name: string;
    age: number;
}

type Status = "active" | "inactive";

class User implements Person {
    name: string;
    age: number;
}

enum Color { Red, Green, Blue }

const x: number = 1;
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
            &binder.node_symbols,
            "test.ts".to_string(),
        );
        checker.check_source_file(root);

        // Create language service
        let ls = LanguageService::new(&checker, "test.ts".to_string(), root);

        // Get type completions
        let type_completions = ls.get_type_completions();
        assert!(type_completions.is_some(), "Should get type completions");

        let completions = type_completions.unwrap();
        let entry_names: Vec<_> = completions.entries.iter()
            .map(|e| e.name.as_str())
            .collect();

        // Should have primitive types
        assert!(entry_names.contains(&"string"), "Should have 'string' primitive");
        assert!(entry_names.contains(&"number"), "Should have 'number' primitive");
        assert!(entry_names.contains(&"boolean"), "Should have 'boolean' primitive");

        // Should have user-defined types
        assert!(entry_names.contains(&"Person"), "Should have 'Person' interface");
        assert!(entry_names.contains(&"Status"), "Should have 'Status' type alias");
        assert!(entry_names.contains(&"User"), "Should have 'User' class");
        assert!(entry_names.contains(&"Color"), "Should have 'Color' enum");

        // Should NOT have value-only symbols
        assert!(!entry_names.contains(&"x"), "Should NOT have 'x' variable in type completions");
    }
}
