//! Service utilities for language service operations.
//!
//! This module provides shared helper functions used across all language service features:
//! - AST navigation (finding tokens, containers, etc.)
//! - Symbol and type queries
//! - Text and name resolution
//! - Position utilities

use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;
use crate::scanner::SyntaxKind;
use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::{Type, TypeId, CheckerState};

use super::text_span::{TextSpan, TextRange};

// =============================================================================
// Script Element Kinds (for IDE display)
// =============================================================================

/// The kind of a program element for display purposes.
/// Matches TypeScript's `ScriptElementKind` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptElementKind {
    Unknown,
    Warning,
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

impl ScriptElementKind {
    /// Get the string representation matching TypeScript.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unknown => "",
            Self::Warning => "warning",
            Self::Keyword => "keyword",
            Self::ScriptElement => "script",
            Self::ModuleElement => "module",
            Self::ClassElement => "class",
            Self::LocalClassElement => "local class",
            Self::InterfaceElement => "interface",
            Self::TypeElement => "type",
            Self::EnumElement => "enum",
            Self::EnumMemberElement => "enum member",
            Self::VariableElement => "var",
            Self::LocalVariableElement => "local var",
            Self::FunctionElement => "function",
            Self::LocalFunctionElement => "local function",
            Self::MemberFunctionElement => "method",
            Self::MemberGetAccessorElement => "getter",
            Self::MemberSetAccessorElement => "setter",
            Self::MemberVariableElement => "property",
            Self::ConstructorImplementationElement => "constructor",
            Self::CallSignatureElement => "call",
            Self::IndexSignatureElement => "index",
            Self::ConstructSignatureElement => "construct",
            Self::ParameterElement => "parameter",
            Self::TypeParameterElement => "type parameter",
            Self::PrimitiveType => "primitive type",
            Self::Label => "label",
            Self::Alias => "alias",
            Self::ConstElement => "const",
            Self::LetElement => "let",
            Self::Directory => "directory",
            Self::ExternalModuleName => "external module name",
            Self::JsxAttribute => "JSX attribute",
            Self::String => "string",
            Self::Link => "link",
            Self::LinkName => "link name",
            Self::LinkText => "link text",
        }
    }
}

// =============================================================================
// Script Element Kind Modifiers
// =============================================================================

/// Modifiers for script elements.
/// Matches TypeScript's `ScriptElementKindModifier` enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ScriptElementKindModifier {
    bits: u32,
}

impl ScriptElementKindModifier {
    pub const NONE: u32 = 0;
    pub const PUBLIC: u32 = 1 << 0;
    pub const PRIVATE: u32 = 1 << 1;
    pub const PROTECTED: u32 = 1 << 2;
    pub const EXPORT: u32 = 1 << 3;
    pub const DECLARE: u32 = 1 << 4;
    pub const STATIC: u32 = 1 << 5;
    pub const ABSTRACT: u32 = 1 << 6;
    pub const OPTIONAL: u32 = 1 << 7;
    pub const DEPRECATED: u32 = 1 << 8;
    pub const DTS: u32 = 1 << 9;
    pub const TS: u32 = 1 << 10;
    pub const TSX: u32 = 1 << 11;
    pub const JS: u32 = 1 << 12;
    pub const JSX: u32 = 1 << 13;
    pub const JSON: u32 = 1 << 14;
    pub const DMTS: u32 = 1 << 15;
    pub const MTS: u32 = 1 << 16;
    pub const MJS: u32 = 1 << 17;
    pub const DCTS: u32 = 1 << 18;
    pub const CTS: u32 = 1 << 19;
    pub const CJS: u32 = 1 << 20;

    pub fn new(bits: u32) -> Self {
        Self { bits }
    }

    pub fn is_empty(&self) -> bool {
        self.bits == 0
    }

    pub fn contains(&self, flag: u32) -> bool {
        (self.bits & flag) != 0
    }

    pub fn insert(&mut self, flag: u32) {
        self.bits |= flag;
    }

    /// Convert to a comma-separated string of modifiers.
    pub fn to_string(&self) -> String {
        let mut parts = Vec::new();

        if self.contains(Self::PUBLIC) {
            parts.push("public");
        }
        if self.contains(Self::PRIVATE) {
            parts.push("private");
        }
        if self.contains(Self::PROTECTED) {
            parts.push("protected");
        }
        if self.contains(Self::EXPORT) {
            parts.push("export");
        }
        if self.contains(Self::DECLARE) {
            parts.push("declare");
        }
        if self.contains(Self::STATIC) {
            parts.push("static");
        }
        if self.contains(Self::ABSTRACT) {
            parts.push("abstract");
        }
        if self.contains(Self::OPTIONAL) {
            parts.push("optional");
        }
        if self.contains(Self::DEPRECATED) {
            parts.push("deprecated");
        }

        parts.join(",")
    }
}

