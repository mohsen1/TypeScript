//! Intersection Type Solver
//!
//! Handles intersection type creation, simplification, and resolution:
//! - Creating intersection types
//! - Simplifying/reducing intersections
//! - Resolving intersection members
//! - Handling intersection with union types (distribution)

use crate::types::intersection::{
    Type, ObjectType, Property, FunctionType, Parameter,
    flatten_intersection, merge_object_types,
    distribute_intersection_over_union,
};

/// Intersection solver for type computations
pub struct IntersectionSolver {
    /// Maximum recursion depth for type operations
    max_depth: usize,
    /// Current recursion depth
    current_depth: usize,
}

impl IntersectionSolver {
    pub fn new() -> Self {
        IntersectionSolver {
            max_depth: 100,
            current_depth: 0,
        }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        IntersectionSolver {
            max_depth,
            current_depth: 0,
        }
    }

    /// Create an intersection type with full simplification
    pub fn intersect(&mut self, types: Vec<Type>) -> Type {
        if self.current_depth >= self.max_depth {
            // Prevent infinite recursion
            return Type::Intersection(types);
        }
        self.current_depth += 1;

        let result = self.simplify_intersection(types);

        self.current_depth -= 1;
        result
    }

    /// Simplify an intersection type
    fn simplify_intersection(&mut self, types: Vec<Type>) -> Type {
        if types.is_empty() {
            return Type::Unknown;
        }
        if types.len() == 1 {
            return types.into_iter().next().unwrap();
        }

        // Step 1: Flatten nested intersections
        let flattened = flatten_intersection(types);

        // Step 2: Check for immediate reduction cases
        if flattened.iter().any(|t| t.is_never()) {
            return Type::Never;
        }

        // Step 3: Handle any type (absorbs everything except never)
        if flattened.iter().any(|t| t.is_any()) {
            return Type::Any;
        }

        // Step 4: Check for union types that need distribution
        if flattened.iter().any(|t| t.is_union()) {
            return distribute_intersection_over_union(flattened);
        }

        // Step 5: Simplify primitives and literals
        let simplified = self.simplify_primitive_intersection(flattened);
        if simplified.is_never() || simplified.is_any() {
            return simplified;
        }

        // Step 6: Merge object types
        if let Type::Intersection(types) = simplified {
            return self.merge_intersection_objects(types);
        }

        simplified
    }

    /// Simplify intersection of primitive types
    fn simplify_primitive_intersection(&self, types: Vec<Type>) -> Type {
        let mut primitives = Vec::new();
        let mut literals = Vec::new();
        let mut others = Vec::new();

        for ty in types {
            match &ty {
                Type::Boolean | Type::Number | Type::String | Type::BigInt | Type::Symbol => {
                    primitives.push(ty);
                }
                Type::BooleanLiteral(_) | Type::NumberLiteral(_) | Type::StringLiteral(_) => {
                    literals.push(ty);
                }
                _ => {
                    others.push(ty);
                }
            }
        }

        // Check for incompatible primitives
        if primitives.len() > 1 {
            // Multiple different primitive types = never
            let first = &primitives[0];
            for prim in &primitives[1..] {
                if !self.are_same_primitive_kind(first, prim) {
                    return Type::Never;
                }
            }
        }

        // Check for literal incompatibility with primitives
        for literal in &literals {
            for prim in &primitives {
                if !self.literal_compatible_with_primitive(literal, prim) {
                    return Type::Never;
                }
            }
        }

        // Check for incompatible literals
        if literals.len() > 1 {
            // Check all literals are the same
            let first = &literals[0];
            for lit in &literals[1..] {
                if first != lit {
                    return Type::Never;
                }
            }
        }

        // Build result
        let mut result = Vec::new();

        // If we have literals, they take precedence over base types
        if !literals.is_empty() {
            result.extend(literals);
        } else {
            result.extend(primitives);
        }
        result.extend(others);

        if result.is_empty() {
            Type::Unknown
        } else if result.len() == 1 {
            result.into_iter().next().unwrap()
        } else {
            Type::Intersection(result)
        }
    }

    /// Check if two types are the same kind of primitive
    fn are_same_primitive_kind(&self, a: &Type, b: &Type) -> bool {
        matches!(
            (a, b),
            (Type::Boolean, Type::Boolean)
                | (Type::Number, Type::Number)
                | (Type::String, Type::String)
                | (Type::BigInt, Type::BigInt)
                | (Type::Symbol, Type::Symbol)
        )
    }

    /// Check if a literal type is compatible with a primitive type
    fn literal_compatible_with_primitive(&self, literal: &Type, primitive: &Type) -> bool {
        matches!(
            (literal, primitive),
            (Type::BooleanLiteral(_), Type::Boolean)
                | (Type::NumberLiteral(_), Type::Number)
                | (Type::StringLiteral(_), Type::String)
        )
    }

