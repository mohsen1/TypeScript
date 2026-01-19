//! Union Type Checking
//!
//! Handles union type operations and discriminated union patterns:
//! - Discriminated union detection
//! - Union narrowing through property access
//! - Type guards and narrowing
//! - Union simplification

use crate::types::intersection::{Type, ObjectType, Property};
use std::collections::{HashMap, HashSet};

/// Discriminant property info
#[derive(Debug, Clone, PartialEq)]
pub struct DiscriminantProperty {
    /// Property name (e.g., "kind", "type", "tag")
    pub name: String,
    /// All possible literal values for this discriminant
    pub values: Vec<Type>,
    /// Maps literal values to the corresponding union member index
    pub value_to_member: HashMap<String, usize>,
}

/// Result of discriminated union analysis
#[derive(Debug, Clone)]
pub struct DiscriminatedUnionInfo {
    /// The discriminant properties found
    pub discriminants: Vec<DiscriminantProperty>,
    /// Whether this is a valid discriminated union
    pub is_discriminated: bool,
    /// The union members
    pub members: Vec<Type>,
}

/// Narrowing result
#[derive(Debug, Clone)]
pub struct NarrowingResult {
    /// The narrowed type
    pub narrowed_type: Type,
    /// Union members that were eliminated
    pub eliminated: Vec<Type>,
    /// Union members that remain
    pub remaining: Vec<Type>,
}

/// Union type checker
pub struct UnionChecker {
    /// Common discriminant property names to check first
    common_discriminants: Vec<String>,
}

impl UnionChecker {
    pub fn new() -> Self {
        UnionChecker {
            common_discriminants: vec![
                "kind".to_string(),
                "type".to_string(),
                "tag".to_string(),
                "_tag".to_string(),
                "discriminator".to_string(),
            ],
        }
    }

    /// Analyze a union type for discriminated union patterns
    pub fn analyze_discriminated_union(&self, union: &Type) -> DiscriminatedUnionInfo {
        let members = match union {
            Type::Union(types) => types.clone(),
            _ => {
                return DiscriminatedUnionInfo {
                    discriminants: vec![],
                    is_discriminated: false,
                    members: vec![union.clone()],
                };
            }
        };

        if members.len() < 2 {
            return DiscriminatedUnionInfo {
                discriminants: vec![],
                is_discriminated: false,
                members,
            };
        }

        // Find common properties with literal types across all members
        let discriminants = self.find_discriminant_properties(&members);

        let is_discriminated = !discriminants.is_empty();

        DiscriminatedUnionInfo {
            discriminants,
            is_discriminated,
            members,
        }
    }

    /// Find properties that can serve as discriminants
    fn find_discriminant_properties(&self, members: &[Type]) -> Vec<DiscriminantProperty> {
        let mut discriminants = Vec::new();

        // Get all object types
        let objects: Vec<&ObjectType> = members
            .iter()
            .filter_map(|t| t.as_object())
            .collect();

        // If not all members are objects, can't be discriminated
        if objects.len() != members.len() {
            return discriminants;
        }

        // Find properties that exist in all objects with literal types
        let first_obj = objects[0];

        // Check common discriminant names first, then all properties
        let mut property_names: Vec<&String> = self.common_discriminants
            .iter()
            .filter(|name| first_obj.properties.contains_key(*name))
            .collect();

        // Add other properties
        for name in first_obj.properties.keys() {
            if !self.common_discriminants.contains(name) {
                property_names.push(name);
            }
        }

        for prop_name in property_names {
            // Check if this property exists in all members with literal types
            let mut all_have_literal = true;
            let mut values = Vec::new();
            let mut value_to_member = HashMap::new();

            for (idx, obj) in objects.iter().enumerate() {
                if let Some(prop) = obj.properties.get(prop_name) {
                    if let Some(literal_key) = get_literal_key(&prop.prop_type) {
                        // Check for duplicate discriminant values
                        if value_to_member.contains_key(&literal_key) {
                            all_have_literal = false;
                            break;
                        }
                        values.push(prop.prop_type.clone());
                        value_to_member.insert(literal_key, idx);
                    } else {
                        all_have_literal = false;
                        break;
                    }
                } else {
                    all_have_literal = false;
                    break;
                }
            }

            if all_have_literal && !values.is_empty() {
                discriminants.push(DiscriminantProperty {
                    name: prop_name.clone(),
                    values,
                    value_to_member,
                });
            }
        }

        discriminants
    }

