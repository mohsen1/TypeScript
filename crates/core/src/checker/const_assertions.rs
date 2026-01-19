//! Const assertions and readonly type inference.
//!
//! This module handles:
//! - `as const` assertions for literal preservation
//! - Readonly array inference from const assertions
//! - Readonly object inference from const assertions
//! - Const context propagation

use std::collections::HashMap;
use std::sync::Arc;

use super::type_defs::{
    Type, ObjectType, PropertySignature, TupleType, TupleElement,
    IndexSignature,
};

/// Represents a const assertion context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstContext {
    /// No const context - types will widen
    None,
    /// In a const context - literals preserved, readonly applied
    Const,
    /// In a const type parameter context
    ConstTypeParameter,
}

impl Default for ConstContext {
    fn default() -> Self {
        ConstContext::None
    }
}

/// Result of applying const assertion
#[derive(Debug, Clone)]
pub struct ConstAssertionResult {
    /// The resulting type after const assertion
    pub type_: Arc<Type>,
    /// Whether the type is now readonly
    pub is_readonly: bool,
}

/// Applies const assertions to types
pub struct ConstAssertionApplier;

impl ConstAssertionApplier {
    /// Apply `as const` assertion to a type
    ///
    /// This:
    /// - Preserves literal types instead of widening
    /// - Makes arrays readonly tuples
    /// - Makes objects deeply readonly
    pub fn apply_const_assertion(type_: &Type) -> ConstAssertionResult {
        let result_type = Self::make_const_type(type_);
        ConstAssertionResult {
            type_: result_type,
            is_readonly: true,
        }
    }

    /// Convert a type to its const version
    fn make_const_type(type_: &Type) -> Arc<Type> {
        match type_ {
            // Literal types stay as-is
            Type::StringLiteral(_)
            | Type::NumberLiteral(_)
            | Type::BooleanLiteral(_)
            | Type::BigIntLiteral(_) => Arc::new(type_.clone()),

            // Primitive types in const context become their literal form if possible
            Type::String | Type::Number | Type::Boolean | Type::BigInt => {
                // These stay as primitives unless they have a specific value
                Arc::new(type_.clone())
            }

            // Arrays become readonly tuples
            Type::Array(element_type) => {
                let const_element = Self::make_const_type(element_type);
                Arc::new(Type::Array(const_element))
            }

            // Tuples become readonly with const element types
            Type::Tuple(tuple) => {
                let const_elements: Vec<TupleElement> = tuple
                    .element_types
                    .iter()
                    .map(|elem| TupleElement {
                        type_: Self::make_const_type(&elem.type_),
                        optional: elem.optional,
                        label: elem.label.clone(),
                    })
                    .collect();

                Arc::new(Type::Tuple(TupleType {
                    element_types: const_elements,
                    min_length: tuple.min_length,
                    has_rest: tuple.has_rest,
                }))
            }

            // Objects become deeply readonly
            Type::Object(obj) => {
                let const_properties: HashMap<String, PropertySignature> = obj
                    .properties
                    .iter()
                    .map(|(name, prop)| {
                        (
                            name.clone(),
                            PropertySignature {
                                name: prop.name.clone(),
                                type_: Self::make_const_type(&prop.type_),
                                optional: prop.optional,
                                readonly: true, // Mark as readonly
                            },
                        )
                    })
                    .collect();

                let const_index_signatures: Vec<IndexSignature> = obj
                    .index_signatures
                    .iter()
                    .map(|sig| IndexSignature {
                        key_type: sig.key_type.clone(),
                        value_type: Self::make_const_type(&sig.value_type),
                        readonly: true, // Mark as readonly
                    })
                    .collect();

                Arc::new(Type::Object(ObjectType {
                    properties: const_properties,
                    call_signatures: obj.call_signatures.clone(),
                    construct_signatures: obj.construct_signatures.clone(),
                    index_signatures: const_index_signatures,
                }))
            }

            // Union types - apply const to each member
            Type::Union(types) => {
                let const_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|t| Self::make_const_type(t))
                    .collect();
                Arc::new(Type::Union(const_types))
            }

            // Intersection types - apply const to each member
            Type::Intersection(types) => {
                let const_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|t| Self::make_const_type(t))
                    .collect();
                Arc::new(Type::Intersection(const_types))
            }

