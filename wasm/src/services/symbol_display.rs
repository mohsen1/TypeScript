//! Symbol display utilities for IDE features.
//!
//! This module provides functionality for converting symbols and types
//! into display strings for hover, completions, and other IDE features.

use crate::binder::{Symbol, SymbolId, SymbolFlags};
use crate::checker::{Type, TypeId, CheckerState};
use crate::parser::ast::{Node, NodeId};
use crate::parser::arena::NodeArena;

use super::utilities::{ScriptElementKind, ScriptElementKindModifier};

// =============================================================================
// Symbol Display Part
// =============================================================================

/// A part of a symbol display string with its kind.
/// Matches TypeScript's `SymbolDisplayPart` interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolDisplayPart {
    pub text: String,
    pub kind: SymbolDisplayPartKind,
}

impl SymbolDisplayPart {
    pub fn new(text: String, kind: SymbolDisplayPartKind) -> Self {
        Self { text, kind }
    }

    pub fn text(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::Text)
    }

    pub fn keyword(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::Keyword)
    }

    pub fn punctuation(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::Punctuation)
    }

    pub fn space() -> Self {
        Self::new(" ".to_string(), SymbolDisplayPartKind::Space)
    }

    pub fn line_break() -> Self {
        Self::new("\n".to_string(), SymbolDisplayPartKind::LineBreak)
    }

    pub fn class_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::ClassName)
    }

    pub fn interface_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::InterfaceName)
    }

    pub fn type_parameter_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::TypeParameterName)
    }

    pub fn parameter_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::ParameterName)
    }

    pub fn property_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::PropertyName)
    }

    pub fn local_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::LocalName)
    }

    pub fn function_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::FunctionName)
    }

    pub fn method_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::MethodName)
    }

    pub fn module_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::ModuleName)
    }

    pub fn enum_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::EnumName)
    }

    pub fn enum_member_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::EnumMemberName)
    }

    pub fn alias_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::AliasName)
    }

    pub fn type_alias_name(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::TypeAliasName)
    }

    pub fn string_literal(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::StringLiteral)
    }

    pub fn number_literal(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::NumberLiteral)
    }

    pub fn operator(text: &str) -> Self {
        Self::new(text.to_string(), SymbolDisplayPartKind::Operator)
    }
}

/// The kind of a symbol display part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolDisplayPartKind {
    AliasName,
    ClassName,
    EnumName,
    EnumMemberName,
    FieldName,
    InterfaceName,
    Keyword,
    LineBreak,
    NumberLiteral,
    StringLiteral,
    LocalName,
    MethodName,
    ModuleName,
    Operator,
    ParameterName,
    PropertyName,
    Punctuation,
    Space,
    Text,
    TypeParameterName,
    FunctionName,
    TypeAliasName,
    Link,
    LinkName,
    LinkText,
}

impl SymbolDisplayPartKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AliasName => "aliasName",
            Self::ClassName => "className",
            Self::EnumName => "enumName",
            Self::EnumMemberName => "enumMemberName",
            Self::FieldName => "fieldName",
            Self::InterfaceName => "interfaceName",
            Self::Keyword => "keyword",
            Self::LineBreak => "lineBreak",
            Self::NumberLiteral => "numericLiteral",
            Self::StringLiteral => "stringLiteral",
            Self::LocalName => "localName",
            Self::MethodName => "methodName",
            Self::ModuleName => "moduleName",
            Self::Operator => "operator",
            Self::ParameterName => "parameterName",
            Self::PropertyName => "propertyName",
            Self::Punctuation => "punctuation",
            Self::Space => "space",
            Self::Text => "text",
            Self::TypeParameterName => "typeParameterName",
            Self::FunctionName => "functionName",
            Self::TypeAliasName => "typeAliasName",
            Self::Link => "link",
            Self::LinkName => "linkName",
            Self::LinkText => "linkText",
        }
    }
}

// =============================================================================
// JSDoc Tag Info
// =============================================================================

/// Information about a JSDoc tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSDocTagInfo {
    pub name: String,
    pub text: Option<Vec<SymbolDisplayPart>>,
}

impl JSDocTagInfo {
    pub fn new(name: String) -> Self {
        Self { name, text: None }
    }

