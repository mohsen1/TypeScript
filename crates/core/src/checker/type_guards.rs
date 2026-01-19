//! Type guards implementation for TypeScript type narrowing.
//!
//! This module implements various type guard patterns:
//! - typeof type guards
//! - instanceof type guards
//! - User-defined type guards (x is T)
//! - in operator narrowing
//! - Equality checks narrowing
//! - Truthiness narrowing

use std::sync::Arc;

use super::type_defs::{Type, ObjectType, ClassType, PropertySignature};

/// Result of a type guard check
#[derive(Debug, Clone)]
pub struct TypeGuardResult {
    /// Type when the guard is true
    pub true_type: Arc<Type>,
    /// Type when the guard is false
    pub false_type: Arc<Type>,
}

/// Type guard kind
#[derive(Debug, Clone)]
pub enum TypeGuardKind {
    /// typeof x === "string"
    Typeof(String),
    /// x instanceof Class
    Instanceof(Arc<Type>),
    /// User-defined type guard (x is T)
    UserDefined(Arc<Type>),
    /// x === value (equality)
    StrictEquality(Arc<Type>),
    /// x == value (loose equality)
    LooseEquality(Arc<Type>),
    /// "prop" in x
    InOperator(String),
    /// Truthiness check
    Truthiness,
    /// x !== null && x !== undefined
    NonNullable,
}

/// Type guard evaluator
pub struct TypeGuardEvaluator;

impl TypeGuardEvaluator {
    /// Apply a typeof type guard
    pub fn narrow_typeof(source_type: &Type, typeof_value: &str, negate: bool) -> TypeGuardResult {
        let narrowed = Self::typeof_narrow_type(source_type, typeof_value);
        let excluded = Self::typeof_exclude_type(source_type, typeof_value);

        if negate {
            TypeGuardResult {
                true_type: excluded,
                false_type: narrowed,
            }
        } else {
            TypeGuardResult {
                true_type: narrowed,
                false_type: excluded,
            }
        }
    }

