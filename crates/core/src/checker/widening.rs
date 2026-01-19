//! Literal type widening implementation.
//!
//! This module handles:
//! - Fresh literal types vs widened types
//! - Widening rules for literals
//! - Preventing widening in const contexts
//! - Template literal widening
//! - Enum literal widening

use std::collections::HashMap;
use std::sync::Arc;

#[allow(unused_imports)]
use super::type_defs::{
    Type, ObjectType, PropertySignature, TupleType, TupleElement,
    EnumType, EnumMember, EnumValue,
};
use super::const_assertions::ConstContext;

/// A fresh literal type that may be widened
#[derive(Debug, Clone)]
pub struct FreshLiteralType {
    /// The literal type
    pub type_: Arc<Type>,
    /// Whether this literal is "fresh" (can be widened)
    pub is_fresh: bool,
    /// The widened form of this type
    pub widened_type: Arc<Type>,
}

impl FreshLiteralType {
    /// Create a new fresh string literal
    pub fn fresh_string(value: String) -> Self {
        Self {
            type_: Arc::new(Type::StringLiteral(value)),
            is_fresh: true,
            widened_type: Arc::new(Type::String),
        }
    }

    /// Create a new fresh number literal
    pub fn fresh_number(value: f64) -> Self {
        Self {
            type_: Arc::new(Type::NumberLiteral(value)),
            is_fresh: true,
            widened_type: Arc::new(Type::Number),
        }
    }

    /// Create a new fresh boolean literal
    pub fn fresh_boolean(value: bool) -> Self {
        Self {
            type_: Arc::new(Type::BooleanLiteral(value)),
            is_fresh: true,
            widened_type: Arc::new(Type::Boolean),
        }
    }

    /// Create a new fresh bigint literal
    pub fn fresh_bigint(value: i64) -> Self {
        Self {
            type_: Arc::new(Type::BigIntLiteral(value)),
            is_fresh: true,
            widened_type: Arc::new(Type::BigInt),
        }
    }

    /// Create a non-fresh (permanent) literal
    pub fn permanent(type_: Arc<Type>) -> Self {
        let widened = TypeWidener::get_widened_type(&type_);
        Self {
            type_,
            is_fresh: false,
            widened_type: widened,
        }
    }

    /// Get the type, widening if fresh and not in const context
    pub fn get_type(&self, context: ConstContext) -> Arc<Type> {
        if self.is_fresh && context == ConstContext::None {
            self.widened_type.clone()
        } else {
            self.type_.clone()
        }
    }

    /// Mark this literal as non-fresh (prevents widening)
    pub fn make_non_fresh(&mut self) {
        self.is_fresh = false;
    }
}

/// Type widener - handles widening of literal types
pub struct TypeWidener;

impl TypeWidener {
    /// Widen a type according to TypeScript widening rules
    ///
    /// Widening converts:
    /// - String literals -> string
    /// - Number literals -> number
    /// - Boolean literals -> boolean
    /// - BigInt literals -> bigint
    /// - null -> null (special case in strict mode)
    /// - undefined -> undefined (special case in strict mode)
    pub fn widen(type_: &Type, context: ConstContext) -> Arc<Type> {
        if context != ConstContext::None {
            // Don't widen in const context
            return Arc::new(type_.clone());
        }

        Self::widen_type_internal(type_)
    }

    /// Internal widening implementation
    fn widen_type_internal(type_: &Type) -> Arc<Type> {
        match type_ {
            // Widen literals to their base types
            Type::StringLiteral(_) => Arc::new(Type::String),
            Type::NumberLiteral(_) => Arc::new(Type::Number),
            Type::BooleanLiteral(_) => Arc::new(Type::Boolean),
            Type::BigIntLiteral(_) => Arc::new(Type::BigInt),

            // Widen null/undefined in non-strict contexts
            // (In strict mode, these don't widen)
            Type::Null => Arc::new(Type::Null),
            Type::Undefined => Arc::new(Type::Undefined),

            // Widen array element types
            Type::Array(element_type) => {
                Arc::new(Type::Array(Self::widen_type_internal(element_type)))
            }

            // Widen tuple element types
            Type::Tuple(tuple) => {
                let widened_elements: Vec<TupleElement> = tuple
                    .element_types
                    .iter()
                    .map(|elem| TupleElement {
                        type_: Self::widen_type_internal(&elem.type_),
                        optional: elem.optional,
                        label: elem.label.clone(),
                    })
                    .collect();

                Arc::new(Type::Tuple(TupleType {
                    element_types: widened_elements,
                    min_length: tuple.min_length,
                    has_rest: tuple.has_rest,
                }))
            }

            // Widen object property types
            Type::Object(obj) => {
                let widened_properties: HashMap<String, PropertySignature> = obj
                    .properties
                    .iter()
                    .map(|(name, prop)| {
                        (
                            name.clone(),
                            PropertySignature {
                                name: prop.name.clone(),
                                type_: Self::widen_type_internal(&prop.type_),
                                optional: prop.optional,
                                readonly: prop.readonly,
                            },
                        )
                    })
                    .collect();

                Arc::new(Type::Object(ObjectType {
                    properties: widened_properties,
                    call_signatures: obj.call_signatures.clone(),
                    construct_signatures: obj.construct_signatures.clone(),
                    index_signatures: obj.index_signatures.clone(),
                }))
            }

            // Widen union members
            Type::Union(types) => {
                let widened_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|t| Self::widen_type_internal(t))
                    .collect();

                // Remove duplicates after widening
                Self::simplify_union(widened_types)
            }

            // Widen intersection members
            Type::Intersection(types) => {
                let widened_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|t| Self::widen_type_internal(t))
                    .collect();
                Arc::new(Type::Intersection(widened_types))
            }

