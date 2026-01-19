//! Parser support for private identifiers (#field) syntax.
//!
//! This module handles parsing of:
//! - Private field declarations (#field: type)
//! - Private method declarations (#method() {})
//! - Private accessor declarations (get #prop(), set #prop())
//! - Private field access (this.#field)
//! - Private field existence check (#field in obj)
//! - Private static members (static #field)

use std::collections::HashMap;

/// Represents a private identifier (starts with #).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrivateIdentifier {
    /// The name without the # prefix
    pub name: String,
    /// The original text including #
    pub text: String,
    /// Position in source
    pub pos: usize,
    /// End position in source
    pub end: usize,
}

impl PrivateIdentifier {
    /// Create a new private identifier from a name (without #).
    pub fn new(name: impl Into<String>, pos: usize, end: usize) -> Self {
        let name = name.into();
        Self {
            text: format!("#{}", name),
            name,
            pos,
            end,
        }
    }

    /// Parse a private identifier from text.
    pub fn parse(text: &str, pos: usize) -> Option<Self> {
        if !text.starts_with('#') {
            return None;
        }

        let name_part = &text[1..];
        if name_part.is_empty() || !is_valid_identifier_start(name_part.chars().next()?) {
            return None;
        }

        // Validate rest of identifier
        for ch in name_part.chars().skip(1) {
            if !is_valid_identifier_part(ch) {
                return None;
            }
        }

        Some(Self {
            name: name_part.to_string(),
            text: text.to_string(),
            pos,
            end: pos + text.len(),
        })
    }

    /// Check if this is a valid private identifier.
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && is_valid_identifier_start(self.name.chars().next().unwrap())
    }
}

/// Check if character can start an identifier.
fn is_valid_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == '$'
}

/// Check if character can be part of an identifier.
fn is_valid_identifier_part(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}

/// Kind of private class member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivateMemberKind {
    /// Private field (#field)
    Field,
    /// Private method (#method())
    Method,
    /// Private getter (get #prop())
    Getter,
    /// Private setter (set #prop())
    Setter,
}

/// Modifiers that can apply to private members.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PrivateMemberModifiers {
    /// Whether this is a static member.
    pub is_static: bool,
    /// Whether this is readonly (for fields).
    pub is_readonly: bool,
}

/// A private class element declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct PrivateClassElement {
    /// The private identifier.
    pub name: PrivateIdentifier,
    /// Kind of member.
    pub kind: PrivateMemberKind,
    /// Modifiers.
    pub modifiers: PrivateMemberModifiers,
    /// Type annotation (if any).
    pub type_annotation: Option<String>,
    /// Initializer expression (for fields).
    pub initializer: Option<String>,
    /// Parameters (for methods and setters).
    pub parameters: Vec<PrivateParameter>,
    /// Return type (for methods and getters).
    pub return_type: Option<String>,
    /// Method body (for methods and accessors).
    pub body: Option<String>,
}

/// A parameter in a private method.
#[derive(Debug, Clone, PartialEq)]
pub struct PrivateParameter {
    pub name: String,
    pub type_annotation: Option<String>,
    pub optional: bool,
    pub initializer: Option<String>,
}

impl PrivateClassElement {
    /// Create a new private field declaration.
    pub fn field(
        name: PrivateIdentifier,
        modifiers: PrivateMemberModifiers,
        type_annotation: Option<String>,
        initializer: Option<String>,
    ) -> Self {
        Self {
            name,
            kind: PrivateMemberKind::Field,
            modifiers,
            type_annotation,
            initializer,
            parameters: vec![],
            return_type: None,
            body: None,
        }
    }

    /// Create a new private method declaration.
    pub fn method(
        name: PrivateIdentifier,
        modifiers: PrivateMemberModifiers,
        parameters: Vec<PrivateParameter>,
        return_type: Option<String>,
        body: Option<String>,
    ) -> Self {
        Self {
            name,
            kind: PrivateMemberKind::Method,
            modifiers,
            type_annotation: None,
            initializer: None,
            parameters,
            return_type,
            body,
        }
    }