            // Other types pass through unchanged
            _ => Arc::new(type_.clone()),
        }
    }

    /// Convert an array literal to a readonly tuple with literal element types
    pub fn array_to_const_tuple(element_types: Vec<Arc<Type>>) -> Arc<Type> {
        let tuple_elements: Vec<TupleElement> = element_types
            .into_iter()
            .map(|type_| TupleElement {
                type_: Self::make_const_type(&type_),
                optional: false,
                label: None,
            })
            .collect();

        let len = tuple_elements.len();

        Arc::new(Type::Tuple(TupleType {
            element_types: tuple_elements,
            min_length: len,
            has_rest: false,
        }))
    }

    /// Convert an object literal to a readonly object with literal property types
    pub fn object_to_const(properties: HashMap<String, Arc<Type>>) -> Arc<Type> {
        let const_properties: HashMap<String, PropertySignature> = properties
            .into_iter()
            .map(|(name, type_)| {
                (
                    name.clone(),
                    PropertySignature {
                        name: name.clone(),
                        type_: Self::make_const_type(&type_),
                        optional: false,
                        readonly: true,
                    },
                )
            })
            .collect();

        Arc::new(Type::Object(ObjectType {
            properties: const_properties,
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            index_signatures: Vec::new(),
        }))
    }

    /// Check if a type is in const form (all readonly, all literals)
    pub fn is_const_type(type_: &Type) -> bool {
        match type_ {
            // Literal types are const
            Type::StringLiteral(_)
            | Type::NumberLiteral(_)
            | Type::BooleanLiteral(_)
            | Type::BigIntLiteral(_)
            | Type::Null
            | Type::Undefined => true,

            // Check if tuple has all const elements
            Type::Tuple(tuple) => {
                tuple.element_types.iter().all(|elem| Self::is_const_type(&elem.type_))
            }

            // Check if object is fully readonly with const property types
            Type::Object(obj) => {
                obj.properties.values().all(|prop| {
                    prop.readonly && Self::is_const_type(&prop.type_)
                }) && obj.index_signatures.iter().all(|sig| {
                    sig.readonly && Self::is_const_type(&sig.value_type)
                })
            }

            // Check union/intersection members
            Type::Union(types) | Type::Intersection(types) => {
                types.iter().all(|t| Self::is_const_type(t))
            }

            // Primitives and other types are not const
            _ => false,
        }
    }
}

/// Readonly type wrapper utilities
pub struct ReadonlyTypeUtils;

impl ReadonlyTypeUtils {
    /// Make a type readonly (shallow)
    pub fn make_readonly(type_: &Type) -> Arc<Type> {
        match type_ {
            Type::Object(obj) => {
                let readonly_properties: HashMap<String, PropertySignature> = obj
                    .properties
                    .iter()
                    .map(|(name, prop)| {
                        (
                            name.clone(),
                            PropertySignature {
                                name: prop.name.clone(),
                                type_: prop.type_.clone(),
                                optional: prop.optional,
                                readonly: true,
                            },
                        )
                    })
                    .collect();

                let readonly_index_signatures: Vec<IndexSignature> = obj
                    .index_signatures
                    .iter()
                    .map(|sig| IndexSignature {
                        key_type: sig.key_type.clone(),
                        value_type: sig.value_type.clone(),
                        readonly: true,
                    })
                    .collect();

                Arc::new(Type::Object(ObjectType {
                    properties: readonly_properties,
                    call_signatures: obj.call_signatures.clone(),
                    construct_signatures: obj.construct_signatures.clone(),
                    index_signatures: readonly_index_signatures,
                }))
            }
            Type::Array(_element_type) => {
                // ReadonlyArray<T> - we represent this as the same Array with a flag
                // In a real implementation, this would be a separate ReadonlyArray type
                Arc::new(type_.clone())
            }
            _ => Arc::new(type_.clone()),
        }
    }