            // Other types don't widen
            _ => Arc::new(type_.clone()),
        }
    }

    /// Get the widened form of a type without applying widening
    pub fn get_widened_type(type_: &Type) -> Arc<Type> {
        match type_ {
            Type::StringLiteral(_) => Arc::new(Type::String),
            Type::NumberLiteral(_) => Arc::new(Type::Number),
            Type::BooleanLiteral(_) => Arc::new(Type::Boolean),
            Type::BigIntLiteral(_) => Arc::new(Type::BigInt),
            _ => Arc::new(type_.clone()),
        }
    }

    /// Check if a type would be widened
    pub fn would_widen(type_: &Type) -> bool {
        matches!(
            type_,
            Type::StringLiteral(_)
                | Type::NumberLiteral(_)
                | Type::BooleanLiteral(_)
                | Type::BigIntLiteral(_)
        )
    }

    /// Check if a type is a fresh literal type
    pub fn is_fresh_literal(type_: &Type) -> bool {
        // In a real implementation, this would check a flag on the type
        // For now, we consider all literals as potentially fresh
        Self::would_widen(type_)
    }

    /// Simplify a union by removing duplicates after widening
    fn simplify_union(types: Vec<Arc<Type>>) -> Arc<Type> {
        let mut seen_string = false;
        let mut seen_number = false;
        let mut seen_boolean = false;
        let mut seen_bigint = false;
        let mut result = Vec::new();

        for t in types {
            match t.as_ref() {
                Type::String => {
                    if !seen_string {
                        seen_string = true;
                        result.push(t);
                    }
                }
                Type::Number => {
                    if !seen_number {
                        seen_number = true;
                        result.push(t);
                    }
                }
                Type::Boolean => {
                    if !seen_boolean {
                        seen_boolean = true;
                        result.push(t);
                    }
                }
                Type::BigInt => {
                    if !seen_bigint {
                        seen_bigint = true;
                        result.push(t);
                    }
                }
                Type::StringLiteral(_) if seen_string => continue,
                Type::NumberLiteral(_) if seen_number => continue,
                Type::BooleanLiteral(_) if seen_boolean => continue,
                Type::BigIntLiteral(_) if seen_bigint => continue,
                _ => result.push(t),
            }
        }

        match result.len() {
            0 => Arc::new(Type::Never),
            1 => result.into_iter().next().unwrap(),
            _ => Arc::new(Type::Union(result)),
        }
    }

    /// Widen null and undefined to any (for loose null checks)
    pub fn widen_null_and_undefined(type_: &Type) -> Arc<Type> {
        match type_ {
            Type::Null | Type::Undefined => Arc::new(Type::Any),
            Type::Union(types) => {
                let widened: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| !matches!(t.as_ref(), Type::Null | Type::Undefined))
                    .cloned()
                    .collect();

                match widened.len() {
                    0 => Arc::new(Type::Any),
                    1 => widened.into_iter().next().unwrap(),
                    _ => Arc::new(Type::Union(widened)),
                }
            }
            _ => Arc::new(type_.clone()),
        }
    }
}

/// Handles enum literal types and their widening
pub struct EnumLiteralWidener;

impl EnumLiteralWidener {
    /// Get the literal type for an enum member
    pub fn get_enum_member_type(enum_type: &EnumType, member_name: &str) -> Option<Arc<Type>> {
        enum_type
            .members
            .iter()
            .find(|m| m.name == member_name)
            .map(|member| match &member.value {
                EnumValue::Number(n) => Arc::new(Type::NumberLiteral(*n)),
                EnumValue::String(s) => Arc::new(Type::StringLiteral(s.clone())),
            })
    }

