//! Conditional type resolution.
//!
//! This module implements TypeScript's conditional types:
//! `T extends U ? X : Y`
//!
//! Features:
//! - Basic conditional type evaluation
//! - Distribution over union types
//! - Infer type extraction

use std::collections::HashMap;

use super::{
    generics::{GenericInstantiator, TypeSubstitution},
    PrimitiveKind, Type,
};

/// Conditional type representation.
#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalType {
    /// The type being checked (T in `T extends U ? X : Y`)
    pub check_type: Type,
    /// The type to extend (U in `T extends U ? X : Y`)
    pub extends_type: Type,
    /// The type if the condition is true (X in `T extends U ? X : Y`)
    pub true_type: Type,
    /// The type if the condition is false (Y in `T extends U ? X : Y`)
    pub false_type: Type,
}

impl ConditionalType {
    /// Create a new conditional type.
    pub fn new(check_type: Type, extends_type: Type, true_type: Type, false_type: Type) -> Self {
        Self {
            check_type,
            extends_type,
            true_type,
            false_type,
        }
    }
}

/// Conditional type resolver.
pub struct ConditionalResolver<F>
where
    F: Fn(&Type, &Type) -> bool,
{
    /// Function to check if a type extends another.
    is_subtype: F,
}

impl<F> ConditionalResolver<F>
where
    F: Fn(&Type, &Type) -> bool,
{
    /// Create a new resolver with the given subtype checker.
    pub fn new(is_subtype: F) -> Self {
        Self { is_subtype }
    }

    /// Resolve a conditional type to its result type.
    pub fn resolve(&self, cond: &ConditionalType) -> Type {
        // Check if check_type is a union - if so, distribute
        if let Type::Union(members) = &cond.check_type {
            return self.resolve_distributed(members, cond);
        }

        // Check if check_type is a type parameter - if so, defer
        if let Type::TypeParameter(_) = &cond.check_type {
            // Cannot resolve yet, return the conditional as-is
            return Type::Conditional(Box::new(cond.clone()));
        }

        // Handle infer types in extends_type
        if self.has_infer_type(&cond.extends_type) {
            return self.resolve_with_infer(cond);
        }

        // Simple resolution
        if (self.is_subtype)(&cond.check_type, &cond.extends_type) {
            cond.true_type.clone()
        } else {
            cond.false_type.clone()
        }
    }

    /// Resolve with distribution over union types.
    fn resolve_distributed(&self, members: &[Type], cond: &ConditionalType) -> Type {
        let results: Vec<Type> = members
            .iter()
            .map(|member| {
                let member_cond = ConditionalType {
                    check_type: member.clone(),
                    extends_type: cond.extends_type.clone(),
                    true_type: self.substitute_check_type(&cond.true_type, member),
                    false_type: self.substitute_check_type(&cond.false_type, member),
                };
                self.resolve(&member_cond)
            })
            .collect();

        // Deduplicate and flatten the results
        let flattened = self.flatten_union(results);

        if flattened.is_empty() {
            Type::Never
        } else if flattened.len() == 1 {
            flattened.into_iter().next().unwrap()
        } else {
            Type::Union(flattened)
        }
    }

    /// Substitute the check type in a type (for distributed conditional types).
    fn substitute_check_type(&self, ty: &Type, replacement: &Type) -> Type {
        // This is a simplified version - in reality, we need to track
        // which type parameter represents the check type
        match ty {
            Type::TypeParameter(tp) => {
                // If this matches our distributed type parameter, replace it
                // For simplicity, we assume the first type parameter is the check type
                if tp.name == "T" {
                    replacement.clone()
                } else {
                    ty.clone()
                }
            }
            Type::Array(elem) => {
                Type::Array(Box::new(self.substitute_check_type(elem, replacement)))
            }
            Type::Tuple(elems) => Type::Tuple(
                elems
                    .iter()
                    .map(|e| self.substitute_check_type(e, replacement))
                    .collect(),
            ),
            Type::Union(members) => Type::Union(
                members
                    .iter()
                    .map(|m| self.substitute_check_type(m, replacement))
                    .collect(),
            ),
            Type::Intersection(members) => Type::Intersection(
                members
                    .iter()
                    .map(|m| self.substitute_check_type(m, replacement))
                    .collect(),
            ),
            _ => ty.clone(),
        }
    }

    /// Check if a type contains an Infer type.
    fn has_infer_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Infer(_) => true,
            Type::Array(elem) => self.has_infer_type(elem),
            Type::Tuple(elems) => elems.iter().any(|e| self.has_infer_type(e)),
            Type::Union(members) => members.iter().any(|m| self.has_infer_type(m)),
            Type::Intersection(members) => members.iter().any(|m| self.has_infer_type(m)),
            Type::Function(func) => {
                func.parameters.iter().any(|p| self.has_infer_type(&p.ty))
                    || self.has_infer_type(&func.return_type)
            }
            Type::Object(obj) => obj.properties.values().any(|p| self.has_infer_type(&p.ty)),
            _ => false,
        }
    }

    /// Resolve a conditional type that contains infer types.
    fn resolve_with_infer(&self, cond: &ConditionalType) -> Type {
        // Try to match and extract inferred types
        let mut inferences = HashMap::new();

        let matches =
            self.match_and_infer(&cond.check_type, &cond.extends_type, &mut inferences);

        if matches {
            // Substitute inferred types into true_type
            let instantiator = GenericInstantiator::new(inferences);
            instantiator.instantiate(&cond.true_type)
        } else {
            // Use false_type
            cond.false_type.clone()
        }
    }

    /// Match a type against a pattern and extract inferred types.
    fn match_and_infer(
        &self,
        source: &Type,
        pattern: &Type,
        inferences: &mut TypeSubstitution,
    ) -> bool {
        match pattern {
            Type::Infer(name) => {
                // Record the inferred type
                inferences.insert(name.clone(), source.clone());
                true
            }

            Type::Array(pattern_elem) => {
                if let Type::Array(source_elem) = source {
                    self.match_and_infer(source_elem, pattern_elem, inferences)
                } else {
                    false
                }
            }

            Type::Tuple(pattern_elems) => {
                if let Type::Tuple(source_elems) = source {
                    if source_elems.len() != pattern_elems.len() {
                        return false;
                    }
                    source_elems
                        .iter()
                        .zip(pattern_elems.iter())
                        .all(|(s, p)| self.match_and_infer(s, p, inferences))
                } else {
                    false
                }
            }

            Type::Function(pattern_func) => {
                if let Type::Function(source_func) = source {
                    // Match parameters (contravariant)
                    let params_match = source_func.parameters.len() == pattern_func.parameters.len()
                        && source_func
                            .parameters
                            .iter()
                            .zip(pattern_func.parameters.iter())
                            .all(|(s, p)| self.match_and_infer(&s.ty, &p.ty, inferences));

                    // Match return type (covariant)
                    let return_matches = self.match_and_infer(
                        &source_func.return_type,
                        &pattern_func.return_type,
                        inferences,
                    );

                    params_match && return_matches
                } else {
                    false
                }
            }

            Type::Object(pattern_obj) => {
                if let Type::Object(source_obj) = source {
                    // All pattern properties must be matched
                    pattern_obj.properties.iter().all(|(name, pattern_prop)| {
                        if let Some(source_prop) = source_obj.properties.get(name) {
                            self.match_and_infer(&source_prop.ty, &pattern_prop.ty, inferences)
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            }

            // For simple types, just check subtype relationship
            _ => (self.is_subtype)(source, pattern),
        }
    }

    /// Flatten nested unions and remove duplicates.
    fn flatten_union(&self, types: Vec<Type>) -> Vec<Type> {
        let mut result = Vec::new();

        for ty in types {
            match ty {
                Type::Union(members) => {
                    for member in members {
                        if !result.contains(&member) && !matches!(member, Type::Never) {
                            result.push(member);
                        }
                    }
                }
                Type::Never => {
                    // Skip never types in unions
                }
                other => {
                    if !result.contains(&other) {
                        result.push(other);
                    }
                }
            }
        }

        result
    }
}

/// Helper function to check if a type extends another for primitive types.
pub fn primitive_extends(source: &Type, target: &Type) -> bool {
    match (source, target) {
        // any extends anything
        (Type::Any, _) => true,
        // anything extends any
        (_, Type::Any) => true,
        // anything extends unknown
        (_, Type::Unknown) => true,
        // never extends anything
        (Type::Never, _) => true,
        // Same types
        (Type::Primitive(a), Type::Primitive(b)) => a == b,
        (Type::Null, Type::Null) => true,
        (Type::Undefined, Type::Undefined) => true,
        (Type::Void, Type::Void) => true,
        // Literal extends its base type
        (Type::Literal(lit), Type::Primitive(prim)) => match (lit, prim) {
            (super::LiteralType::Number(_), PrimitiveKind::Number) => true,
            (super::LiteralType::String(_), PrimitiveKind::String) => true,
            (super::LiteralType::Boolean(_), PrimitiveKind::Boolean) => true,
            (super::LiteralType::BigInt(_), PrimitiveKind::BigInt) => true,
            _ => false,
        },
        // Same literal values
        (Type::Literal(a), Type::Literal(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{FunctionType, TypeParameter};

    fn simple_subtype(source: &Type, target: &Type) -> bool {
        primitive_extends(source, target)
            || match (source, target) {
                (Type::Object(_), Type::Object(_)) => true, // Simplified
                _ => false,
            }
    }

    #[test]
    fn test_simple_conditional_true() {
        let resolver = ConditionalResolver::new(simple_subtype);

        // number extends number ? string : boolean
        let cond = ConditionalType::new(
            Type::number(),
            Type::number(),
            Type::string(),
            Type::boolean(),
        );

        let result = resolver.resolve(&cond);
        assert!(matches!(result, Type::Primitive(PrimitiveKind::String)));
    }

    #[test]
    fn test_simple_conditional_false() {
        let resolver = ConditionalResolver::new(simple_subtype);

        // string extends number ? string : boolean
        let cond = ConditionalType::new(
            Type::string(),
            Type::number(),
            Type::string(),
            Type::boolean(),
        );

        let result = resolver.resolve(&cond);
        assert!(matches!(result, Type::Primitive(PrimitiveKind::Boolean)));
    }

    #[test]
    fn test_conditional_with_never() {
        let resolver = ConditionalResolver::new(simple_subtype);

        // never extends number ? string : boolean
        // never extends anything, so result is string
        let cond = ConditionalType::new(
            Type::Never,
            Type::number(),
            Type::string(),
            Type::boolean(),
        );

        let result = resolver.resolve(&cond);
        assert!(matches!(result, Type::Primitive(PrimitiveKind::String)));
    }

    #[test]
    fn test_conditional_distributed_union() {
        let resolver = ConditionalResolver::new(simple_subtype);

        // (string | number) extends string ? true : false
        // Distributes to: (string extends string ? true : false) | (number extends string ? true : false)
        // = true | false
        let cond = ConditionalType::new(
            Type::Union(vec![Type::string(), Type::number()]),
            Type::string(),
            Type::Literal(super::super::LiteralType::Boolean(true)),
            Type::Literal(super::super::LiteralType::Boolean(false)),
        );

        let result = resolver.resolve(&cond);

        // Should be a union of true and false
        if let Type::Union(members) = result {
            assert_eq!(members.len(), 2);
        } else {
            panic!("Expected union type");
        }
    }

    #[test]
    fn test_conditional_with_any() {
        let resolver = ConditionalResolver::new(simple_subtype);

        // any extends number ? string : boolean
        // any extends anything, so result is string
        let cond = ConditionalType::new(
            Type::Any,
            Type::number(),
            Type::string(),
            Type::boolean(),
        );

        let result = resolver.resolve(&cond);
        assert!(matches!(result, Type::Primitive(PrimitiveKind::String)));
    }

    #[test]
    fn test_conditional_with_unknown() {
        let resolver = ConditionalResolver::new(simple_subtype);

        // number extends unknown ? string : boolean
        // everything extends unknown, so result is string
        let cond = ConditionalType::new(
            Type::number(),
            Type::Unknown,
            Type::string(),
            Type::boolean(),
        );

        let result = resolver.resolve(&cond);
        assert!(matches!(result, Type::Primitive(PrimitiveKind::String)));
    }

    #[test]
    fn test_infer_return_type() {
        let resolver = ConditionalResolver::new(simple_subtype);

        // (x: number) => string extends (x: number) => infer R ? R : never
        // Should infer R = string
        let source_func = Type::Function(FunctionType {
            type_parameters: vec![],
            parameters: vec![super::super::Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                optional: false,
            }],
            return_type: Box::new(Type::string()),
            rest_parameter: None,
        });

        let pattern_func = Type::Function(FunctionType {
            type_parameters: vec![],
            parameters: vec![super::super::Parameter {
                name: "x".to_string(),
                ty: Type::number(),
                optional: false,
            }],
            return_type: Box::new(Type::Infer("R".to_string())),
            rest_parameter: None,
        });

        let cond = ConditionalType::new(
            source_func,
            pattern_func,
            Type::TypeParameter(TypeParameter {
                name: "R".to_string(),
                constraint: None,
                default: None,
            }),
            Type::Never,
        );

        let result = resolver.resolve(&cond);
        assert!(matches!(result, Type::Primitive(PrimitiveKind::String)));
    }

    #[test]
    fn test_infer_array_element() {
        let resolver = ConditionalResolver::new(simple_subtype);

        // Array<number> extends Array<infer T> ? T : never
        // Should infer T = number
        let cond = ConditionalType::new(
            Type::Array(Box::new(Type::number())),
            Type::Array(Box::new(Type::Infer("T".to_string()))),
            Type::TypeParameter(TypeParameter {
                name: "T".to_string(),
                constraint: None,
                default: None,
            }),
            Type::Never,
        );

        let result = resolver.resolve(&cond);
        assert!(matches!(result, Type::Primitive(PrimitiveKind::Number)));
    }
}