    /// Narrow a type based on typeof check
    fn typeof_narrow_type(source_type: &Type, typeof_value: &str) -> Arc<Type> {
        match source_type {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| Self::type_matches_typeof(t, typeof_value))
                    .cloned()
                    .collect();

                Self::create_union_or_never(filtered)
            }
            _ => {
                if Self::type_matches_typeof(source_type, typeof_value) {
                    Arc::new(source_type.clone())
                } else {
                    // Create a type that matches the typeof
                    Self::type_from_typeof(typeof_value)
                }
            }
        }
    }

    /// Exclude types that match typeof from source type
    fn typeof_exclude_type(source_type: &Type, typeof_value: &str) -> Arc<Type> {
        match source_type {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| !Self::type_matches_typeof(t, typeof_value))
                    .cloned()
                    .collect();

                Self::create_union_or_never(filtered)
            }
            _ => {
                if Self::type_matches_typeof(source_type, typeof_value) {
                    Arc::new(Type::Never)
                } else {
                    Arc::new(source_type.clone())
                }
            }
        }
    }

    /// Check if a type matches a typeof value
    fn type_matches_typeof(t: &Type, typeof_value: &str) -> bool {
        match (t, typeof_value) {
            (Type::String | Type::StringLiteral(_), "string") => true,
            (Type::Number | Type::NumberLiteral(_), "number") => true,
            (Type::Boolean | Type::BooleanLiteral(_), "boolean") => true,
            (Type::BigInt | Type::BigIntLiteral(_), "bigint") => true,
            (Type::Symbol | Type::UniqueSymbol(_), "symbol") => true,
            (Type::Undefined | Type::Void, "undefined") => true,
            (Type::Function(_), "function") => true,
            (Type::Object(_) | Type::Array(_) | Type::Tuple(_) | Type::Null, "object") => true,
            (Type::Any | Type::Unknown, _) => true, // any/unknown matches everything
            _ => false,
        }
    }

    /// Create a type from a typeof value
    fn type_from_typeof(typeof_value: &str) -> Arc<Type> {
        match typeof_value {
            "string" => Arc::new(Type::String),
            "number" => Arc::new(Type::Number),
            "boolean" => Arc::new(Type::Boolean),
            "bigint" => Arc::new(Type::BigInt),
            "symbol" => Arc::new(Type::Symbol),
            "undefined" => Arc::new(Type::Undefined),
            "function" => Arc::new(Type::Function(super::type_defs::FunctionType {
                signatures: Vec::new(),
            })),
            "object" => Arc::new(Type::Union(vec![
                Arc::new(Type::Object(ObjectType::default())),
                Arc::new(Type::Null),
            ])),
            _ => Arc::new(Type::Never), // Unknown typeof value
        }
    }

    /// Apply instanceof type guard
    pub fn narrow_instanceof(
        source_type: &Type,
        constructor_type: &Type,
        negate: bool,
    ) -> TypeGuardResult {
        let narrowed = Self::instanceof_narrow_type(source_type, constructor_type);
        let excluded = Self::instanceof_exclude_type(source_type, constructor_type);

        if negate {
            TypeGuardResult {
                true_type: excluded,
                false_type: narrowed,
            }
        } else {
            TypeGuardResult {
                true_type: narrowed,
                false_type: excluded,
            }
        }
    }

    /// Narrow a type based on instanceof check
    fn instanceof_narrow_type(source_type: &Type, constructor_type: &Type) -> Arc<Type> {
        match source_type {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| Self::could_be_instance_of(t, constructor_type))
                    .cloned()
                    .collect();

                if filtered.is_empty() {
                    // If nothing in union matches, narrow to the constructor type
                    Arc::new(constructor_type.clone())
                } else {
                    Self::create_union_or_never(filtered)
                }
            }
            Type::Any | Type::Unknown => Arc::new(constructor_type.clone()),
            _ => {
                if Self::could_be_instance_of(source_type, constructor_type) {
                    Arc::new(source_type.clone())
                } else {
                    Arc::new(constructor_type.clone())
                }
            }
        }
    }

    /// Exclude types that could be instanceof constructor
    fn instanceof_exclude_type(source_type: &Type, constructor_type: &Type) -> Arc<Type> {
        match source_type {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| !Self::is_definitely_instance_of(t, constructor_type))
                    .cloned()
                    .collect();

                Self::create_union_or_never(filtered)
            }
            _ => {
                if Self::is_definitely_instance_of(source_type, constructor_type) {
                    Arc::new(Type::Never)
                } else {
                    Arc::new(source_type.clone())
                }
            }
        }
    }

    /// Check if a type could be an instance of a constructor
    fn could_be_instance_of(t: &Type, constructor_type: &Type) -> bool {
        match t {
            Type::Any | Type::Unknown => true,
            Type::Object(_) => true, // Objects could be instances
            Type::Class(class) => {
                // Check class hierarchy
                Self::class_extends(class, constructor_type)
            }
            _ => false,
        }
    }

    /// Check if a type is definitely an instance of constructor
    fn is_definitely_instance_of(t: &Type, constructor_type: &Type) -> bool {
        match (t, constructor_type) {
            (Type::Class(source_class), Type::Class(target_class)) => {
                source_class.name == target_class.name || Self::class_extends(source_class, constructor_type)
            }
            _ => false,
        }
    }

    /// Check if a class extends another
    fn class_extends(class: &ClassType, target: &Type) -> bool {
        if let Some(ref extends) = class.extends {
            if let Type::Class(parent) = extends.as_ref() {
                if let Type::Class(target_class) = target {
                    if parent.name == target_class.name {
                        return true;
                    }
                }
                return Self::class_extends(parent, target);
            }
        }
        false
    }

    /// Apply user-defined type guard
    pub fn narrow_user_defined(
        source_type: &Type,
        predicate_type: &Type,
        negate: bool,
    ) -> TypeGuardResult {
        let narrowed = Self::intersect_types(source_type, predicate_type);
        let excluded = Self::exclude_type(source_type, predicate_type);

        if negate {
            TypeGuardResult {
                true_type: excluded,
                false_type: narrowed,
            }
        } else {
            TypeGuardResult {
                true_type: narrowed,
                false_type: excluded,
            }
        }
    }

    /// Apply "in" operator narrowing
    pub fn narrow_in_operator(
        source_type: &Type,
        property_name: &str,
        negate: bool,
    ) -> TypeGuardResult {
        let narrowed = Self::narrow_by_property_presence(source_type, property_name, true);
        let excluded = Self::narrow_by_property_presence(source_type, property_name, false);

        if negate {
            TypeGuardResult {
                true_type: excluded,
                false_type: narrowed,
            }
        } else {
            TypeGuardResult {
                true_type: narrowed,
                false_type: excluded,
            }
        }
    }

    /// Narrow type by property presence
    fn narrow_by_property_presence(
        source_type: &Type,
        property_name: &str,
        must_have: bool,
    ) -> Arc<Type> {
        match source_type {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| {
                        let has_prop = Self::type_has_property(t, property_name);
                        if must_have { has_prop } else { !has_prop }
                    })
                    .cloned()
                    .collect();

                Self::create_union_or_never(filtered)
            }
            Type::Object(obj) => {
                let has_prop = obj.properties.contains_key(property_name);
                if (must_have && has_prop) || (!must_have && !has_prop) {
                    Arc::new(source_type.clone())
                } else if must_have {
                    // Add the property as required
                    let mut new_obj = obj.clone();
                    new_obj.properties.insert(
                        property_name.to_string(),
                        PropertySignature {
                            name: property_name.to_string(),
                            type_: Arc::new(Type::Unknown),
                            optional: false,
                            readonly: false,
                        },
                    );
                    Arc::new(Type::Object(new_obj))
                } else {
                    Arc::new(Type::Never)
                }
            }
            Type::Any => Arc::new(Type::Any),
            Type::Unknown => {
                if must_have {
                    let mut obj = ObjectType::default();
                    obj.properties.insert(
                        property_name.to_string(),
                        PropertySignature {
                            name: property_name.to_string(),
                            type_: Arc::new(Type::Unknown),
                            optional: false,
                            readonly: false,
                        },
                    );
                    Arc::new(Type::Object(obj))
                } else {
                    Arc::new(Type::Unknown)
                }
            }
            _ => {
                if must_have {
                    Arc::new(Type::Never)
                } else {
                    Arc::new(source_type.clone())
                }
            }
        }
    }

    /// Check if a type has a property
    fn type_has_property(t: &Type, property_name: &str) -> bool {
        match t {
            Type::Object(obj) => obj.properties.contains_key(property_name),
            Type::Class(class) => class.members.contains_key(property_name),
            Type::Any | Type::Unknown => true,
            Type::Array(_) => {
                matches!(property_name, "length" | "push" | "pop" | "map" | "filter")
            }
            Type::String | Type::StringLiteral(_) => {
                matches!(property_name, "length" | "charAt" | "split" | "slice")
            }
            _ => false,
        }
    }

    /// Apply strict equality narrowing (===)
    pub fn narrow_strict_equality(
        source_type: &Type,
        compared_type: &Type,
        negate: bool,
    ) -> TypeGuardResult {
        let narrowed = Self::narrow_by_equality(source_type, compared_type);
        let excluded = Self::exclude_by_equality(source_type, compared_type);

        if negate {
            TypeGuardResult {
                true_type: excluded,
                false_type: narrowed,
            }
        } else {
            TypeGuardResult {
                true_type: narrowed,
                false_type: excluded,
            }
        }
    }

    /// Narrow by strict equality
    fn narrow_by_equality(source_type: &Type, compared_type: &Type) -> Arc<Type> {
        match source_type {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| Self::types_could_be_equal(t, compared_type))
                    .cloned()
                    .collect();

                if filtered.is_empty() {
                    Arc::new(compared_type.clone())
                } else {
                    Self::create_union_or_never(filtered)
                }
            }
            _ => {
                if Self::types_could_be_equal(source_type, compared_type) {
                    // Narrow to the more specific type
                    if Self::is_literal_type(compared_type) {
                        Arc::new(compared_type.clone())
                    } else {
                        Arc::new(source_type.clone())
                    }
                } else {
                    Arc::new(Type::Never)
                }
            }
        }
    }

    /// Exclude by strict equality
    fn exclude_by_equality(source_type: &Type, compared_type: &Type) -> Arc<Type> {
        match source_type {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| !Self::types_definitely_equal(t, compared_type))
                    .cloned()
                    .collect();

                Self::create_union_or_never(filtered)
            }
            _ => {
                if Self::types_definitely_equal(source_type, compared_type) {
                    Arc::new(Type::Never)
                } else {
                    Arc::new(source_type.clone())
                }
            }
        }
    }

    /// Check if types could be equal
    fn types_could_be_equal(a: &Type, b: &Type) -> bool {
        match (a, b) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (Type::StringLiteral(s1), Type::StringLiteral(s2)) => s1 == s2,
            (Type::StringLiteral(_), Type::String) | (Type::String, Type::StringLiteral(_)) => true,
            (Type::String, Type::String) => true,
            (Type::NumberLiteral(n1), Type::NumberLiteral(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Type::NumberLiteral(_), Type::Number) | (Type::Number, Type::NumberLiteral(_)) => true,
            (Type::Number, Type::Number) => true,
            (Type::BooleanLiteral(b1), Type::BooleanLiteral(b2)) => b1 == b2,
            (Type::BooleanLiteral(_), Type::Boolean) | (Type::Boolean, Type::BooleanLiteral(_)) => true,
            (Type::Boolean, Type::Boolean) => true,
            (Type::Null, Type::Null) => true,
            (Type::Undefined, Type::Undefined) => true,
            _ => false,
        }
    }

    /// Check if types are definitely equal
    fn types_definitely_equal(a: &Type, b: &Type) -> bool {
        match (a, b) {
            (Type::StringLiteral(s1), Type::StringLiteral(s2)) => s1 == s2,
            (Type::NumberLiteral(n1), Type::NumberLiteral(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Type::BooleanLiteral(b1), Type::BooleanLiteral(b2)) => b1 == b2,
            (Type::Null, Type::Null) => true,
            (Type::Undefined, Type::Undefined) => true,
            _ => false,
        }
    }

    /// Check if type is a literal type
    fn is_literal_type(t: &Type) -> bool {
        matches!(
            t,
            Type::StringLiteral(_)
                | Type::NumberLiteral(_)
                | Type::BooleanLiteral(_)
                | Type::BigIntLiteral(_)
                | Type::Null
                | Type::Undefined
        )
    }

    /// Apply truthiness narrowing
    pub fn narrow_truthiness(source_type: &Type, negate: bool) -> TypeGuardResult {
        let truthy = Self::get_truthy_type(source_type);
        let falsy = Self::get_falsy_type(source_type);

        if negate {
            TypeGuardResult {
                true_type: falsy,
                false_type: truthy,
            }
        } else {
            TypeGuardResult {
                true_type: truthy,
                false_type: falsy,
            }
        }
    }

    /// Get truthy variant of a type
    fn get_truthy_type(source_type: &Type) -> Arc<Type> {
        match source_type {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| !t.is_falsy())
                    .map(|t| Self::get_truthy_type(t))
                    .collect();

                Self::create_union_or_never(filtered)
            }
            Type::Boolean => Arc::new(Type::BooleanLiteral(true)),
            Type::String => Arc::new(Type::String), // Non-empty strings, but keep as string
            Type::Number => Arc::new(Type::Number), // Non-zero numbers, but keep as number
            t if t.is_falsy() => Arc::new(Type::Never),
            _ => Arc::new(source_type.clone()),
        }
    }

    /// Get falsy variant of a type
    fn get_falsy_type(source_type: &Type) -> Arc<Type> {
        match source_type {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| t.is_falsy() || !t.is_definitely_truthy())
                    .cloned()
                    .collect();

                Self::create_union_or_never(filtered)
            }
            Type::Boolean => Arc::new(Type::BooleanLiteral(false)),
            t if t.is_definitely_truthy() => Arc::new(Type::Never),
            _ => Arc::new(source_type.clone()),
        }
    }

    /// Apply non-nullable narrowing (removes null and undefined)
    pub fn narrow_non_nullable(source_type: &Type) -> Arc<Type> {
        match source_type {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| !matches!(t.as_ref(), Type::Null | Type::Undefined))
                    .cloned()
                    .collect();

                Self::create_union_or_never(filtered)
            }
            Type::Null | Type::Undefined => Arc::new(Type::Never),
            _ => Arc::new(source_type.clone()),
        }
    }

    /// Intersect two types
    fn intersect_types(a: &Type, b: &Type) -> Arc<Type> {
        match (a, b) {
            (Type::Any, t) | (t, Type::Any) => Arc::new(t.clone()),
            (Type::Unknown, t) | (t, Type::Unknown) => Arc::new(t.clone()),
            (Type::Never, _) | (_, Type::Never) => Arc::new(Type::Never),
            (Type::Union(types), other) | (other, Type::Union(types)) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter_map(|t| {
                        let intersected = Self::intersect_types(t, other);
                        if matches!(intersected.as_ref(), Type::Never) {
                            None
                        } else {
                            Some(intersected)
                        }
                    })
                    .collect();

                Self::create_union_or_never(filtered)
            }
            // Same primitive types
            (Type::String, Type::String) => Arc::new(Type::String),
            (Type::Number, Type::Number) => Arc::new(Type::Number),
            (Type::Boolean, Type::Boolean) => Arc::new(Type::Boolean),
            (Type::BigInt, Type::BigInt) => Arc::new(Type::BigInt),
            (Type::Symbol, Type::Symbol) => Arc::new(Type::Symbol),
            (Type::Null, Type::Null) => Arc::new(Type::Null),
            (Type::Undefined, Type::Undefined) => Arc::new(Type::Undefined),
            (Type::Void, Type::Void) => Arc::new(Type::Void),
            // Literal narrowing
            (Type::String, Type::StringLiteral(s)) | (Type::StringLiteral(s), Type::String) => {
                Arc::new(Type::StringLiteral(s.clone()))
            }
            (Type::Number, Type::NumberLiteral(n)) | (Type::NumberLiteral(n), Type::Number) => {
                Arc::new(Type::NumberLiteral(*n))
            }
            (Type::Boolean, Type::BooleanLiteral(b)) | (Type::BooleanLiteral(b), Type::Boolean) => {
                Arc::new(Type::BooleanLiteral(*b))
            }
            // Same literal types
            (Type::StringLiteral(s1), Type::StringLiteral(s2)) if s1 == s2 => {
                Arc::new(Type::StringLiteral(s1.clone()))
            }
            (Type::NumberLiteral(n1), Type::NumberLiteral(n2)) if (n1 - n2).abs() < f64::EPSILON => {
                Arc::new(Type::NumberLiteral(*n1))
            }
            (Type::BooleanLiteral(b1), Type::BooleanLiteral(b2)) if b1 == b2 => {
                Arc::new(Type::BooleanLiteral(*b1))
            }
            // Incompatible types
            (Type::String, Type::Number) | (Type::Number, Type::String) => Arc::new(Type::Never),
            (Type::String, Type::Boolean) | (Type::Boolean, Type::String) => Arc::new(Type::Never),
            (Type::Number, Type::Boolean) | (Type::Boolean, Type::Number) => Arc::new(Type::Never),
            (Type::StringLiteral(_), Type::Number | Type::NumberLiteral(_)) => Arc::new(Type::Never),
            (Type::Number | Type::NumberLiteral(_), Type::StringLiteral(_)) => Arc::new(Type::Never),
            (Type::StringLiteral(_), Type::Boolean | Type::BooleanLiteral(_)) => Arc::new(Type::Never),
            (Type::Boolean | Type::BooleanLiteral(_), Type::StringLiteral(_)) => Arc::new(Type::Never),
            (Type::NumberLiteral(_), Type::Boolean | Type::BooleanLiteral(_)) => Arc::new(Type::Never),
            (Type::Boolean | Type::BooleanLiteral(_), Type::NumberLiteral(_)) => Arc::new(Type::Never),
            (Type::Null, _) | (_, Type::Null) => Arc::new(Type::Never),
            (Type::Undefined, _) | (_, Type::Undefined) => Arc::new(Type::Never),
            _ => Arc::new(Type::Intersection(vec![
                Arc::new(a.clone()),
                Arc::new(b.clone()),
            ])),
        }
    }

    /// Exclude one type from another
    fn exclude_type(source: &Type, excluded: &Type) -> Arc<Type> {
        match source {
            Type::Union(types) => {
                let filtered: Vec<Arc<Type>> = types
                    .iter()
                    .filter(|t| !Self::types_overlap(t, excluded))
                    .cloned()
                    .collect();

                Self::create_union_or_never(filtered)
            }
            _ => {
                if Self::types_overlap(source, excluded) {
                    Arc::new(Type::Never)
                } else {
                    Arc::new(source.clone())
                }
            }
        }
    }

    /// Check if two types overlap
    fn types_overlap(a: &Type, b: &Type) -> bool {
        match (a, b) {
            (Type::Any, _) | (_, Type::Any) => true,
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            (Type::Never, _) | (_, Type::Never) => false,
            (Type::String, Type::String | Type::StringLiteral(_)) => true,
            (Type::StringLiteral(_), Type::String) => true,
            (Type::StringLiteral(s1), Type::StringLiteral(s2)) => s1 == s2,
            (Type::Number, Type::Number | Type::NumberLiteral(_)) => true,
            (Type::NumberLiteral(_), Type::Number) => true,
            (Type::NumberLiteral(n1), Type::NumberLiteral(n2)) => (n1 - n2).abs() < f64::EPSILON,
            (Type::Boolean, Type::Boolean | Type::BooleanLiteral(_)) => true,
            (Type::BooleanLiteral(_), Type::Boolean) => true,
            (Type::BooleanLiteral(b1), Type::BooleanLiteral(b2)) => b1 == b2,
            (Type::Null, Type::Null) => true,
            (Type::Undefined, Type::Undefined) => true,
            (Type::Object(_), Type::Object(_)) => true, // Objects may overlap
            _ => false,
        }
    }

    /// Create union or never from types
    fn create_union_or_never(types: Vec<Arc<Type>>) -> Arc<Type> {
        match types.len() {
            0 => Arc::new(Type::Never),
            1 => types.into_iter().next().unwrap(),
            _ => Arc::new(Type::Union(types)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typeof_string_narrowing() {
        let union = Type::Union(vec![
            Arc::new(Type::String),
            Arc::new(Type::Number),
        ]);

        let result = TypeGuardEvaluator::narrow_typeof(&union, "string", false);

        match result.true_type.as_ref() {
            Type::String => (),
            _ => panic!("Expected String type"),
        }

        match result.false_type.as_ref() {
            Type::Number => (),
            _ => panic!("Expected Number type"),
        }
    }

    #[test]
    fn test_typeof_negated() {
        let union = Type::Union(vec![
            Arc::new(Type::String),
            Arc::new(Type::Number),
        ]);

        let result = TypeGuardEvaluator::narrow_typeof(&union, "string", true);

        match result.true_type.as_ref() {
            Type::Number => (),
            _ => panic!("Expected Number type (negated)"),
        }
    }

    #[test]
    fn test_strict_equality_null() {
        let union = Type::Union(vec![
            Arc::new(Type::String),
            Arc::new(Type::Null),
        ]);

        let result = TypeGuardEvaluator::narrow_strict_equality(&union, &Type::Null, false);

        match result.true_type.as_ref() {
            Type::Null => (),
            _ => panic!("Expected Null type"),
        }

        match result.false_type.as_ref() {
            Type::String => (),
            _ => panic!("Expected String type"),
        }
    }

    #[test]
    fn test_in_operator_narrowing() {
        let union = Type::Union(vec![
            Arc::new(Type::Object({
                let mut obj = ObjectType::default();
                obj.properties.insert(
                    "name".to_string(),
                    PropertySignature {
                        name: "name".to_string(),
                        type_: Arc::new(Type::String),
                        optional: false,
                        readonly: false,
                    },
                );
                obj
            })),
            Arc::new(Type::Object(ObjectType::default())),
        ]);

        let result = TypeGuardEvaluator::narrow_in_operator(&union, "name", false);

        // The true type should have the "name" property
        match result.true_type.as_ref() {
            Type::Object(obj) => {
                assert!(obj.properties.contains_key("name"));
            }
            _ => panic!("Expected Object type with 'name' property"),
        }
    }

    #[test]
    fn test_truthiness_narrowing() {
        let union = Type::Union(vec![
            Arc::new(Type::String),
            Arc::new(Type::Null),
            Arc::new(Type::Undefined),
        ]);

        let result = TypeGuardEvaluator::narrow_truthiness(&union, false);

        // True type should exclude null and undefined
        match result.true_type.as_ref() {
            Type::String => (),
            _ => panic!("Expected String type after truthiness narrowing"),
        }
    }

    #[test]
    fn test_non_nullable_narrowing() {
        let union = Type::Union(vec![
            Arc::new(Type::String),
            Arc::new(Type::Null),
            Arc::new(Type::Undefined),
        ]);

        let result = TypeGuardEvaluator::narrow_non_nullable(&union);

        match result.as_ref() {
            Type::String => (),
            _ => panic!("Expected String type after non-nullable narrowing"),
        }
    }

    #[test]
    fn test_literal_equality_narrowing() {
        let union = Type::Union(vec![
            Arc::new(Type::StringLiteral("foo".to_string())),
            Arc::new(Type::StringLiteral("bar".to_string())),
        ]);

        let result = TypeGuardEvaluator::narrow_strict_equality(
            &union,
            &Type::StringLiteral("foo".to_string()),
            false,
        );

        match result.true_type.as_ref() {
            Type::StringLiteral(s) => assert_eq!(s, "foo"),
            _ => panic!("Expected StringLiteral 'foo'"),
        }

        match result.false_type.as_ref() {
            Type::StringLiteral(s) => assert_eq!(s, "bar"),
            _ => panic!("Expected StringLiteral 'bar'"),
        }
    }
}