    /// Create a new private getter declaration.
    pub fn getter(
        name: PrivateIdentifier,
        modifiers: PrivateMemberModifiers,
        return_type: Option<String>,
        body: Option<String>,
    ) -> Self {
        Self {
            name,
            kind: PrivateMemberKind::Getter,
            modifiers,
            type_annotation: None,
            initializer: None,
            parameters: vec![],
            return_type,
            body,
        }
    }

    /// Create a new private setter declaration.
    pub fn setter(
        name: PrivateIdentifier,
        modifiers: PrivateMemberModifiers,
        parameter: PrivateParameter,
        body: Option<String>,
    ) -> Self {
        Self {
            name,
            kind: PrivateMemberKind::Setter,
            modifiers,
            type_annotation: None,
            initializer: None,
            parameters: vec![parameter],
            return_type: None,
            body,
        }
    }

    /// Check if this is a static member.
    pub fn is_static(&self) -> bool {
        self.modifiers.is_static
    }

    /// Check if this is a field.
    pub fn is_field(&self) -> bool {
        matches!(self.kind, PrivateMemberKind::Field)
    }

    /// Check if this is a method.
    pub fn is_method(&self) -> bool {
        matches!(self.kind, PrivateMemberKind::Method)
    }

    /// Check if this is an accessor.
    pub fn is_accessor(&self) -> bool {
        matches!(
            self.kind,
            PrivateMemberKind::Getter | PrivateMemberKind::Setter
        )
    }
}

/// Represents a private field access expression.
#[derive(Debug, Clone, PartialEq)]
pub struct PrivateFieldAccess {
    /// The object being accessed.
    pub object: String,
    /// The private identifier.
    pub name: PrivateIdentifier,
}

impl PrivateFieldAccess {
    pub fn new(object: impl Into<String>, name: PrivateIdentifier) -> Self {
        Self {
            object: object.into(),
            name,
        }
    }
}

/// Represents a private field existence check (#field in obj).
#[derive(Debug, Clone, PartialEq)]
pub struct PrivateFieldInExpression {
    /// The private identifier being checked.
    pub name: PrivateIdentifier,
    /// The object being checked.
    pub object: String,
}

impl PrivateFieldInExpression {
    pub fn new(name: PrivateIdentifier, object: impl Into<String>) -> Self {
        Self {
            name,
            object: object.into(),
        }
    }
}

/// Parser for private class members.
pub struct PrivateIdentifierParser {
    /// Current position in source.
    pos: usize,
    /// Source text.
    source: String,
}

impl PrivateIdentifierParser {
    /// Create a new parser.
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            pos: 0,
            source: source.into(),
        }
    }

    /// Try to parse a private identifier at current position.
    pub fn parse_private_identifier(&mut self) -> Option<PrivateIdentifier> {
        self.skip_whitespace();

        if !self.current_char()?.eq(&'#') {
            return None;
        }

        let start = self.pos;
        self.advance(); // skip #

        let name_start = self.pos;
        while let Some(ch) = self.current_char() {
            if self.pos == name_start {
                if !is_valid_identifier_start(ch) {
                    return None;
                }
            } else if !is_valid_identifier_part(ch) {
                break;
            }
            self.advance();
        }

        if self.pos == name_start {
            return None;
        }

        let name = self.source[name_start..self.pos].to_string();
        Some(PrivateIdentifier::new(name, start, self.pos))
    }

    /// Skip whitespace.
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if !ch.is_whitespace() {
                break;
            }
            self.advance();
        }
    }

    /// Get current character.
    fn current_char(&self) -> Option<char> {
        self.source.chars().nth(self.pos)
    }

    /// Advance position.
    fn advance(&mut self) {
        self.pos += 1;
    }

    /// Check if we're at end.
    #[allow(dead_code)]
    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }
}

/// Collect all private identifiers in a class.
pub fn collect_private_identifiers(members: &[PrivateClassElement]) -> Vec<&PrivateIdentifier> {
    members.iter().map(|m| &m.name).collect()
}

/// Check if a name is a private identifier.
pub fn is_private_identifier(name: &str) -> bool {
    name.starts_with('#') && name.len() > 1
}