    /// Widen an enum literal to the enum type
    pub fn widen_enum_literal(enum_type: &EnumType) -> Arc<Type> {
        Arc::new(Type::Enum(enum_type.clone()))
    }

    /// Get all literal types in an enum as a union
    pub fn get_enum_literal_union(enum_type: &EnumType) -> Arc<Type> {
        let literal_types: Vec<Arc<Type>> = enum_type
            .members
            .iter()
            .map(|member| match &member.value {
                EnumValue::Number(n) => Arc::new(Type::NumberLiteral(*n)),
                EnumValue::String(s) => Arc::new(Type::StringLiteral(s.clone())),
            })
            .collect();

        match literal_types.len() {
            0 => Arc::new(Type::Never),
            1 => literal_types.into_iter().next().unwrap(),
            _ => Arc::new(Type::Union(literal_types)),
        }
    }

    /// Check if an enum is a string enum
    pub fn is_string_enum(enum_type: &EnumType) -> bool {
        enum_type
            .members
            .iter()
            .all(|m| matches!(m.value, EnumValue::String(_)))
    }

    /// Check if an enum is a numeric enum
    pub fn is_numeric_enum(enum_type: &EnumType) -> bool {
        enum_type
            .members
            .iter()
            .all(|m| matches!(m.value, EnumValue::Number(_)))
    }

    /// Check if an enum is a heterogeneous enum (mixed string and number)
    pub fn is_heterogeneous_enum(enum_type: &EnumType) -> bool {
        !Self::is_string_enum(enum_type) && !Self::is_numeric_enum(enum_type)
    }
}

/// Template literal type widening
pub struct TemplateLiteralWidener;

impl TemplateLiteralWidener {
    /// Widen template literal parts to string
    ///
    /// Template literal types like `hello ${string}` widen to string
    /// when assigned to a non-const context
    pub fn widen_template_literal(parts: &[TemplatePart]) -> Arc<Type> {
        // If all parts are string literals, the result is a string literal
        let all_literal = parts.iter().all(|p| matches!(p, TemplatePart::Literal(_)));

        if all_literal {
            let combined: String = parts
                .iter()
                .filter_map(|p| match p {
                    TemplatePart::Literal(s) => Some(s.as_str()),
                    _ => None,
                })
                .collect();
            Arc::new(Type::StringLiteral(combined))
        } else {
            // Template contains placeholders, widen to string
            Arc::new(Type::String)
        }
    }

    /// Check if a template literal type should widen
    pub fn should_widen_template(parts: &[TemplatePart], context: ConstContext) -> bool {
        if context != ConstContext::None {
            return false;
        }

        // Widen if any part is not a literal
        parts.iter().any(|p| !matches!(p, TemplatePart::Literal(_)))
    }
}