    /// Merge object types in an intersection
    fn merge_intersection_objects(&mut self, types: Vec<Type>) -> Type {
        let (objects, others): (Vec<_>, Vec<_>) = types
            .into_iter()
            .partition(|t| matches!(t, Type::Object(_)));

        if objects.len() <= 1 {
            // Nothing to merge
            let mut result: Vec<Type> = objects;
            result.extend(others);
            if result.len() == 1 {
                return result.into_iter().next().unwrap();
            }
            return Type::Intersection(result);
        }

        // Extract ObjectTypes and merge them
        let object_types: Vec<ObjectType> = objects
            .into_iter()
            .filter_map(|t| match t {
                Type::Object(obj) => Some(obj),
                _ => None,
            })
            .collect();

        let merged = merge_object_types(object_types);

        // Check if merged object has any never-typed properties
        for prop in merged.properties.values() {
            if prop.prop_type.is_never() {
                return Type::Never;
            }
        }

        if others.is_empty() {
            Type::Object(merged)
        } else {
            let mut result = vec![Type::Object(merged)];
            result.extend(others);
            Type::Intersection(result)
        }
    }

    /// Resolve an intersection type to its simplified form
    pub fn resolve(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Intersection(types) => self.intersect(types.clone()),
            Type::Union(types) => {
                // Resolve each union member
                let resolved: Vec<Type> = types.iter().map(|t| self.resolve(t)).collect();
                self.simplify_union(resolved)
            }
            Type::Object(obj) => {
                // Resolve property types
                let mut resolved_props = obj.properties.clone();
                for prop in resolved_props.values_mut() {
                    prop.prop_type = self.resolve(&prop.prop_type);
                }
                Type::Object(ObjectType {
                    properties: resolved_props,
                    call_signatures: obj.call_signatures.clone(),
                    construct_signatures: obj.construct_signatures.clone(),
                    string_index: obj.string_index.as_ref().map(|t| Box::new(self.resolve(t))),
                    number_index: obj.number_index.as_ref().map(|t| Box::new(self.resolve(t))),
                })
            }
            Type::Array(elem) => Type::Array(Box::new(self.resolve(elem))),
            Type::Tuple(elems) => {
                Type::Tuple(elems.iter().map(|t| self.resolve(t)).collect())
            }
            Type::Function(func) => {
                Type::Function(FunctionType {
                    type_params: func.type_params.clone(),
                    params: func.params.iter().map(|p| Parameter {
                        name: p.name.clone(),
                        param_type: self.resolve(&p.param_type),
                        optional: p.optional,
                    }).collect(),
                    return_type: Box::new(self.resolve(&func.return_type)),
                    rest_param: func.rest_param,
                })
            }
            other => other.clone(),
        }
    }

    /// Simplify a union type
    fn simplify_union(&self, types: Vec<Type>) -> Type {
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

        // Remove never types (identity for union)
        let filtered: Vec<_> = flattened.into_iter().filter(|t| !t.is_never()).collect();

        if filtered.is_empty() {
            return Type::Never;
        }
        if filtered.len() == 1 {
            return filtered.into_iter().next().unwrap();
        }

        // any absorbs everything in a union
        if filtered.iter().any(|t| t.is_any()) {
            return Type::Any;
        }

        // unknown absorbs everything except any
        if filtered.iter().any(|t| t.is_unknown()) {
            return Type::Unknown;
        }

        // Remove duplicates
        let mut deduped = Vec::new();
        for ty in filtered {
            if !deduped.contains(&ty) {
                deduped.push(ty);
            }
        }

        if deduped.len() == 1 {
            deduped.into_iter().next().unwrap()
        } else {
            Type::Union(deduped)
        }
    }

    /// Check if an intersection reduces to never
    pub fn reduces_to_never(&mut self, types: &[Type]) -> bool {
        let result = self.intersect(types.to_vec());
        result.is_never()
    }

    /// Get the apparent members of an intersection type
    pub fn get_apparent_properties(&mut self, ty: &Type) -> Vec<Property> {
        let resolved = self.resolve(ty);

        match resolved {
            Type::Object(obj) => obj.properties.values().cloned().collect(),
            Type::Intersection(types) => {
                let merged = self.intersect(types);
                if let Type::Object(obj) = merged {
                    obj.properties.values().cloned().collect()
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    }
}

impl Default for IntersectionSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a simple intersection type
pub fn intersect_types(a: Type, b: Type) -> Type {
    let mut solver = IntersectionSolver::new();
    solver.intersect(vec![a, b])
}

/// Create an intersection from multiple types
pub fn intersect_all(types: Vec<Type>) -> Type {
    let mut solver = IntersectionSolver::new();
    solver.intersect(types)
}

/// Check if intersection simplifies to never
pub fn is_empty_intersection(types: &[Type]) -> bool {
    let mut solver = IntersectionSolver::new();
    solver.reduces_to_never(types)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_object(props: Vec<(&str, Type)>) -> Type {
        let mut properties = HashMap::new();
        for (name, ty) in props {
            properties.insert(name.to_string(), Property::new(name, ty));
        }
        Type::Object(ObjectType::with_properties(properties))
    }

    #[test]
    fn test_intersect_simple_objects() {
        let obj1 = make_object(vec![("a", Type::Number)]);
        let obj2 = make_object(vec![("b", Type::String)]);

        let result = intersect_types(obj1, obj2);

        match result {
            Type::Object(obj) => {
                assert!(obj.properties.contains_key("a"));
                assert!(obj.properties.contains_key("b"));
            }
            _ => panic!("Expected object type"),
        }
    }

    #[test]
    fn test_intersect_incompatible_primitives() {
        let result = intersect_types(Type::Number, Type::String);
        assert!(result.is_never());
    }

    #[test]
    fn test_intersect_literal_with_base() {
        let result = intersect_types(Type::NumberLiteral(42.0), Type::Number);
        assert!(!result.is_never());
        // Should be the literal type (more specific)
        match result {
            Type::NumberLiteral(n) => assert!((n - 42.0).abs() < f64::EPSILON),
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_intersect_incompatible_literals() {
        let result = intersect_types(
            Type::NumberLiteral(1.0),
            Type::NumberLiteral(2.0),
        );
        assert!(result.is_never());
    }

    #[test]
    fn test_intersect_with_never() {
        let result = intersect_types(Type::Number, Type::Never);
        assert!(result.is_never());
    }

    #[test]
    fn test_intersect_with_any() {
        let result = intersect_types(Type::Number, Type::Any);
        assert!(result.is_any());
    }

    #[test]
    fn test_intersect_nested() {
        let inner = Type::Intersection(vec![
            make_object(vec![("a", Type::Number)]),
            make_object(vec![("b", Type::String)]),
        ]);
        let outer = make_object(vec![("c", Type::Boolean)]);

        let result = intersect_types(inner, outer);

        match result {
            Type::Object(obj) => {
                assert!(obj.properties.contains_key("a"));
                assert!(obj.properties.contains_key("b"));
                assert!(obj.properties.contains_key("c"));
            }
            _ => panic!("Expected object type"),
        }
    }

    #[test]
    fn test_intersect_with_union_distribution() {
        let union = Type::Union(vec![Type::Number, Type::String]);
        let obj = make_object(vec![("x", Type::Boolean)]);

        let result = intersect_types(union, obj);

        match result {
            Type::Union(types) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_is_empty_intersection() {
        assert!(is_empty_intersection(&[Type::Number, Type::String]));
        assert!(!is_empty_intersection(&[Type::Number, Type::NumberLiteral(42.0)]));
    }

    #[test]
    fn test_intersect_overlapping_properties() {
        let obj1 = make_object(vec![("x", Type::Number)]);
        let obj2 = make_object(vec![("x", Type::Number)]);

        let result = intersect_types(obj1, obj2);

        match result {
            Type::Object(obj) => {
                assert!(obj.properties.contains_key("x"));
                assert_eq!(obj.properties.get("x").unwrap().prop_type, Type::Number);
            }
            _ => panic!("Expected object type"),
        }
    }

    #[test]
    fn test_intersect_conflicting_properties() {
        // { x: number } & { x: string } should have x: never
        let obj1 = make_object(vec![("x", Type::Number)]);
        let obj2 = make_object(vec![("x", Type::String)]);

        let result = intersect_types(obj1, obj2);

        // The intersection of number & string is never
        assert!(result.is_never());
    }

    #[test]
    fn test_resolve_nested_intersection() {
        let mut solver = IntersectionSolver::new();

        let nested = Type::Intersection(vec![
            Type::Intersection(vec![
                make_object(vec![("a", Type::Number)]),
                make_object(vec![("b", Type::String)]),
            ]),
            make_object(vec![("c", Type::Boolean)]),
        ]);

        let resolved = solver.resolve(&nested);

        match resolved {
            Type::Object(obj) => {
                assert!(obj.properties.contains_key("a"));
                assert!(obj.properties.contains_key("b"));
                assert!(obj.properties.contains_key("c"));
            }
            _ => panic!("Expected object type"),
        }
    }

    #[test]
    fn test_get_apparent_properties() {
        let mut solver = IntersectionSolver::new();

        let intersection = Type::Intersection(vec![
            make_object(vec![("a", Type::Number)]),
            make_object(vec![("b", Type::String)]),
        ]);

        let props = solver.get_apparent_properties(&intersection);
        assert_eq!(props.len(), 2);
    }

    #[test]
    fn test_intersect_all() {
        let result = intersect_all(vec![
            make_object(vec![("a", Type::Number)]),
            make_object(vec![("b", Type::String)]),
            make_object(vec![("c", Type::Boolean)]),
        ]);

        match result {
            Type::Object(obj) => {
                assert_eq!(obj.properties.len(), 3);
            }
            _ => panic!("Expected object type"),
        }
    }

    #[test]
    fn test_empty_intersection_is_unknown() {
        let result = intersect_all(vec![]);
        assert!(result.is_unknown());
    }

    #[test]
    fn test_single_type_intersection() {
        let result = intersect_all(vec![Type::Number]);
        assert_eq!(result, Type::Number);
    }
}