    /// Narrow a union type based on a discriminant check
    pub fn narrow_by_discriminant(
        &self,
        union: &Type,
        property_name: &str,
        value: &Type,
    ) -> NarrowingResult {
        let members = match union {
            Type::Union(types) => types.clone(),
            _ => {
                return NarrowingResult {
                    narrowed_type: union.clone(),
                    eliminated: vec![],
                    remaining: vec![union.clone()],
                };
            }
        };

        let value_key = get_literal_key(value);

        let mut remaining = Vec::new();
        let mut eliminated = Vec::new();

        for member in members {
            if let Type::Object(ref obj) = member {
                if let Some(prop) = obj.properties.get(property_name) {
                    let prop_key = get_literal_key(&prop.prop_type);
                    if prop_key == value_key {
                        remaining.push(member);
                    } else {
                        eliminated.push(member);
                    }
                } else {
                    // Property doesn't exist - could still match if value is undefined
                    if matches!(value, Type::Undefined) {
                        remaining.push(member);
                    } else {
                        eliminated.push(member);
                    }
                }
            } else {
                // Non-object member - can't narrow
                remaining.push(member);
            }
        }

        let narrowed_type = create_union_from_types(remaining.clone());

        NarrowingResult {
            narrowed_type,
            eliminated,
            remaining,
        }
    }

    /// Narrow a union by excluding specific types
    pub fn narrow_by_exclusion(&self, union: &Type, excluded: &[Type]) -> NarrowingResult {
        let members = match union {
            Type::Union(types) => types.clone(),
            _ => {
                if excluded.contains(union) {
                    return NarrowingResult {
                        narrowed_type: Type::Never,
                        eliminated: vec![union.clone()],
                        remaining: vec![],
                    };
                }
                return NarrowingResult {
                    narrowed_type: union.clone(),
                    eliminated: vec![],
                    remaining: vec![union.clone()],
                };
            }
        };

        let mut remaining = Vec::new();
        let mut eliminated = Vec::new();

        for member in members {
            if excluded.contains(&member) {
                eliminated.push(member);
            } else {
                remaining.push(member);
            }
        }

        let narrowed_type = create_union_from_types(remaining.clone());

        NarrowingResult {
            narrowed_type,
            eliminated,
            remaining,
        }
    }

    /// Narrow a union based on typeof check
    pub fn narrow_by_typeof(&self, union: &Type, typeof_value: &str) -> NarrowingResult {
        let members = match union {
            Type::Union(types) => types.clone(),
            _ => vec![union.clone()],
        };

        let mut remaining = Vec::new();
        let mut eliminated = Vec::new();

        for member in members {
            if type_matches_typeof(&member, typeof_value) {
                remaining.push(member);
            } else {
                eliminated.push(member);
            }
        }

        let narrowed_type = create_union_from_types(remaining.clone());

        NarrowingResult {
            narrowed_type,
            eliminated,
            remaining,
        }
    }

    /// Narrow a union based on truthiness check
    pub fn narrow_by_truthiness(&self, union: &Type, is_truthy: bool) -> NarrowingResult {
        let members = match union {
            Type::Union(types) => types.clone(),
            _ => vec![union.clone()],
        };

        let mut remaining = Vec::new();
        let mut eliminated = Vec::new();

        for member in members {
            let is_member_falsy = is_falsy_type(&member);
            if is_truthy {
                // Keep non-falsy types
                if !is_member_falsy {
                    remaining.push(member);
                } else {
                    eliminated.push(member);
                }
            } else {
                // Keep falsy types
                if is_member_falsy {
                    remaining.push(member);
                } else {
                    eliminated.push(member);
                }
            }
        }

        let narrowed_type = create_union_from_types(remaining.clone());

        NarrowingResult {
            narrowed_type,
            eliminated,
            remaining,
        }
    }