// =============================================================================
// Highlight Span Kind
// =============================================================================

/// The kind of highlight for document highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlightSpanKind {
    None,
    Definition,
    Reference,
    WrittenReference,
}

// =============================================================================
// AST Navigation Utilities
// =============================================================================

/// Check if a syntax kind is a token (not a node).
#[inline]
pub fn is_token(kind: SyntaxKind) -> bool {
    kind <= SyntaxKind::LastToken
}

/// Check if a syntax kind is a keyword.
#[inline]
pub fn is_keyword(kind: SyntaxKind) -> bool {
    kind >= SyntaxKind::FirstKeyword && kind <= SyntaxKind::LastKeyword
}

/// Check if a syntax kind is a punctuation token.
#[inline]
pub fn is_punctuation(kind: SyntaxKind) -> bool {
    kind >= SyntaxKind::FirstPunctuation && kind <= SyntaxKind::LastPunctuation
}

/// Check if a syntax kind is a trivia token.
#[inline]
pub fn is_trivia(kind: SyntaxKind) -> bool {
    kind >= SyntaxKind::FirstTriviaToken && kind <= SyntaxKind::LastTriviaToken
}

/// Check if a syntax kind is a literal.
#[inline]
pub fn is_literal_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::NumericLiteral
            | SyntaxKind::BigIntLiteral
            | SyntaxKind::StringLiteral
            | SyntaxKind::JsxText
            | SyntaxKind::JsxTextAllWhiteSpaces
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::NoSubstitutionTemplateLiteral
    )
}

/// Check if a kind represents an identifier or keyword that can be used as an identifier.
#[inline]
pub fn is_identifier_or_keyword(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::Identifier || is_keyword(kind)
}

/// Check if a node represents a declaration.
pub fn is_declaration_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::VariableDeclaration
            | SyntaxKind::Parameter
            | SyntaxKind::BindingElement
            | SyntaxKind::PropertyDeclaration
            | SyntaxKind::PropertySignature
            | SyntaxKind::PropertyAssignment
            | SyntaxKind::ShorthandPropertyAssignment
            | SyntaxKind::SpreadAssignment
            | SyntaxKind::EnumMember
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::GetAccessor
            | SyntaxKind::SetAccessor
            | SyntaxKind::Constructor
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::InterfaceDeclaration
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::EnumDeclaration
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::ImportEqualsDeclaration
            | SyntaxKind::ImportClause
            | SyntaxKind::NamespaceImport
            | SyntaxKind::ImportSpecifier
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::TypeParameter
            | SyntaxKind::NamespaceExportDeclaration
    )
}

/// Check if a node represents a statement.
pub fn is_statement_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::BreakStatement
            | SyntaxKind::ContinueStatement
            | SyntaxKind::DebuggerStatement
            | SyntaxKind::DoStatement
            | SyntaxKind::ExpressionStatement
            | SyntaxKind::EmptyStatement
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::IfStatement
            | SyntaxKind::LabeledStatement
            | SyntaxKind::ReturnStatement
            | SyntaxKind::SwitchStatement
            | SyntaxKind::ThrowStatement
            | SyntaxKind::TryStatement
            | SyntaxKind::VariableStatement
            | SyntaxKind::WhileStatement
            | SyntaxKind::WithStatement
    )
}

/// Check if a node represents an expression.
pub fn is_expression_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ArrayLiteralExpression
            | SyntaxKind::ArrowFunction
            | SyntaxKind::AsExpression
            | SyntaxKind::AwaitExpression
            | SyntaxKind::BinaryExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::ClassExpression
            | SyntaxKind::CommaListExpression
            | SyntaxKind::ConditionalExpression
            | SyntaxKind::DeleteExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::FalseKeyword
            | SyntaxKind::FunctionExpression
            | SyntaxKind::Identifier
            | SyntaxKind::JsxElement
            | SyntaxKind::JsxSelfClosingElement
            | SyntaxKind::JsxFragment
            | SyntaxKind::MetaProperty
            | SyntaxKind::NewExpression
            | SyntaxKind::NonNullExpression
            | SyntaxKind::NoSubstitutionTemplateLiteral
            | SyntaxKind::NullKeyword
            | SyntaxKind::NumericLiteral
            | SyntaxKind::ObjectLiteralExpression
            | SyntaxKind::OmittedExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::PostfixUnaryExpression
            | SyntaxKind::PrefixUnaryExpression
            | SyntaxKind::PropertyAccessExpression
            | SyntaxKind::RegularExpressionLiteral
            | SyntaxKind::SpreadElement
            | SyntaxKind::StringLiteral
            | SyntaxKind::SuperKeyword
            | SyntaxKind::TaggedTemplateExpression
            | SyntaxKind::TemplateExpression
            | SyntaxKind::ThisKeyword
            | SyntaxKind::TrueKeyword
            | SyntaxKind::TypeAssertionExpression
            | SyntaxKind::TypeOfExpression
            | SyntaxKind::VoidExpression
            | SyntaxKind::YieldExpression
    )
}

