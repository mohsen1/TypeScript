//! Intersection Types
//!
//! Handles TypeScript intersection type representation and manipulation:
//! - Intersection type creation (T & U)
//! - Property merging for object types
//! - Flattening nested intersections
//! - Distribution over union types

use std::collections::HashMap;

/// Type representation for the type system
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Never type - bottom type (empty intersection result)
    Never,
    /// Unknown type - top type
    Unknown,
    /// Any type
    Any,
    /// Void type
    Void,
    /// Undefined type
    Undefined,
    /// Null type
    Null,
    /// Boolean primitive
    Boolean,
    /// Number primitive
    Number,
    /// String primitive
    String,
    /// BigInt primitive
    BigInt,
    /// Symbol primitive
    Symbol,
    /// Boolean literal
    BooleanLiteral(bool),
    /// Number literal
    NumberLiteral(f64),
    /// String literal
    StringLiteral(String),
    /// Object type with properties
    Object(ObjectType),
    /// Array type
    Array(Box<Type>),
    /// Tuple type
    Tuple(Vec<Type>),
    /// Function type
    Function(FunctionType),
    /// Union type (T | U)
    Union(Vec<Type>),
    /// Intersection type (T & U)
    Intersection(Vec<Type>),
    /// Type reference (named type)
    Reference { name: String, type_args: Vec<Type> },
    /// Type parameter
    TypeParameter { name: String, constraint: Option<Box<Type>> },
    /// Conditional type
    Conditional {
        check_type: Box<Type>,
        extends_type: Box<Type>,
        true_type: Box<Type>,
        false_type: Box<Type>,
    },
    /// Indexed access type (T[K])
    IndexedAccess { object_type: Box<Type>, index_type: Box<Type> },
    /// Mapped type
    Mapped {
        type_parameter: String,
        constraint: Box<Type>,
        template: Box<Type>,
        readonly: Option<bool>,
        optional: Option<bool>,
    },
}

/// Object type with properties and index signatures
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectType {
    /// Named properties
    pub properties: HashMap<String, Property>,
    /// Call signatures (for callable objects)
    pub call_signatures: Vec<FunctionType>,
    /// Construct signatures (for newable objects)
    pub construct_signatures: Vec<FunctionType>,
    /// String index signature: [key: string]: T
    pub string_index: Option<Box<Type>>,
    /// Number index signature: [key: number]: T
    pub number_index: Option<Box<Type>>,
}

impl ObjectType {
    pub fn new() -> Self {
        ObjectType {
            properties: HashMap::new(),
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            string_index: None,
            number_index: None,
        }
    }

    pub fn with_properties(properties: HashMap<String, Property>) -> Self {
        ObjectType {
            properties,
            call_signatures: Vec::new(),
            construct_signatures: Vec::new(),
            string_index: None,
            number_index: None,
        }
    }
}

impl Default for ObjectType {
    fn default() -> Self {
        Self::new()
    }
}

/// Object property
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// Property name
    pub name: String,
    /// Property type
    pub prop_type: Type,
    /// Is optional (?)
    pub optional: bool,
    /// Is readonly
    pub readonly: bool,
}

impl Property {
    pub fn new(name: impl Into<String>, prop_type: Type) -> Self {
        Property {
            name: name.into(),
            prop_type,
            optional: false,
            readonly: false,
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }
}

/// Function type
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    /// Type parameters
    pub type_params: Vec<TypeParameter>,
    /// Parameters
    pub params: Vec<Parameter>,
    /// Return type
    pub return_type: Box<Type>,
    /// Is this type from a rest parameter?
    pub rest_param: bool,
}

/// Type parameter declaration
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParameter {
    pub name: String,
    pub constraint: Option<Box<Type>>,
    pub default: Option<Box<Type>>,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
    pub optional: bool,
}