    pub fn with_text(name: String, text: Vec<SymbolDisplayPart>) -> Self {
        Self { name, text: Some(text) }
    }
}

// =============================================================================
// Display Parts Builder
// =============================================================================

/// Builder for creating display parts.
#[derive(Debug, Default)]
pub struct DisplayPartsBuilder {
    parts: Vec<SymbolDisplayPart>,
}

impl DisplayPartsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, part: SymbolDisplayPart) -> &mut Self {
        self.parts.push(part);
        self
    }

    pub fn push_text(&mut self, text: &str) -> &mut Self {
        self.push(SymbolDisplayPart::text(text))
    }

    pub fn push_keyword(&mut self, keyword: &str) -> &mut Self {
        self.push(SymbolDisplayPart::keyword(keyword))
    }

    pub fn push_punctuation(&mut self, punct: &str) -> &mut Self {
        self.push(SymbolDisplayPart::punctuation(punct))
    }

    pub fn push_space(&mut self) -> &mut Self {
        self.push(SymbolDisplayPart::space())
    }

    pub fn push_line_break(&mut self) -> &mut Self {
        self.push(SymbolDisplayPart::line_break())
    }

    pub fn push_symbol_name(&mut self, symbol: &Symbol) -> &mut Self {
        let flags = symbol.flags;
        let name = &symbol.escaped_name;

        let part = if flags.contains(SymbolFlags::Class) {
            SymbolDisplayPart::class_name(name)
        } else if flags.contains(SymbolFlags::Interface) {
            SymbolDisplayPart::interface_name(name)
        } else if flags.contains(SymbolFlags::TypeAlias) {
            SymbolDisplayPart::type_alias_name(name)
        } else if flags.contains(SymbolFlags::Enum) {
            SymbolDisplayPart::enum_name(name)
        } else if flags.contains(SymbolFlags::EnumMember) {
            SymbolDisplayPart::enum_member_name(name)
        } else if flags.contains(SymbolFlags::Function) {
            SymbolDisplayPart::function_name(name)
        } else if flags.contains(SymbolFlags::Method) {
            SymbolDisplayPart::method_name(name)
        } else if flags.contains(SymbolFlags::Property) {
            SymbolDisplayPart::property_name(name)
        } else if flags.contains(SymbolFlags::Module) || flags.contains(SymbolFlags::Namespace) {
            SymbolDisplayPart::module_name(name)
        } else if flags.contains(SymbolFlags::TypeParameter) {
            SymbolDisplayPart::type_parameter_name(name)
        } else if flags.contains(SymbolFlags::Alias) {
            SymbolDisplayPart::alias_name(name)
        } else {
            SymbolDisplayPart::local_name(name)
        };

        self.push(part)
    }

    pub fn build(self) -> Vec<SymbolDisplayPart> {
        self.parts
    }

    pub fn to_string(&self) -> String {
        self.parts.iter().map(|p| p.text.as_str()).collect()
    }
}

// =============================================================================
// Symbol Display Functions
// =============================================================================

/// Get display parts for a symbol's declaration.
pub fn get_symbol_display_parts(
    symbol: &Symbol,
    include_type: bool,
) -> Vec<SymbolDisplayPart> {
    let mut builder = DisplayPartsBuilder::new();

    // Add the declaration keyword
    let flags = symbol.flags;

    if flags.contains(SymbolFlags::Class) {
        builder.push_keyword("class");
    } else if flags.contains(SymbolFlags::Interface) {
        builder.push_keyword("interface");
    } else if flags.contains(SymbolFlags::TypeAlias) {
        builder.push_keyword("type");
    } else if flags.contains(SymbolFlags::Enum) {
        builder.push_keyword("enum");
    } else if flags.contains(SymbolFlags::EnumMember) {
        builder.push_punctuation("(");
        builder.push_keyword("enum member");
        builder.push_punctuation(")");
    } else if flags.contains(SymbolFlags::Function) {
        builder.push_keyword("function");
    } else if flags.contains(SymbolFlags::Method) {
        builder.push_punctuation("(");
        builder.push_keyword("method");
        builder.push_punctuation(")");
    } else if flags.contains(SymbolFlags::Property) {
        builder.push_punctuation("(");
        builder.push_keyword("property");
        builder.push_punctuation(")");
    } else if flags.contains(SymbolFlags::GetAccessor) {
        builder.push_punctuation("(");
        builder.push_keyword("getter");
        builder.push_punctuation(")");
    } else if flags.contains(SymbolFlags::SetAccessor) {
        builder.push_punctuation("(");
        builder.push_keyword("setter");
        builder.push_punctuation(")");
    } else if flags.contains(SymbolFlags::Constructor) {
        builder.push_keyword("constructor");
    } else if flags.contains(SymbolFlags::Module) || flags.contains(SymbolFlags::Namespace) {
        builder.push_keyword("namespace");
    } else if flags.contains(SymbolFlags::TypeParameter) {
        builder.push_punctuation("(");
        builder.push_keyword("type parameter");
        builder.push_punctuation(")");
    } else if flags.contains(SymbolFlags::Const) {
        builder.push_keyword("const");
    } else if flags.contains(SymbolFlags::BlockScopedVariable) {
        builder.push_keyword("let");
    } else if flags.contains(SymbolFlags::FunctionScopedVariable) {
        builder.push_keyword("var");
    }

    builder.push_space();
    builder.push_symbol_name(symbol);

    builder.build()
}