/// A part of a template literal
#[derive(Debug, Clone)]
pub enum TemplatePart {
    /// A literal string portion
    Literal(String),
    /// A type placeholder
    Placeholder(Arc<Type>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_literal_widening() {
        let fresh = FreshLiteralType::fresh_string("hello".to_string());

        // In const context, get the literal
        let in_const = fresh.get_type(ConstContext::Const);
        match in_const.as_ref() {
            Type::StringLiteral(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected StringLiteral in const context"),
        }

        // In normal context, get widened type
        let widened = fresh.get_type(ConstContext::None);
        match widened.as_ref() {
            Type::String => (),
            _ => panic!("Expected String in normal context"),
        }
    }

    #[test]
    fn test_widen_literals() {
        let str_lit = Type::StringLiteral("test".to_string());
        let widened = TypeWidener::widen(&str_lit, ConstContext::None);
        assert!(matches!(widened.as_ref(), Type::String));

        let num_lit = Type::NumberLiteral(42.0);
        let widened = TypeWidener::widen(&num_lit, ConstContext::None);
        assert!(matches!(widened.as_ref(), Type::Number));

        let bool_lit = Type::BooleanLiteral(true);
        let widened = TypeWidener::widen(&bool_lit, ConstContext::None);
        assert!(matches!(widened.as_ref(), Type::Boolean));
    }

    #[test]
    fn test_no_widening_in_const_context() {
        let str_lit = Type::StringLiteral("test".to_string());
        let result = TypeWidener::widen(&str_lit, ConstContext::Const);

        match result.as_ref() {
            Type::StringLiteral(s) => assert_eq!(s, "test"),
            _ => panic!("Should not widen in const context"),
        }
    }

    #[test]
    fn test_widen_union() {
        let union = Type::Union(vec![
            Arc::new(Type::StringLiteral("a".to_string())),
            Arc::new(Type::StringLiteral("b".to_string())),
        ]);

        let widened = TypeWidener::widen(&union, ConstContext::None);

        // Should simplify to just string
        match widened.as_ref() {
            Type::String => (),
            _ => panic!("Expected String after widening union of string literals"),
        }
    }

    #[test]
    fn test_widen_object() {
        let mut obj = ObjectType::default();
        obj.properties.insert(
            "x".to_string(),
            PropertySignature {
                name: "x".to_string(),
                type_: Arc::new(Type::StringLiteral("hello".to_string())),
                optional: false,
                readonly: false,
            },
        );

        let widened = TypeWidener::widen(&Type::Object(obj), ConstContext::None);

        match widened.as_ref() {
            Type::Object(obj) => {
                match obj.properties.get("x").unwrap().type_.as_ref() {
                    Type::String => (),
                    _ => panic!("Expected property type to be widened to String"),
                }
            }
            _ => panic!("Expected Object"),
        }
    }

    #[test]
    fn test_would_widen() {
        assert!(TypeWidener::would_widen(&Type::StringLiteral("test".to_string())));
        assert!(TypeWidener::would_widen(&Type::NumberLiteral(42.0)));
        assert!(TypeWidener::would_widen(&Type::BooleanLiteral(true)));
        assert!(!TypeWidener::would_widen(&Type::String));
        assert!(!TypeWidener::would_widen(&Type::Number));
    }

    #[test]
    fn test_enum_literal_types() {
        let enum_type = EnumType {
            name: "Color".to_string(),
            members: vec![
                EnumMember {
                    name: "Red".to_string(),
                    value: EnumValue::Number(0.0),
                },
                EnumMember {
                    name: "Green".to_string(),
                    value: EnumValue::Number(1.0),
                },
                EnumMember {
                    name: "Blue".to_string(),
                    value: EnumValue::Number(2.0),
                },
            ],
        };

        assert!(EnumLiteralWidener::is_numeric_enum(&enum_type));
        assert!(!EnumLiteralWidener::is_string_enum(&enum_type));

        let member_type = EnumLiteralWidener::get_enum_member_type(&enum_type, "Red");
        assert!(member_type.is_some());
        match member_type.unwrap().as_ref() {
            Type::NumberLiteral(n) => assert_eq!(*n, 0.0),
            _ => panic!("Expected NumberLiteral"),
        }
    }

    #[test]
    fn test_string_enum() {
        let enum_type = EnumType {
            name: "Direction".to_string(),
            members: vec![
                EnumMember {
                    name: "Up".to_string(),
                    value: EnumValue::String("UP".to_string()),
                },
                EnumMember {
                    name: "Down".to_string(),
                    value: EnumValue::String("DOWN".to_string()),
                },
            ],
        };

        assert!(EnumLiteralWidener::is_string_enum(&enum_type));
        assert!(!EnumLiteralWidener::is_numeric_enum(&enum_type));
    }

    #[test]
    fn test_template_literal_widening() {
        // All literal parts should produce a literal
        let parts = vec![
            TemplatePart::Literal("hello ".to_string()),
            TemplatePart::Literal("world".to_string()),
        ];
        let result = TemplateLiteralWidener::widen_template_literal(&parts);
        match result.as_ref() {
            Type::StringLiteral(s) => assert_eq!(s, "hello world"),
            _ => panic!("Expected StringLiteral"),
        }

        // With placeholder, should widen to string
        let parts_with_placeholder = vec![
            TemplatePart::Literal("hello ".to_string()),
            TemplatePart::Placeholder(Arc::new(Type::String)),
        ];
        let result = TemplateLiteralWidener::widen_template_literal(&parts_with_placeholder);
        assert!(matches!(result.as_ref(), Type::String));
    }

    #[test]
    fn test_widen_null_and_undefined() {
        let union = Type::Union(vec![
            Arc::new(Type::String),
            Arc::new(Type::Null),
            Arc::new(Type::Undefined),
        ]);

        let widened = TypeWidener::widen_null_and_undefined(&union);

        match widened.as_ref() {
            Type::String => (),
            _ => panic!("Expected String after widening null/undefined from union"),
        }
    }

    #[test]
    fn test_permanent_literal() {
        let mut fresh = FreshLiteralType::fresh_string("test".to_string());
        assert!(fresh.is_fresh);

        fresh.make_non_fresh();
        assert!(!fresh.is_fresh);

        // Non-fresh literals don't widen even in normal context
        let result = fresh.get_type(ConstContext::None);
        match result.as_ref() {
            Type::StringLiteral(s) => assert_eq!(s, "test"),
            _ => panic!("Non-fresh literal should not widen"),
        }
    }
}