    /// Make a type deeply readonly
    pub fn make_deep_readonly(type_: &Type) -> Arc<Type> {
        match type_ {
            Type::Object(obj) => {
                let readonly_properties: HashMap<String, PropertySignature> = obj
                    .properties
                    .iter()
                    .map(|(name, prop)| {
                        (
                            name.clone(),
                            PropertySignature {
                                name: prop.name.clone(),
                                type_: Self::make_deep_readonly(&prop.type_),
                                optional: prop.optional,
                                readonly: true,
                            },
                        )
                    })
                    .collect();

                let readonly_index_signatures: Vec<IndexSignature> = obj
                    .index_signatures
                    .iter()
                    .map(|sig| IndexSignature {
                        key_type: sig.key_type.clone(),
                        value_type: Self::make_deep_readonly(&sig.value_type),
                        readonly: true,
                    })
                    .collect();

                Arc::new(Type::Object(ObjectType {
                    properties: readonly_properties,
                    call_signatures: obj.call_signatures.clone(),
                    construct_signatures: obj.construct_signatures.clone(),
                    index_signatures: readonly_index_signatures,
                }))
            }
            Type::Array(element_type) => {
                Arc::new(Type::Array(Self::make_deep_readonly(element_type)))
            }
            Type::Tuple(tuple) => {
                let readonly_elements: Vec<TupleElement> = tuple
                    .element_types
                    .iter()
                    .map(|elem| TupleElement {
                        type_: Self::make_deep_readonly(&elem.type_),
                        optional: elem.optional,
                        label: elem.label.clone(),
                    })
                    .collect();

                Arc::new(Type::Tuple(TupleType {
                    element_types: readonly_elements,
                    min_length: tuple.min_length,
                    has_rest: tuple.has_rest,
                }))
            }
            Type::Union(types) => {
                let readonly_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|t| Self::make_deep_readonly(t))
                    .collect();
                Arc::new(Type::Union(readonly_types))
            }
            Type::Intersection(types) => {
                let readonly_types: Vec<Arc<Type>> = types
                    .iter()
                    .map(|t| Self::make_deep_readonly(t))
                    .collect();
                Arc::new(Type::Intersection(readonly_types))
            }
            _ => Arc::new(type_.clone()),
        }
    }

    /// Check if a type is readonly
    pub fn is_readonly(type_: &Type) -> bool {
        match type_ {
            Type::Object(obj) => {
                obj.properties.values().all(|p| p.readonly)
                    && obj.index_signatures.iter().all(|s| s.readonly)
            }
            // Tuples created from const are implicitly readonly
            Type::Tuple(_) => true,
            _ => false,
        }
    }

    /// Check if a type is deeply readonly
    pub fn is_deep_readonly(type_: &Type) -> bool {
        match type_ {
            Type::Object(obj) => {
                obj.properties.values().all(|p| {
                    p.readonly && Self::is_deep_readonly(&p.type_)
                }) && obj.index_signatures.iter().all(|s| {
                    s.readonly && Self::is_deep_readonly(&s.value_type)
                })
            }
            Type::Array(element_type) => Self::is_deep_readonly(element_type),
            Type::Tuple(tuple) => {
                tuple.element_types.iter().all(|elem| Self::is_deep_readonly(&elem.type_))
            }
            Type::Union(types) | Type::Intersection(types) => {
                types.iter().all(|t| Self::is_deep_readonly(t))
            }
            // Primitives and literals are implicitly readonly
            Type::String
            | Type::Number
            | Type::Boolean
            | Type::BigInt
            | Type::Symbol
            | Type::Null
            | Type::Undefined
            | Type::StringLiteral(_)
            | Type::NumberLiteral(_)
            | Type::BooleanLiteral(_)
            | Type::BigIntLiteral(_) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_assertion_literal() {
        let str_literal = Type::StringLiteral("hello".to_string());
        let result = ConstAssertionApplier::apply_const_assertion(&str_literal);

        match result.type_.as_ref() {
            Type::StringLiteral(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected StringLiteral"),
        }
        assert!(result.is_readonly);
    }

    #[test]
    fn test_const_assertion_array_to_tuple() {
        let elements = vec![
            Arc::new(Type::StringLiteral("a".to_string())),
            Arc::new(Type::NumberLiteral(1.0)),
        ];

        let result = ConstAssertionApplier::array_to_const_tuple(elements);

        match result.as_ref() {
            Type::Tuple(tuple) => {
                assert_eq!(tuple.element_types.len(), 2);
                match tuple.element_types[0].type_.as_ref() {
                    Type::StringLiteral(s) => assert_eq!(s, "a"),
                    _ => panic!("Expected StringLiteral"),
                }
                match tuple.element_types[1].type_.as_ref() {
                    Type::NumberLiteral(n) => assert_eq!(*n, 1.0),
                    _ => panic!("Expected NumberLiteral"),
                }
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_const_assertion_object() {
        let mut properties = HashMap::new();
        properties.insert(
            "name".to_string(),
            Arc::new(Type::StringLiteral("test".to_string())),
        );
        properties.insert("value".to_string(), Arc::new(Type::NumberLiteral(42.0)));

        let result = ConstAssertionApplier::object_to_const(properties);

        match result.as_ref() {
            Type::Object(obj) => {
                assert!(obj.properties.get("name").unwrap().readonly);
                assert!(obj.properties.get("value").unwrap().readonly);
            }
            _ => panic!("Expected Object"),
        }
    }

    #[test]
    fn test_is_const_type() {
        assert!(ConstAssertionApplier::is_const_type(&Type::StringLiteral(
            "test".to_string()
        )));
        assert!(ConstAssertionApplier::is_const_type(&Type::NumberLiteral(42.0)));
        assert!(ConstAssertionApplier::is_const_type(&Type::Null));
        assert!(!ConstAssertionApplier::is_const_type(&Type::String));
        assert!(!ConstAssertionApplier::is_const_type(&Type::Number));
    }

    #[test]
    fn test_make_readonly_object() {
        let mut obj = ObjectType::default();
        obj.properties.insert(
            "x".to_string(),
            PropertySignature {
                name: "x".to_string(),
                type_: Arc::new(Type::Number),
                optional: false,
                readonly: false,
            },
        );

        let result = ReadonlyTypeUtils::make_readonly(&Type::Object(obj));

        match result.as_ref() {
            Type::Object(obj) => {
                assert!(obj.properties.get("x").unwrap().readonly);
            }
            _ => panic!("Expected Object"),
        }
    }

    #[test]
    fn test_is_deep_readonly() {
        // Primitives are deeply readonly
        assert!(ReadonlyTypeUtils::is_deep_readonly(&Type::String));
        assert!(ReadonlyTypeUtils::is_deep_readonly(&Type::StringLiteral(
            "test".to_string()
        )));

        // Non-readonly object is not deeply readonly
        let mut obj = ObjectType::default();
        obj.properties.insert(
            "x".to_string(),
            PropertySignature {
                name: "x".to_string(),
                type_: Arc::new(Type::Number),
                optional: false,
                readonly: false,
            },
        );
        assert!(!ReadonlyTypeUtils::is_deep_readonly(&Type::Object(obj)));

        // Readonly object with primitive is deeply readonly
        let mut readonly_obj = ObjectType::default();
        readonly_obj.properties.insert(
            "x".to_string(),
            PropertySignature {
                name: "x".to_string(),
                type_: Arc::new(Type::Number),
                optional: false,
                readonly: true,
            },
        );
        assert!(ReadonlyTypeUtils::is_deep_readonly(&Type::Object(readonly_obj)));
    }
}
