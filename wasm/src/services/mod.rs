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
//!
//! NOTE: This is a minimal skeleton implementation. Full integration
//! with the checker requires additional work.

use serde::{Deserialize, Serialize};

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
// Language Service (Placeholder)
// =============================================================================

/// Placeholder for the full LanguageService implementation.
/// This will be wired up to the checker in future work.
pub struct LanguageService {
    // TODO: Add state for document registry, program, etc.
}

impl LanguageService {
    pub fn new() -> Self {
        Self {}
    }

    /// Get completions at a position.
    pub fn get_completions_at_position(
        &self,
        _file_name: &str,
        _position: u32,
    ) -> Option<CompletionInfo> {
        // TODO: Wire up to checker
        None
    }

    /// Get quick info (hover) at a position.
    pub fn get_quick_info_at_position(
        &self,
        _file_name: &str,
        _position: u32,
    ) -> Option<QuickInfo> {
        // TODO: Wire up to checker
        None
    }

    /// Get definition at a position.
    pub fn get_definition_at_position(
        &self,
        _file_name: &str,
        _position: u32,
    ) -> Vec<DefinitionInfo> {
        // TODO: Wire up to checker
        Vec::new()
    }

    /// Get references at a position.
    pub fn get_references_at_position(
        &self,
        _file_name: &str,
        _position: u32,
    ) -> Vec<ReferenceEntry> {
        // TODO: Wire up to checker
        Vec::new()
    }

    /// Get rename info at a position.
    pub fn get_rename_info(
        &self,
        _file_name: &str,
        _position: u32,
    ) -> RenameInfo {
        RenameInfo {
            can_rename: false,
            display_name: String::new(),
            full_display_name: String::new(),
            kind: ScriptElementKind::Unknown,
            kind_modifiers: String::new(),
            trigger_span: TextSpan::default(),
        }
    }

    /// Get signature help at a position.
    pub fn get_signature_help_items(
        &self,
        _file_name: &str,
        _position: u32,
    ) -> Option<SignatureHelpItems> {
        // TODO: Wire up to checker
        None
    }

    /// Get document highlights.
    pub fn get_document_highlights(
        &self,
        _file_name: &str,
        _position: u32,
        _files_to_search: &[String],
    ) -> Vec<DocumentHighlights> {
        // TODO: Wire up to checker
        Vec::new()
    }

    /// Get navigation bar items.
    pub fn get_navigation_bar_items(
        &self,
        _file_name: &str,
    ) -> Vec<NavigationBarItem> {
        // TODO: Wire up to checker
        Vec::new()
    }

    /// Get code fixes.
    pub fn get_code_fixes_at_position(
        &self,
        _file_name: &str,
        _start: u32,
        _end: u32,
        _error_codes: &[u32],
    ) -> Vec<CodeFixAction> {
        // TODO: Wire up to checker
        Vec::new()
    }

    /// Get formatting edits.
    pub fn get_formatting_edits_for_range(
        &self,
        _file_name: &str,
        _start: u32,
        _end: u32,
        _options: &FormatCodeSettings,
    ) -> Vec<TextChange> {
        // TODO: Implement formatting
        Vec::new()
    }

    /// Get inlay hints.
    pub fn provide_inlay_hints(
        &self,
        _file_name: &str,
        _span: TextSpan,
    ) -> Vec<InlayHint> {
        // TODO: Wire up to checker
        Vec::new()
    }

    /// Get outlining spans.
    pub fn get_outlining_spans(
        &self,
        _file_name: &str,
    ) -> Vec<OutliningSpan> {
        // TODO: Implement outlining
        Vec::new()
    }

    /// Get call hierarchy item.
    pub fn prepare_call_hierarchy(
        &self,
        _file_name: &str,
        _position: u32,
    ) -> Option<CallHierarchyItem> {
        // TODO: Wire up to checker
        None
    }
}

impl Default for LanguageService {
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
    fn test_language_service_create() {
        let _ls = LanguageService::new();
    }
}