// =============================================================================
// TextSpan Creation Utilities
// =============================================================================

/// Create a TextSpan from start and length.
#[inline]
pub fn create_text_span(start: u32, length: u32) -> TextSpan {
    TextSpan::new(start, length)
}

/// Create a TextSpan from start and end bounds.
#[inline]
pub fn create_text_span_from_bounds(start: u32, end: u32) -> TextSpan {
    TextSpan::from_bounds(start, end)
}

/// Create a TextSpan from a TextRange.
#[inline]
pub fn create_text_span_from_range(range: &TextRange) -> TextSpan {
    TextSpan::from_bounds(range.pos, range.end)
}

/// Create a TextSpan from a node.
pub fn create_text_span_from_node(arena: &NodeArena, node_id: NodeId) -> Option<TextSpan> {
    let node = arena.get(node_id)?;
    Some(TextSpan::from_bounds(node.pos(), node.end()))
}

// =============================================================================
// Position Utilities
// =============================================================================

/// Skip trivia (whitespace and comments) from a position.
pub fn skip_trivia(text: &str, pos: usize, stop_after_line_break: bool) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut pos = pos;

    loop {
        if pos >= chars.len() {
            return pos;
        }

        let ch = chars[pos];

        match ch {
            // Whitespace
            ' ' | '\t' | '\u{000B}' | '\u{000C}' | '\u{00A0}' | '\u{FEFF}' | '\u{1680}'
            | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' => {
                pos += 1;
            }
            // Line breaks
            '\r' => {
                if stop_after_line_break {
                    return pos;
                }
                pos += 1;
                if pos < chars.len() && chars[pos] == '\n' {
                    pos += 1;
                }
            }
            '\n' | '\u{2028}' | '\u{2029}' => {
                if stop_after_line_break {
                    return pos;
                }
                pos += 1;
            }
            // Comments
            '/' => {
                if pos + 1 < chars.len() {
                    match chars[pos + 1] {
                        '/' => {
                            // Single-line comment
                            pos += 2;
                            while pos < chars.len() {
                                let c = chars[pos];
                                if c == '\r' || c == '\n' || c == '\u{2028}' || c == '\u{2029}' {
                                    break;
                                }
                                pos += 1;
                            }
                        }
                        '*' => {
                            // Multi-line comment
                            pos += 2;
                            while pos + 1 < chars.len() {
                                if chars[pos] == '*' && chars[pos + 1] == '/' {
                                    pos += 2;
                                    break;
                                }
                                pos += 1;
                            }
                        }
                        _ => return pos,
                    }
                } else {
                    return pos;
                }
            }
            _ => return pos,
        }
    }
}

/// Get the start position of a token, skipping trivia.
pub fn get_token_pos_of_node(arena: &NodeArena, node_id: NodeId, source_text: &str) -> Option<u32> {
    let node = arena.get(node_id)?;
    let pos = node.pos() as usize;
    Some(skip_trivia(source_text, pos, false) as u32)
}

// =============================================================================
// Symbol Utilities
// =============================================================================

/// Get a unique ID for a symbol (for caching purposes).
pub fn get_symbol_id(symbol: &Symbol) -> u32 {
    // Use the symbol's declaration position as a stable ID
    symbol.declarations.first().map(|d| d.0).unwrap_or(0)
}

/// Get the name of a symbol.
pub fn get_symbol_name(symbol: &Symbol) -> &str {
    &symbol.escaped_name
}

/// Check if a symbol has a specific flag.
#[inline]
pub fn symbol_has_flag(symbol: &Symbol, flag: SymbolFlags) -> bool {
    symbol.flags.contains(flag)
}