/// Get documentation for a symbol from JSDoc.
pub fn get_symbol_documentation(
    symbol: &Symbol,
) -> Vec<SymbolDisplayPart> {
    // In a full implementation, this would extract JSDoc comments
    // from the symbol's declarations
    Vec::new()
}

/// Get JSDoc tags for a symbol.
pub fn get_symbol_jsdoc_tags(
    symbol: &Symbol,
) -> Vec<JSDocTagInfo> {
    // In a full implementation, this would extract JSDoc tags
    // from the symbol's declarations
    Vec::new()
}

/// Get the full display for a symbol including type.
pub fn get_symbol_display_parts_documentation_and_kind(
    symbol: &Symbol,
) -> (Vec<SymbolDisplayPart>, Vec<SymbolDisplayPart>, ScriptElementKind, Vec<JSDocTagInfo>) {
    let display_parts = get_symbol_display_parts(symbol, true);
    let documentation = get_symbol_documentation(symbol);
    let kind = super::utilities::get_symbol_kind(symbol);
    let tags = get_symbol_jsdoc_tags(symbol);

    (display_parts, documentation, kind, tags)
}

// =============================================================================
// Type Display
// =============================================================================

/// Options for type display.
#[derive(Debug, Clone, Default)]
pub struct TypeDisplayOptions {
    /// Maximum truncation length.
    pub maximum_length: Option<u32>,
    /// Whether to include constraint for type parameters.
    pub include_constraints: bool,
    /// Whether to expand type aliases.
    pub expand_type_aliases: bool,
}

/// Convert a type to display parts.
pub fn type_to_display_parts(
    checker: &CheckerState,
    type_id: TypeId,
    options: &TypeDisplayOptions,
) -> Vec<SymbolDisplayPart> {
    let type_string = checker.type_to_string(type_id);

    // For now, just return as text. A full implementation would
    // parse the type and colorize different parts.
    let mut parts = Vec::new();

    // Truncate if necessary
    let text = if let Some(max_len) = options.maximum_length {
        if type_string.len() > max_len as usize {
            format!("{}...", &type_string[..max_len as usize - 3])
        } else {
            type_string
        }
    } else {
        type_string
    };

    parts.push(SymbolDisplayPart::text(&text));
    parts
}

/// Convert display parts to a single string.
pub fn display_parts_to_string(parts: &[SymbolDisplayPart]) -> String {
    parts.iter().map(|p| p.text.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_parts_builder() {
        let mut builder = DisplayPartsBuilder::new();
        builder
            .push_keyword("const")
            .push_space()
            .push(SymbolDisplayPart::local_name("x"))
            .push_punctuation(":")
            .push_space()
            .push_keyword("number");

        let parts = builder.build();
        assert_eq!(parts.len(), 6);
        assert_eq!(display_parts_to_string(&parts), "const x: number");
    }

    #[test]
    fn test_symbol_display_part_kinds() {
        assert_eq!(SymbolDisplayPartKind::Keyword.as_str(), "keyword");
        assert_eq!(SymbolDisplayPartKind::ClassName.as_str(), "className");
    }
}