impl Type {
    /// Check if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Type::Boolean
                | Type::Number
                | Type::String
                | Type::BigInt
                | Type::Symbol
                | Type::BooleanLiteral(_)
                | Type::NumberLiteral(_)
                | Type::StringLiteral(_)
        )
    }

    /// Check if this is the never type
    pub fn is_never(&self) -> bool {
        matches!(self, Type::Never)
    }

    /// Check if this is the any type
    pub fn is_any(&self) -> bool {
        matches!(self, Type::Any)
    }

    /// Check if this is the unknown type
    pub fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }

    /// Check if this is an object type
    pub fn is_object(&self) -> bool {
        matches!(self, Type::Object(_))
    }

    /// Check if this is a union type
    pub fn is_union(&self) -> bool {
        matches!(self, Type::Union(_))
    }

    /// Check if this is an intersection type
    pub fn is_intersection(&self) -> bool {
        matches!(self, Type::Intersection(_))
    }

    /// Get as object type if this is one
    pub fn as_object(&self) -> Option<&ObjectType> {
        match self {
            Type::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get as union types if this is one
    pub fn as_union(&self) -> Option<&Vec<Type>> {
        match self {
            Type::Union(types) => Some(types),
            _ => None,
        }
    }

    /// Get as intersection types if this is one
    pub fn as_intersection(&self) -> Option<&Vec<Type>> {
        match self {
            Type::Intersection(types) => Some(types),
            _ => None,
        }
    }
}

/// Create an intersection type from multiple types
pub fn create_intersection(types: Vec<Type>) -> Type {
    if types.is_empty() {
        return Type::Unknown; // Empty intersection is unknown (top type)
    }
    if types.len() == 1 {
        return types.into_iter().next().unwrap();
    }

    // Flatten nested intersections and simplify
    let flattened = flatten_intersection(types);

    // Check for never reduction
    if flattened.iter().any(|t| t.is_never()) {
        return Type::Never;
    }

    // Check for any/unknown absorption
    // any & T = any (absorbs everything except never)
    if flattened.iter().any(|t| t.is_any()) {
        return Type::Any;
    }

    // Remove duplicates while preserving order
    let deduped = deduplicate_types(flattened);

    if deduped.len() == 1 {
        return deduped.into_iter().next().unwrap();
    }

    // Check for primitive intersections (result in never)
    if has_incompatible_primitives(&deduped) {
        return Type::Never;
    }

    // Try to merge object types
    let (objects, others): (Vec<_>, Vec<_>) = deduped
        .into_iter()
        .partition(|t| matches!(t, Type::Object(_)));

    if objects.len() > 1 {
        // Merge object types
        let merged_obj = merge_object_types(
            objects
                .into_iter()
                .filter_map(|t| match t {
                    Type::Object(obj) => Some(obj),
                    _ => None,
                })
                .collect(),
        );

        if others.is_empty() {
            return Type::Object(merged_obj);
        }

        let mut result = others;
        result.push(Type::Object(merged_obj));

        if result.len() == 1 {
            return result.into_iter().next().unwrap();
        }

        return Type::Intersection(result);
    }

    let all_types: Vec<_> = objects.into_iter().chain(others).collect();

    if all_types.len() == 1 {
        return all_types.into_iter().next().unwrap();
    }

    Type::Intersection(all_types)
}

/// Flatten nested intersection types
pub fn flatten_intersection(types: Vec<Type>) -> Vec<Type> {
    let mut result = Vec::new();

    for ty in types {
        match ty {
            Type::Intersection(inner) => {
                // Recursively flatten
                result.extend(flatten_intersection(inner));
            }
            other => {
                result.push(other);
            }
        }
    }

    result
}

/// Check if there are incompatible primitive types
fn has_incompatible_primitives(types: &[Type]) -> bool {
    let primitives: Vec<_> = types.iter().filter(|t| t.is_primitive()).collect();

    if primitives.len() < 2 {
        return false;
    }

    // Check for incompatible primitives
    for i in 0..primitives.len() {
        for j in (i + 1)..primitives.len() {
            if !are_compatible_primitives(primitives[i], primitives[j]) {
                return true;
            }
        }
    }

    false
}

/// Check if two primitive types can be intersected without resulting in never
fn are_compatible_primitives(a: &Type, b: &Type) -> bool {
    match (a, b) {
        // Same type is compatible
        (Type::Boolean, Type::Boolean) => true,
        (Type::Number, Type::Number) => true,
        (Type::String, Type::String) => true,
        (Type::BigInt, Type::BigInt) => true,
        (Type::Symbol, Type::Symbol) => true,

        // Literal types are compatible with their base types
        (Type::BooleanLiteral(_), Type::Boolean) => true,
        (Type::Boolean, Type::BooleanLiteral(_)) => true,
        (Type::NumberLiteral(_), Type::Number) => true,
        (Type::Number, Type::NumberLiteral(_)) => true,
        (Type::StringLiteral(_), Type::String) => true,
        (Type::String, Type::StringLiteral(_)) => true,

        // Same literals are compatible
        (Type::BooleanLiteral(a), Type::BooleanLiteral(b)) => a == b,
        (Type::NumberLiteral(a), Type::NumberLiteral(b)) => (a - b).abs() < f64::EPSILON,
        (Type::StringLiteral(a), Type::StringLiteral(b)) => a == b,

        // Different primitive types are incompatible
        _ => false,
    }
}

/// Remove duplicate types from a list
fn deduplicate_types(types: Vec<Type>) -> Vec<Type> {
    let mut result = Vec::new();

    for ty in types {
        if !result.contains(&ty) {
            result.push(ty);
        }
    }

    result
}

/// Merge multiple object types into one
pub fn merge_object_types(objects: Vec<ObjectType>) -> ObjectType {
    let mut result = ObjectType::new();

    for obj in objects {
        // Merge properties
        for (name, prop) in obj.properties {
            if let Some(existing) = result.properties.get_mut(&name) {
                // Property exists - intersect the types
                existing.prop_type = create_intersection(vec![
                    existing.prop_type.clone(),
                    prop.prop_type,
                ]);
                // Optional only if both are optional
                existing.optional = existing.optional && prop.optional;
                // Readonly if either is readonly
                existing.readonly = existing.readonly || prop.readonly;
            } else {
                result.properties.insert(name, prop);
            }
        }

        // Merge call signatures
        result.call_signatures.extend(obj.call_signatures);

        // Merge construct signatures
        result.construct_signatures.extend(obj.construct_signatures);

        // Intersect index signatures
        if let Some(string_idx) = obj.string_index {
            result.string_index = Some(match result.string_index {
                Some(existing) => Box::new(create_intersection(vec![*existing, *string_idx])),
                None => string_idx,
            });
        }

        if let Some(number_idx) = obj.number_index {
            result.number_index = Some(match result.number_index {
                Some(existing) => Box::new(create_intersection(vec![*existing, *number_idx])),
                None => number_idx,
            });
        }
    }

    result
}

/// Distribute intersection over union types
/// (A | B) & C = (A & C) | (B & C)
pub fn distribute_intersection_over_union(types: Vec<Type>) -> Type {
    // Find the first union type
    let union_idx = types.iter().position(|t| matches!(t, Type::Union(_)));

    match union_idx {
        Some(idx) => {
            let union_types = match &types[idx] {
                Type::Union(u) => u.clone(),
                _ => unreachable!(),
            };

            // Get all non-union types
            let others: Vec<Type> = types
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != idx)
                .map(|(_, t)| t.clone())
                .collect();

            // Distribute: for each union member, create intersection with others
            let distributed: Vec<Type> = union_types
                .into_iter()
                .map(|u| {
                    let mut intersection_types = others.clone();
                    intersection_types.push(u);
                    create_intersection(intersection_types)
                })
                .collect();

            // Create union of the distributed intersections
            if distributed.len() == 1 {
                distributed.into_iter().next().unwrap()
            } else {
                // Recursively distribute if there are more unions
                let union = Type::Union(distributed);
                // Check if result contains more unions to distribute
                if let Type::Union(ref inner) = union {
                    if inner.iter().any(|t| {
                        if let Type::Intersection(i) = t {
                            i.iter().any(|it| matches!(it, Type::Union(_)))
                        } else {
                            false
                        }
                    }) {
                        // More distribution needed - recurse
                        return simplify_union(inner.clone());
                    }
                }
                union
            }
        }
        None => create_intersection(types),
    }
}