/// Get the script element kind for a symbol.
pub fn get_symbol_kind(symbol: &Symbol) -> ScriptElementKind {
    let flags = symbol.flags;

    if flags.contains(SymbolFlags::Class) {
        ScriptElementKind::ClassElement
    } else if flags.contains(SymbolFlags::Interface) {
        ScriptElementKind::InterfaceElement
    } else if flags.contains(SymbolFlags::TypeAlias) {
        ScriptElementKind::TypeElement
    } else if flags.contains(SymbolFlags::Enum) {
        ScriptElementKind::EnumElement
    } else if flags.contains(SymbolFlags::EnumMember) {
        ScriptElementKind::EnumMemberElement
    } else if flags.contains(SymbolFlags::Function) {
        ScriptElementKind::FunctionElement
    } else if flags.contains(SymbolFlags::Method) {
        ScriptElementKind::MemberFunctionElement
    } else if flags.contains(SymbolFlags::GetAccessor) {
        ScriptElementKind::MemberGetAccessorElement
    } else if flags.contains(SymbolFlags::SetAccessor) {
        ScriptElementKind::MemberSetAccessorElement
    } else if flags.contains(SymbolFlags::Property) {
        ScriptElementKind::MemberVariableElement
    } else if flags.contains(SymbolFlags::Constructor) {
        ScriptElementKind::ConstructorImplementationElement
    } else if flags.contains(SymbolFlags::Module) || flags.contains(SymbolFlags::Namespace) {
        ScriptElementKind::ModuleElement
    } else if flags.contains(SymbolFlags::TypeParameter) {
        ScriptElementKind::TypeParameterElement
    } else if flags.contains(SymbolFlags::Alias) {
        ScriptElementKind::Alias
    } else if flags.contains(SymbolFlags::BlockScopedVariable) {
        if flags.contains(SymbolFlags::Const) {
            ScriptElementKind::ConstElement
        } else {
            ScriptElementKind::LetElement
        }
    } else if flags.contains(SymbolFlags::FunctionScopedVariable) {
        ScriptElementKind::VariableElement
    } else {
        ScriptElementKind::Unknown
    }
}

/// Get the modifiers for a symbol.
pub fn get_symbol_modifiers(symbol: &Symbol) -> ScriptElementKindModifier {
    let mut modifiers = ScriptElementKindModifier::new(0);
    let flags = symbol.flags;

    // Check visibility from symbol flags
    if flags.contains(SymbolFlags::Export) {
        modifiers.insert(ScriptElementKindModifier::EXPORT);
    }

    // Note: More detailed modifiers would require checking the declarations
    // This is a simplified implementation

    modifiers
}

// =============================================================================
// Node Finding Utilities
// =============================================================================

/// Find the token at or before a position in the source text.
/// This is a simplified version - the full implementation would use the AST.
pub fn find_token_at_position(text: &str, position: u32) -> Option<TextSpan> {
    let pos = position as usize;
    if pos >= text.len() {
        return None;
    }

    let chars: Vec<char> = text.chars().collect();

    // Find the start of the token
    let mut start = pos;
    while start > 0 {
        let ch = chars[start - 1];
        if !ch.is_alphanumeric() && ch != '_' && ch != '$' {
            break;
        }
        start -= 1;
    }

    // Find the end of the token
    let mut end = pos;
    while end < chars.len() {
        let ch = chars[end];
        if !ch.is_alphanumeric() && ch != '_' && ch != '$' {
            break;
        }
        end += 1;
    }

    if start == end {
        // No token found, return the character at position
        Some(TextSpan::new(pos as u32, 1))
    } else {
        Some(TextSpan::from_bounds(start as u32, end as u32))
    }
}

/// Get the text of a span from the source.
pub fn get_text_of_span(text: &str, span: &TextSpan) -> Option<&str> {
    let start = span.start as usize;
    let end = span.end() as usize;
    if end <= text.len() {
        Some(&text[start..end])
    } else {
        None
    }
}

// =============================================================================
// Diagnostic Utilities
// =============================================================================

/// Format a diagnostic message with arguments.
pub fn format_diagnostic_message(message: &str, args: &[&str]) -> String {
    let mut result = message.to_string();
    for (i, arg) in args.iter().enumerate() {
        result = result.replace(&format!("{{{}}}", i), arg);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_trivia() {
        assert_eq!(skip_trivia("  hello", 0, false), 2);
        assert_eq!(skip_trivia("// comment\nhello", 0, false), 11);
        assert_eq!(skip_trivia("/* comment */hello", 0, false), 13);
    }

    #[test]
    fn test_find_token_at_position() {
        let text = "let foo = bar";
        let span = find_token_at_position(text, 4).unwrap();
        assert_eq!(get_text_of_span(text, &span), Some("foo"));
    }

    #[test]
    fn test_format_diagnostic_message() {
        let msg = format_diagnostic_message("Cannot find name '{0}'.", &["foo"]);
        assert_eq!(msg, "Cannot find name 'foo'.");
    }
}