    /// Check if a value is a valid discriminant for the union
    pub fn is_valid_discriminant_value(
        &self,
        union_info: &DiscriminatedUnionInfo,
        property_name: &str,
        value: &Type,
    ) -> bool {
        for disc in &union_info.discriminants {
            if disc.name == property_name {
                let value_key = get_literal_key(value);
                return value_key.is_some() && disc.value_to_member.contains_key(&value_key.unwrap());
            }
        }
        false
    }

    /// Get the member type for a specific discriminant value
    pub fn get_member_for_discriminant(
        &self,
        union_info: &DiscriminatedUnionInfo,
        property_name: &str,
        value: &Type,
    ) -> Option<Type> {
        for disc in &union_info.discriminants {
            if disc.name == property_name {
                if let Some(value_key) = get_literal_key(value) {
                    if let Some(&idx) = disc.value_to_member.get(&value_key) {
                        return union_info.members.get(idx).cloned();
                    }
                }
            }
        }
        None
    }

    /// Get all uncovered discriminant values
    pub fn get_uncovered_values(
        &self,
        union_info: &DiscriminatedUnionInfo,
        property_name: &str,
        covered_values: &[Type],
    ) -> Vec<Type> {
        for disc in &union_info.discriminants {
            if disc.name == property_name {
                let covered_keys: HashSet<String> = covered_values
                    .iter()
                    .filter_map(get_literal_key)
                    .collect();

                return disc.values
                    .iter()
                    .filter(|v| {
                        if let Some(key) = get_literal_key(v) {
                            !covered_keys.contains(&key)
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect();
            }
        }
        vec![]
    }
}

impl Default for UnionChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Get a unique string key for a literal type
fn get_literal_key(ty: &Type) -> Option<String> {
    match ty {
        Type::StringLiteral(s) => Some(format!("string:{}", s)),
        Type::NumberLiteral(n) => Some(format!("number:{}", n)),
        Type::BooleanLiteral(b) => Some(format!("boolean:{}", b)),
        Type::Null => Some("null".to_string()),
        Type::Undefined => Some("undefined".to_string()),
        _ => None,
    }
}

/// Check if a type matches a typeof check
fn type_matches_typeof(ty: &Type, typeof_value: &str) -> bool {
    match typeof_value {
        "string" => matches!(ty, Type::String | Type::StringLiteral(_)),
        "number" => matches!(ty, Type::Number | Type::NumberLiteral(_)),
        "boolean" => matches!(ty, Type::Boolean | Type::BooleanLiteral(_)),
        "bigint" => matches!(ty, Type::BigInt),
        "symbol" => matches!(ty, Type::Symbol),
        "undefined" => matches!(ty, Type::Undefined | Type::Void),
        "object" => matches!(ty, Type::Object(_) | Type::Array(_) | Type::Tuple(_) | Type::Null),
        "function" => matches!(ty, Type::Function(_)),
        _ => false,
    }
}

/// Check if a type is a falsy type
fn is_falsy_type(ty: &Type) -> bool {
    match ty {
        Type::Null => true,
        Type::Undefined => true,
        Type::Void => true,
        Type::Never => true,
        Type::BooleanLiteral(false) => true,
        Type::NumberLiteral(n) if *n == 0.0 => true,
        Type::StringLiteral(s) if s.is_empty() => true,
        _ => false,
    }
}

/// Create a union type from a list of types
pub fn create_union_from_types(types: Vec<Type>) -> Type {
    if types.is_empty() {
        return Type::Never;
    }
    if types.len() == 1 {
        return types.into_iter().next().unwrap();
    }
    Type::Union(types)
}

/// Simplify a union type by removing duplicates and handling special cases
pub fn simplify_union(types: Vec<Type>) -> Type {
    if types.is_empty() {
        return Type::Never;
    }

    // Flatten nested unions
    let mut flattened = Vec::new();
    for ty in types {
        match ty {
            Type::Union(inner) => flattened.extend(inner),
            other => flattened.push(other),
        }
    }

    // Remove never types (identity for union)
    let filtered: Vec<Type> = flattened.into_iter().filter(|t| !t.is_never()).collect();

    // Check for any - absorbs everything
    if filtered.iter().any(|t| t.is_any()) {
        return Type::Any;
    }

    // Check for unknown - absorbs everything except any
    if filtered.iter().any(|t| t.is_unknown()) {
        return Type::Unknown;
    }

    // Remove duplicates while preserving order
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();
    for ty in filtered {
        let key = format!("{:?}", ty);
        if !seen.contains(&key) {
            seen.insert(key);
            deduped.push(ty);
        }
    }

    // Simplify literal types with their base types
    let simplified = simplify_literals_in_union(deduped);

    if simplified.is_empty() {
        Type::Never
    } else if simplified.len() == 1 {
        simplified.into_iter().next().unwrap()
    } else {
        Type::Union(simplified)
    }
}

/// Simplify literals when their base type is also in the union
fn simplify_literals_in_union(types: Vec<Type>) -> Vec<Type> {
    let has_string = types.iter().any(|t| matches!(t, Type::String));
    let has_number = types.iter().any(|t| matches!(t, Type::Number));
    let has_boolean = types.iter().any(|t| matches!(t, Type::Boolean));

    types
        .into_iter()
        .filter(|t| {
            match t {
                // Remove string literals if string is present
                Type::StringLiteral(_) if has_string => false,
                // Remove number literals if number is present
                Type::NumberLiteral(_) if has_number => false,
                // Remove boolean literals if boolean is present
                Type::BooleanLiteral(_) if has_boolean => false,
                _ => true,
            }
        })
        .collect()
}

/// Get the common properties across all union members
pub fn get_common_properties(union: &Type) -> HashMap<String, Property> {
    let members = match union {
        Type::Union(types) => types,
        _ => return HashMap::new(),
    };

    let objects: Vec<&ObjectType> = members
        .iter()
        .filter_map(|t| t.as_object())
        .collect();

    if objects.is_empty() || objects.len() != members.len() {
        return HashMap::new();
    }

    // Start with first object's properties
    let mut common = objects[0].properties.clone();

    // Keep only properties that exist in all objects
    for obj in &objects[1..] {
        common.retain(|name, _| obj.properties.contains_key(name));
    }

    // For remaining properties, create union of their types
    for (name, prop) in common.iter_mut() {
        let types: Vec<Type> = objects
            .iter()
            .filter_map(|obj| obj.properties.get(name))
            .map(|p| p.prop_type.clone())
            .collect();

        prop.prop_type = simplify_union(types);
        prop.optional = objects.iter().all(|obj| {
            obj.properties.get(name).map_or(true, |p| p.optional)
        });
    }

    common
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_object(props: Vec<(&str, Type)>) -> Type {
        let mut properties = HashMap::new();
        for (name, ty) in props {
            properties.insert(name.to_string(), Property::new(name, ty));
        }
        Type::Object(ObjectType::with_properties(properties))
    }

    fn make_discriminated_union() -> Type {
        // type A = { kind: "a", value: number }
        // type B = { kind: "b", name: string }
        // type Union = A | B
        let a = make_object(vec![
            ("kind", Type::StringLiteral("a".to_string())),
            ("value", Type::Number),
        ]);
        let b = make_object(vec![
            ("kind", Type::StringLiteral("b".to_string())),
            ("name", Type::String),
        ]);
        Type::Union(vec![a, b])
    }

    #[test]
    fn test_discriminated_union_detection() {
        let checker = UnionChecker::new();
        let union = make_discriminated_union();

        let info = checker.analyze_discriminated_union(&union);

        assert!(info.is_discriminated);
        assert_eq!(info.discriminants.len(), 1);
        assert_eq!(info.discriminants[0].name, "kind");
        assert_eq!(info.discriminants[0].values.len(), 2);
    }

    #[test]
    fn test_non_discriminated_union() {
        let checker = UnionChecker::new();

        // Union without common literal property
        let a = make_object(vec![("x", Type::Number)]);
        let b = make_object(vec![("y", Type::String)]);
        let union = Type::Union(vec![a, b]);

        let info = checker.analyze_discriminated_union(&union);

        assert!(!info.is_discriminated);
    }

    #[test]
    fn test_narrow_by_discriminant() {
        let checker = UnionChecker::new();
        let union = make_discriminated_union();

        let result = checker.narrow_by_discriminant(
            &union,
            "kind",
            &Type::StringLiteral("a".to_string()),
        );

        assert_eq!(result.remaining.len(), 1);
        assert_eq!(result.eliminated.len(), 1);

        if let Type::Object(obj) = &result.narrowed_type {
            assert!(obj.properties.contains_key("value"));
        } else {
            panic!("Expected object type");
        }
    }

    #[test]
    fn test_narrow_by_typeof() {
        let checker = UnionChecker::new();
        let union = Type::Union(vec![Type::String, Type::Number, Type::Boolean]);

        let result = checker.narrow_by_typeof(&union, "string");

        assert_eq!(result.remaining.len(), 1);
        assert_eq!(result.narrowed_type, Type::String);
    }

    #[test]
    fn test_narrow_by_truthiness() {
        let checker = UnionChecker::new();
        let union = Type::Union(vec![
            Type::String,
            Type::Null,
            Type::Undefined,
        ]);

        let result = checker.narrow_by_truthiness(&union, true);

        assert_eq!(result.remaining.len(), 1);
        assert_eq!(result.narrowed_type, Type::String);
    }

    #[test]
    fn test_simplify_union_removes_never() {
        let result = simplify_union(vec![Type::Number, Type::Never, Type::String]);

        if let Type::Union(types) = result {
            assert_eq!(types.len(), 2);
            assert!(!types.contains(&Type::Never));
        } else {
            panic!("Expected union type");
        }
    }

    #[test]
    fn test_simplify_union_any_absorbs() {
        let result = simplify_union(vec![Type::Number, Type::Any, Type::String]);
        assert!(result.is_any());
    }

    #[test]
    fn test_simplify_union_deduplicates() {
        let result = simplify_union(vec![Type::Number, Type::String, Type::Number]);

        if let Type::Union(types) = result {
            assert_eq!(types.len(), 2);
        } else {
            panic!("Expected union type");
        }
    }

    #[test]
    fn test_simplify_union_flattens() {
        let inner = Type::Union(vec![Type::Number, Type::String]);
        let result = simplify_union(vec![inner, Type::Boolean]);

        if let Type::Union(types) = result {
            assert_eq!(types.len(), 3);
        } else {
            panic!("Expected union type");
        }
    }

    #[test]
    fn test_simplify_literals_with_base() {
        // string | "hello" should simplify to string
        let result = simplify_union(vec![
            Type::String,
            Type::StringLiteral("hello".to_string()),
        ]);

        assert_eq!(result, Type::String);
    }

    #[test]
    fn test_get_common_properties() {
        let union = make_discriminated_union();
        let common = get_common_properties(&union);

        // "kind" is common to both
        assert!(common.contains_key("kind"));
        // "value" and "name" are not common
        assert!(!common.contains_key("value"));
        assert!(!common.contains_key("name"));
    }

    #[test]
    fn test_get_member_for_discriminant() {
        let checker = UnionChecker::new();
        let union = make_discriminated_union();
        let info = checker.analyze_discriminated_union(&union);

        let member = checker.get_member_for_discriminant(
            &info,
            "kind",
            &Type::StringLiteral("a".to_string()),
        );

        assert!(member.is_some());
        if let Type::Object(obj) = member.unwrap() {
            assert!(obj.properties.contains_key("value"));
        }
    }

    #[test]
    fn test_get_uncovered_values() {
        let checker = UnionChecker::new();
        let union = make_discriminated_union();
        let info = checker.analyze_discriminated_union(&union);

        // Cover only "a"
        let covered = vec![Type::StringLiteral("a".to_string())];
        let uncovered = checker.get_uncovered_values(&info, "kind", &covered);

        assert_eq!(uncovered.len(), 1);
        assert_eq!(uncovered[0], Type::StringLiteral("b".to_string()));
    }

    #[test]
    fn test_empty_union_is_never() {
        let result = simplify_union(vec![]);
        assert!(result.is_never());
    }

    #[test]
    fn test_single_type_union() {
        let result = simplify_union(vec![Type::Number]);
        assert_eq!(result, Type::Number);
    }
}