/// Simplify a union type
fn simplify_union(types: Vec<Type>) -> Type {
    if types.is_empty() {
        return Type::Never;
    }
    if types.len() == 1 {
        return types.into_iter().next().unwrap();
    }

    // Flatten nested unions
    let mut flattened = Vec::new();
    for ty in types {
        match ty {
            Type::Union(inner) => flattened.extend(inner),
            other => flattened.push(other),
        }
    }

    // Remove duplicates
    let deduped = deduplicate_types(flattened);

    // Remove never (identity for union)
    let filtered: Vec<_> = deduped.into_iter().filter(|t| !t.is_never()).collect();

    if filtered.is_empty() {
        return Type::Never;
    }
    if filtered.len() == 1 {
        return filtered.into_iter().next().unwrap();
    }

    // any | T = any
    if filtered.iter().any(|t| t.is_any()) {
        return Type::Any;
    }

    // unknown | T = unknown
    if filtered.iter().any(|t| t.is_unknown()) {
        return Type::Unknown;
    }

    Type::Union(filtered)
}

/// Get all properties from an intersection type
pub fn get_intersection_properties(intersection: &Type) -> Option<HashMap<String, Property>> {
    match intersection {
        Type::Intersection(types) => {
            let objects: Vec<ObjectType> = types
                .iter()
                .filter_map(|t| match t {
                    Type::Object(obj) => Some(obj.clone()),
                    _ => None,
                })
                .collect();

            if objects.is_empty() {
                return None;
            }

            let merged = merge_object_types(objects);
            Some(merged.properties)
        }
        Type::Object(obj) => Some(obj.properties.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_simple_intersection() {
        let obj1 = Type::Object(ObjectType::with_properties(
            [("a".to_string(), Property::new("a", Type::Number))].into_iter().collect()
        ));
        let obj2 = Type::Object(ObjectType::with_properties(
            [("b".to_string(), Property::new("b", Type::String))].into_iter().collect()
        ));

        let intersection = create_intersection(vec![obj1, obj2]);

        match intersection {
            Type::Object(obj) => {
                assert!(obj.properties.contains_key("a"));
                assert!(obj.properties.contains_key("b"));
            }
            _ => panic!("Expected object type"),
        }
    }

    #[test]
    fn test_intersection_with_never() {
        let result = create_intersection(vec![Type::Number, Type::Never]);
        assert!(result.is_never());
    }

    #[test]
    fn test_intersection_incompatible_primitives() {
        // number & string = never
        let result = create_intersection(vec![Type::Number, Type::String]);
        assert!(result.is_never());
    }

    #[test]
    fn test_intersection_compatible_literal() {
        // number & 42 = 42 (literal is more specific)
        let result = create_intersection(vec![Type::Number, Type::NumberLiteral(42.0)]);
        // Should not be never - literal is compatible with base type
        assert!(!result.is_never());
    }

    #[test]
    fn test_flatten_nested_intersection() {
        let inner = Type::Intersection(vec![Type::Number, Type::Boolean]);
        let outer = vec![inner, Type::String];
        let flattened = flatten_intersection(outer);

        assert_eq!(flattened.len(), 3);
        assert!(flattened.contains(&Type::Number));
        assert!(flattened.contains(&Type::Boolean));
        assert!(flattened.contains(&Type::String));
    }

    #[test]
    fn test_merge_object_types() {
        let obj1 = ObjectType::with_properties(
            [("a".to_string(), Property::new("a", Type::Number))].into_iter().collect()
        );
        let obj2 = ObjectType::with_properties(
            [("b".to_string(), Property::new("b", Type::String))].into_iter().collect()
        );

        let merged = merge_object_types(vec![obj1, obj2]);

        assert!(merged.properties.contains_key("a"));
        assert!(merged.properties.contains_key("b"));
    }

    #[test]
    fn test_merge_overlapping_properties() {
        let obj1 = ObjectType::with_properties(
            [("x".to_string(), Property::new("x", Type::Number).optional())].into_iter().collect()
        );
        let obj2 = ObjectType::with_properties(
            [("x".to_string(), Property::new("x", Type::Number))].into_iter().collect()
        );

        let merged = merge_object_types(vec![obj1, obj2]);

        let prop = merged.properties.get("x").unwrap();
        // Optional only if both are optional
        assert!(!prop.optional);
    }

    #[test]
    fn test_empty_intersection() {
        let result = create_intersection(vec![]);
        assert!(result.is_unknown());
    }

    #[test]
    fn test_single_type_intersection() {
        let result = create_intersection(vec![Type::Number]);
        assert_eq!(result, Type::Number);
    }

    #[test]
    fn test_intersection_with_any() {
        // any & T = any
        let result = create_intersection(vec![Type::Any, Type::Number]);
        assert!(result.is_any());
    }

    #[test]
    fn test_deduplicate_types() {
        let types = vec![Type::Number, Type::String, Type::Number];
        let deduped = deduplicate_types(types);
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_distribute_intersection_over_union() {
        // (A | B) & C = (A & C) | (B & C)
        let union = Type::Union(vec![Type::Number, Type::String]);
        let obj = Type::Object(ObjectType::with_properties(
            [("c".to_string(), Property::new("c", Type::Boolean))].into_iter().collect()
        ));

        let result = distribute_intersection_over_union(vec![union, obj]);

        match result {
            Type::Union(types) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_string_literal_intersection() {
        // "hello" & string = "hello"
        let result = create_intersection(vec![
            Type::StringLiteral("hello".to_string()),
            Type::String,
        ]);
        assert!(!result.is_never());
    }

    #[test]
    fn test_incompatible_string_literals() {
        // "hello" & "world" = never
        let result = create_intersection(vec![
            Type::StringLiteral("hello".to_string()),
            Type::StringLiteral("world".to_string()),
        ]);
        assert!(result.is_never());
    }
}