/// Get the public name from a private identifier.
pub fn get_private_identifier_name(text: &str) -> Option<&str> {
    if text.starts_with('#') && text.len() > 1 {
        Some(&text[1..])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_identifier_new() {
        let id = PrivateIdentifier::new("field", 0, 6);
        assert_eq!(id.name, "field");
        assert_eq!(id.text, "#field");
        assert!(id.is_valid());
    }

    #[test]
    fn test_private_identifier_parse() {
        let id = PrivateIdentifier::parse("#myField", 0).unwrap();
        assert_eq!(id.name, "myField");
        assert_eq!(id.text, "#myField");

        // Invalid cases
        assert!(PrivateIdentifier::parse("myField", 0).is_none());
        assert!(PrivateIdentifier::parse("#", 0).is_none());
        assert!(PrivateIdentifier::parse("#123", 0).is_none());
    }

    #[test]
    fn test_private_class_element_field() {
        let id = PrivateIdentifier::new("value", 0, 6);
        let field = PrivateClassElement::field(
            id.clone(),
            PrivateMemberModifiers::default(),
            Some("number".to_string()),
            Some("0".to_string()),
        );

        assert!(field.is_field());
        assert!(!field.is_static());
        assert!(!field.is_method());
        assert_eq!(field.type_annotation, Some("number".to_string()));
    }

    #[test]
    fn test_private_class_element_static_field() {
        let id = PrivateIdentifier::new("count", 0, 6);
        let field = PrivateClassElement::field(
            id.clone(),
            PrivateMemberModifiers {
                is_static: true,
                is_readonly: false,
            },
            Some("number".to_string()),
            None,
        );

        assert!(field.is_field());
        assert!(field.is_static());
    }

    #[test]
    fn test_private_class_element_method() {
        let id = PrivateIdentifier::new("doSomething", 0, 12);
        let method = PrivateClassElement::method(
            id.clone(),
            PrivateMemberModifiers::default(),
            vec![PrivateParameter {
                name: "x".to_string(),
                type_annotation: Some("number".to_string()),
                optional: false,
                initializer: None,
            }],
            Some("void".to_string()),
            Some("{ }".to_string()),
        );

        assert!(method.is_method());
        assert!(!method.is_field());
        assert_eq!(method.parameters.len(), 1);
    }

    #[test]
    fn test_private_class_element_accessors() {
        let id = PrivateIdentifier::new("prop", 0, 5);

        let getter = PrivateClassElement::getter(
            id.clone(),
            PrivateMemberModifiers::default(),
            Some("string".to_string()),
            Some("{ return this.#_prop; }".to_string()),
        );
        assert!(getter.is_accessor());
        assert!(matches!(getter.kind, PrivateMemberKind::Getter));

        let setter = PrivateClassElement::setter(
            id.clone(),
            PrivateMemberModifiers::default(),
            PrivateParameter {
                name: "value".to_string(),
                type_annotation: Some("string".to_string()),
                optional: false,
                initializer: None,
            },
            Some("{ this.#_prop = value; }".to_string()),
        );
        assert!(setter.is_accessor());
        assert!(matches!(setter.kind, PrivateMemberKind::Setter));
    }

    #[test]
    fn test_private_field_access() {
        let id = PrivateIdentifier::new("field", 0, 6);
        let access = PrivateFieldAccess::new("this", id);
        assert_eq!(access.object, "this");
        assert_eq!(access.name.name, "field");
    }

    #[test]
    fn test_private_field_in_expression() {
        let id = PrivateIdentifier::new("field", 0, 6);
        let expr = PrivateFieldInExpression::new(id, "obj");
        assert_eq!(expr.name.name, "field");
        assert_eq!(expr.object, "obj");
    }

    #[test]
    fn test_parser() {
        let mut parser = PrivateIdentifierParser::new("  #myPrivateField");
        let id = parser.parse_private_identifier().unwrap();
        assert_eq!(id.name, "myPrivateField");
    }

    #[test]
    fn test_is_private_identifier() {
        assert!(is_private_identifier("#field"));
        assert!(is_private_identifier("#_internal"));
        assert!(!is_private_identifier("field"));
        assert!(!is_private_identifier("#"));
    }

    #[test]
    fn test_get_private_identifier_name() {
        assert_eq!(get_private_identifier_name("#field"), Some("field"));
        assert_eq!(get_private_identifier_name("#_x"), Some("_x"));
        assert_eq!(get_private_identifier_name("field"), None);
        assert_eq!(get_private_identifier_name("#"), None);
    }
}
