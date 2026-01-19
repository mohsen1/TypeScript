//! Type Assignability
//!
//! Implements type compatibility checking according to TypeScript's rules.

use crate::types::{
    ResolvedType, ObjectType, FunctionType, TypeParameterType, LiteralType,
    PropertySignature, ParameterType, IndexSignature,
};
use zang_core::InternedString;
use std::collections::HashMap;

/// Result of an assignability check
#[derive(Debug, Clone)]
pub enum AssignabilityResult {
    /// Types are assignable
    Assignable,
    /// Types are not assignable
    NotAssignable(AssignabilityError),
}

impl AssignabilityResult {
    pub fn is_assignable(&self) -> bool {
        matches!(self, AssignabilityResult::Assignable)
    }
}

/// Error details for non-assignable types
#[derive(Debug, Clone)]
pub struct AssignabilityError {
    /// The source type that cannot be assigned
    pub source: String,
    /// The target type that the source cannot be assigned to
    pub target: String,
    /// Detailed reason for the failure
    pub reason: AssignabilityReason,
}

/// Reason for assignability failure
#[derive(Debug, Clone)]
pub enum AssignabilityReason {
    /// Types are fundamentally incompatible
    TypeMismatch,
    /// Missing property in target
    MissingProperty(InternedString),
    /// Property type mismatch
    PropertyTypeMismatch(InternedString),
    /// Parameter count mismatch
    ParameterCountMismatch { expected: usize, got: usize },
    /// Parameter type mismatch
    ParameterTypeMismatch(usize),
    /// Return type mismatch
    ReturnTypeMismatch,
    /// Index signature mismatch
    IndexSignatureMismatch,
    /// Type parameter constraint violation
    ConstraintViolation,
    /// Union type member not assignable
    UnionMemberNotAssignable(usize),
    /// Intersection type requirement not met
    IntersectionRequirementNotMet,
}

/// Cache for type relationship results
pub struct RelationshipCache {
    /// Assignability cache: (source_hash, target_hash) -> result
    cache: HashMap<(u64, u64), AssignabilityResult>,
}

impl RelationshipCache {
    /// Creates a new relationship cache
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Gets a cached result
    pub fn get(&self, source_hash: u64, target_hash: u64) -> Option<&AssignabilityResult> {
        self.cache.get(&(source_hash, target_hash))
    }

    /// Stores a result in the cache
    pub fn insert(&mut self, source_hash: u64, target_hash: u64, result: AssignabilityResult) {
        self.cache.insert((source_hash, target_hash), result);
    }

    /// Clears the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl Default for RelationshipCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Type assignability checker
pub struct AssignabilityChecker {
    /// Cache for relationship results
    cache: RelationshipCache,
    /// Recursion depth for detecting infinite loops
    depth: u32,
    /// Maximum recursion depth
    max_depth: u32,
}

impl AssignabilityChecker {
    /// Creates a new assignability checker
    pub fn new() -> Self {
        Self {
            cache: RelationshipCache::new(),
            depth: 0,
            max_depth: 100,
        }
    }

    /// Checks if `source` is assignable to `target`
    pub fn is_assignable_to(&mut self, source: &ResolvedType, target: &ResolvedType) -> AssignabilityResult {
        // Check depth limit
        if self.depth >= self.max_depth {
            return AssignabilityResult::Assignable; // Assume assignable to avoid infinite loops
        }

        self.depth += 1;
        let result = self.check_assignability(source, target);
        self.depth -= 1;
        result
    }

    /// Internal assignability check
    fn check_assignability(&mut self, source: &ResolvedType, target: &ResolvedType) -> AssignabilityResult {
        // Identical types are always assignable
        if self.is_type_identical(source, target) {
            return AssignabilityResult::Assignable;
        }

        // Any is assignable to/from everything (except never for --noImplicitAny)
        if matches!(source, ResolvedType::Any) || matches!(target, ResolvedType::Any) {
            return AssignabilityResult::Assignable;
        }

        // Unknown accepts all types as source
        if matches!(target, ResolvedType::Unknown) {
            return AssignabilityResult::Assignable;
        }

        // Never is assignable to everything
        if matches!(source, ResolvedType::Never) {
            return AssignabilityResult::Assignable;
        }

        // Nothing is assignable to never (except never itself, handled above)
        if matches!(target, ResolvedType::Never) {
            return AssignabilityResult::NotAssignable(AssignabilityError {
                source: format!("{:?}", source),
                target: "never".to_string(),
                reason: AssignabilityReason::TypeMismatch,
            });
        }

        // Undefined is assignable to void
        if matches!(source, ResolvedType::Undefined) && matches!(target, ResolvedType::Void) {
            return AssignabilityResult::Assignable;
        }

        // Check structural compatibility
        match (source, target) {
            // Primitive types
            (ResolvedType::String, ResolvedType::String) => AssignabilityResult::Assignable,
            (ResolvedType::Number, ResolvedType::Number) => AssignabilityResult::Assignable,
            (ResolvedType::Boolean, ResolvedType::Boolean) => AssignabilityResult::Assignable,
            (ResolvedType::BigInt, ResolvedType::BigInt) => AssignabilityResult::Assignable,
            (ResolvedType::Symbol, ResolvedType::Symbol) => AssignabilityResult::Assignable,
            (ResolvedType::Void, ResolvedType::Void) => AssignabilityResult::Assignable,
            (ResolvedType::Null, ResolvedType::Null) => AssignabilityResult::Assignable,
            (ResolvedType::Undefined, ResolvedType::Undefined) => AssignabilityResult::Assignable,

            // Literal types to their base types
            (ResolvedType::Literal(LiteralType::String(_)), ResolvedType::String) => {
                AssignabilityResult::Assignable
            }
            (ResolvedType::Literal(LiteralType::Number(_)), ResolvedType::Number) => {
                AssignabilityResult::Assignable
            }
            (ResolvedType::Literal(LiteralType::Boolean(_)), ResolvedType::Boolean) => {
                AssignabilityResult::Assignable
            }
            (ResolvedType::Literal(LiteralType::BigInt(_)), ResolvedType::BigInt) => {
                AssignabilityResult::Assignable
            }

            // Literal to literal
            (ResolvedType::Literal(a), ResolvedType::Literal(b)) => {
                self.check_literal_assignability(a, b)
            }

            // Array types
            (ResolvedType::Array(source_elem), ResolvedType::Array(target_elem)) => {
                self.is_assignable_to(source_elem, target_elem)
            }

            // Tuple to tuple
            (ResolvedType::Tuple(source_types), ResolvedType::Tuple(target_types)) => {
                self.check_tuple_assignability(source_types, target_types)
            }

            // Tuple to array
            (ResolvedType::Tuple(source_types), ResolvedType::Array(target_elem)) => {
                // All tuple elements must be assignable to the array element type
                for (i, elem) in source_types.iter().enumerate() {
                    if !self.is_assignable_to(elem, target_elem).is_assignable() {
                        return AssignabilityResult::NotAssignable(AssignabilityError {
                            source: format!("{:?}", source),
                            target: format!("{:?}", target),
                            reason: AssignabilityReason::UnionMemberNotAssignable(i),
                        });
                    }
                }
                AssignabilityResult::Assignable
            }

            // Union source: all members must be assignable to target
            (ResolvedType::Union(source_types), _) => {
                for (i, member) in source_types.iter().enumerate() {
                    if !self.is_assignable_to(member, target).is_assignable() {
                        return AssignabilityResult::NotAssignable(AssignabilityError {
                            source: format!("{:?}", source),
                            target: format!("{:?}", target),
                            reason: AssignabilityReason::UnionMemberNotAssignable(i),
                        });
                    }
                }
                AssignabilityResult::Assignable
            }

            // Union target: source must be assignable to at least one member
            (_, ResolvedType::Union(target_types)) => {
                for member in target_types {
                    if self.is_assignable_to(source, member).is_assignable() {
                        return AssignabilityResult::Assignable;
                    }
                }
                AssignabilityResult::NotAssignable(AssignabilityError {
                    source: format!("{:?}", source),
                    target: format!("{:?}", target),
                    reason: AssignabilityReason::TypeMismatch,
                })
            }

            // Intersection source: any member assignable to target is enough
            (ResolvedType::Intersection(source_types), _) => {
                for member in source_types {
                    if self.is_assignable_to(member, target).is_assignable() {
                        return AssignabilityResult::Assignable;
                    }
                }
                // Try the combined type
                // TODO: Properly merge intersection types
                AssignabilityResult::NotAssignable(AssignabilityError {
                    source: format!("{:?}", source),
                    target: format!("{:?}", target),
                    reason: AssignabilityReason::TypeMismatch,
                })
            }

            // Intersection target: source must be assignable to all members
            (_, ResolvedType::Intersection(target_types)) => {
                for (i, member) in target_types.iter().enumerate() {
                    if !self.is_assignable_to(source, member).is_assignable() {
                        return AssignabilityResult::NotAssignable(AssignabilityError {
                            source: format!("{:?}", source),
                            target: format!("{:?}", target),
                            reason: AssignabilityReason::IntersectionRequirementNotMet,
                        });
                    }
                }
                AssignabilityResult::Assignable
            }

            // Object types
            (ResolvedType::Object(source_obj), ResolvedType::Object(target_obj)) => {
                self.check_object_assignability(source_obj, target_obj)
            }

            // Function types
            (ResolvedType::Function(source_func), ResolvedType::Function(target_func)) => {
                self.check_function_assignability(source_func, target_func)
            }

            // Object to function (check for call signature)
            (ResolvedType::Object(obj), ResolvedType::Function(func)) => {
                if let Some(sig) = obj.call_signatures.first() {
                    // Check if the call signature matches the function
                    let func_from_sig = FunctionType {
                        type_parameters: sig.type_parameters.clone(),
                        parameters: sig.parameters.clone(),
                        return_type: sig.return_type.clone(),
                    };
                    self.check_function_assignability(&func_from_sig, func)
                } else {
                    AssignabilityResult::NotAssignable(AssignabilityError {
                        source: format!("{:?}", source),
                        target: format!("{:?}", target),
                        reason: AssignabilityReason::TypeMismatch,
                    })
                }
            }

            // Default: not assignable
            _ => AssignabilityResult::NotAssignable(AssignabilityError {
                source: format!("{:?}", source),
                target: format!("{:?}", target),
                reason: AssignabilityReason::TypeMismatch,
            }),
        }
    }

    /// Checks literal type assignability
    fn check_literal_assignability(&self, source: &LiteralType, target: &LiteralType) -> AssignabilityResult {
        let assignable = match (source, target) {
            (LiteralType::String(a), LiteralType::String(b)) => a == b,
            (LiteralType::Number(a), LiteralType::Number(b)) => (a - b).abs() < f64::EPSILON,
            (LiteralType::Boolean(a), LiteralType::Boolean(b)) => a == b,
            (LiteralType::BigInt(a), LiteralType::BigInt(b)) => a == b,
            _ => false,
        };

        if assignable {
            AssignabilityResult::Assignable
        } else {
            AssignabilityResult::NotAssignable(AssignabilityError {
                source: format!("{:?}", source),
                target: format!("{:?}", target),
                reason: AssignabilityReason::TypeMismatch,
            })
        }
    }

    /// Checks tuple assignability
    fn check_tuple_assignability(
        &mut self,
        source: &[ResolvedType],
        target: &[ResolvedType],
    ) -> AssignabilityResult {
        if source.len() < target.len() {
            return AssignabilityResult::NotAssignable(AssignabilityError {
                source: format!("tuple of length {}", source.len()),
                target: format!("tuple of length {}", target.len()),
                reason: AssignabilityReason::ParameterCountMismatch {
                    expected: target.len(),
                    got: source.len(),
                },
            });
        }

        for (i, (s, t)) in source.iter().zip(target.iter()).enumerate() {
            if !self.is_assignable_to(s, t).is_assignable() {
                return AssignabilityResult::NotAssignable(AssignabilityError {
                    source: format!("{:?}", s),
                    target: format!("{:?}", t),
                    reason: AssignabilityReason::UnionMemberNotAssignable(i),
                });
            }
        }

        AssignabilityResult::Assignable
    }

    /// Checks object type assignability
    fn check_object_assignability(
        &mut self,
        source: &ObjectType,
        target: &ObjectType,
    ) -> AssignabilityResult {
        // Check that all target properties exist in source with compatible types
        for target_prop in &target.properties {
            if let Some(source_prop) = source.properties.iter().find(|p| p.name == target_prop.name) {
                // Check property type compatibility
                if !self.is_assignable_to(&source_prop.ty, &target_prop.ty).is_assignable() {
                    return AssignabilityResult::NotAssignable(AssignabilityError {
                        source: format!("{:?}", source_prop.ty),
                        target: format!("{:?}", target_prop.ty),
                        reason: AssignabilityReason::PropertyTypeMismatch(target_prop.name),
                    });
                }
            } else if !target_prop.optional {
                // Required property missing
                return AssignabilityResult::NotAssignable(AssignabilityError {
                    source: "object".to_string(),
                    target: "object".to_string(),
                    reason: AssignabilityReason::MissingProperty(target_prop.name),
                });
            }
        }

        // Check call signatures
        for target_sig in &target.call_signatures {
            let has_compatible_sig = source.call_signatures.iter().any(|source_sig| {
                let source_func = FunctionType {
                    type_parameters: source_sig.type_parameters.clone(),
                    parameters: source_sig.parameters.clone(),
                    return_type: source_sig.return_type.clone(),
                };
                let target_func = FunctionType {
                    type_parameters: target_sig.type_parameters.clone(),
                    parameters: target_sig.parameters.clone(),
                    return_type: target_sig.return_type.clone(),
                };
                self.check_function_assignability(&source_func, &target_func).is_assignable()
            });

            if !has_compatible_sig && !source.call_signatures.is_empty() {
                return AssignabilityResult::NotAssignable(AssignabilityError {
                    source: "object".to_string(),
                    target: "object".to_string(),
                    reason: AssignabilityReason::TypeMismatch,
                });
            }
        }

        // Check index signatures
        for target_sig in &target.index_signatures {
            let has_compatible_sig = source.index_signatures.iter().any(|source_sig| {
                self.is_assignable_to(&source_sig.key_type, &target_sig.key_type).is_assignable()
                    && self.is_assignable_to(&source_sig.value_type, &target_sig.value_type).is_assignable()
            });

            if !has_compatible_sig && !source.index_signatures.is_empty() {
                return AssignabilityResult::NotAssignable(AssignabilityError {
                    source: "object".to_string(),
                    target: "object".to_string(),
                    reason: AssignabilityReason::IndexSignatureMismatch,
                });
            }
        }

        AssignabilityResult::Assignable
    }

    /// Checks function type assignability (contravariant parameters, covariant return)
    fn check_function_assignability(
        &mut self,
        source: &FunctionType,
        target: &FunctionType,
    ) -> AssignabilityResult {
        // Target can have fewer parameters than source (excess parameters are allowed)
        // But target cannot require more parameters than source provides
        let required_source = source.parameters.iter().filter(|p| !p.optional).count();
        let required_target = target.parameters.iter().filter(|p| !p.optional).count();

        if required_target > source.parameters.len() {
            return AssignabilityResult::NotAssignable(AssignabilityError {
                source: "function".to_string(),
                target: "function".to_string(),
                reason: AssignabilityReason::ParameterCountMismatch {
                    expected: required_target,
                    got: source.parameters.len(),
                },
            });
        }

        // Check parameter types (contravariant)
        for (i, target_param) in target.parameters.iter().enumerate() {
            if i < source.parameters.len() {
                let source_param = &source.parameters[i];
                // Parameters are contravariant: target param should be assignable to source param
                if !self.is_assignable_to(&target_param.ty, &source_param.ty).is_assignable() {
                    return AssignabilityResult::NotAssignable(AssignabilityError {
                        source: format!("{:?}", source_param.ty),
                        target: format!("{:?}", target_param.ty),
                        reason: AssignabilityReason::ParameterTypeMismatch(i),
                    });
                }
            }
        }

        // Check return type (covariant)
        if !self.is_assignable_to(&source.return_type, &target.return_type).is_assignable() {
            return AssignabilityResult::NotAssignable(AssignabilityError {
                source: format!("{:?}", source.return_type),
                target: format!("{:?}", target.return_type),
                reason: AssignabilityReason::ReturnTypeMismatch,
            });
        }

        AssignabilityResult::Assignable
    }

    /// Checks if two types are identical
    fn is_type_identical(&self, a: &ResolvedType, b: &ResolvedType) -> bool {
        match (a, b) {
            (ResolvedType::Any, ResolvedType::Any) => true,
            (ResolvedType::Unknown, ResolvedType::Unknown) => true,
            (ResolvedType::Never, ResolvedType::Never) => true,
            (ResolvedType::Void, ResolvedType::Void) => true,
            (ResolvedType::Undefined, ResolvedType::Undefined) => true,
            (ResolvedType::Null, ResolvedType::Null) => true,
            (ResolvedType::String, ResolvedType::String) => true,
            (ResolvedType::Number, ResolvedType::Number) => true,
            (ResolvedType::Boolean, ResolvedType::Boolean) => true,
            (ResolvedType::BigInt, ResolvedType::BigInt) => true,
            (ResolvedType::Symbol, ResolvedType::Symbol) => true,
            (ResolvedType::Array(a), ResolvedType::Array(b)) => self.is_type_identical(a, b),
            (ResolvedType::Literal(a), ResolvedType::Literal(b)) => {
                match (a, b) {
                    (LiteralType::String(a), LiteralType::String(b)) => a == b,
                    (LiteralType::Number(a), LiteralType::Number(b)) => (a - b).abs() < f64::EPSILON,
                    (LiteralType::Boolean(a), LiteralType::Boolean(b)) => a == b,
                    (LiteralType::BigInt(a), LiteralType::BigInt(b)) => a == b,
                    _ => false,
                }
            }
            _ => false, // For complex types, we don't check deep equality here
        }
    }
}

impl Default for AssignabilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_assignability() {
        let mut checker = AssignabilityChecker::new();

        assert!(checker.is_assignable_to(&ResolvedType::String, &ResolvedType::String).is_assignable());
        assert!(checker.is_assignable_to(&ResolvedType::Number, &ResolvedType::Number).is_assignable());
        assert!(!checker.is_assignable_to(&ResolvedType::String, &ResolvedType::Number).is_assignable());
    }

    #[test]
    fn test_any_assignability() {
        let mut checker = AssignabilityChecker::new();

        assert!(checker.is_assignable_to(&ResolvedType::Any, &ResolvedType::String).is_assignable());
        assert!(checker.is_assignable_to(&ResolvedType::String, &ResolvedType::Any).is_assignable());
    }

    #[test]
    fn test_unknown_assignability() {
        let mut checker = AssignabilityChecker::new();

        assert!(checker.is_assignable_to(&ResolvedType::String, &ResolvedType::Unknown).is_assignable());
        assert!(!checker.is_assignable_to(&ResolvedType::Unknown, &ResolvedType::String).is_assignable());
    }

    #[test]
    fn test_never_assignability() {
        let mut checker = AssignabilityChecker::new();

        assert!(checker.is_assignable_to(&ResolvedType::Never, &ResolvedType::String).is_assignable());
        assert!(!checker.is_assignable_to(&ResolvedType::String, &ResolvedType::Never).is_assignable());
    }

    #[test]
    fn test_literal_to_base_type() {
        let mut checker = AssignabilityChecker::new();

        let string_lit = ResolvedType::Literal(LiteralType::String("hello".to_string()));
        assert!(checker.is_assignable_to(&string_lit, &ResolvedType::String).is_assignable());

        let num_lit = ResolvedType::Literal(LiteralType::Number(42.0));
        assert!(checker.is_assignable_to(&num_lit, &ResolvedType::Number).is_assignable());
    }

    #[test]
    fn test_union_assignability() {
        let mut checker = AssignabilityChecker::new();

        let union = ResolvedType::Union(vec![ResolvedType::String, ResolvedType::Number]);

        // String is assignable to string | number
        assert!(checker.is_assignable_to(&ResolvedType::String, &union).is_assignable());

        // string | number is not assignable to string (because number isn't)
        assert!(!checker.is_assignable_to(&union, &ResolvedType::String).is_assignable());
    }

    #[test]
    fn test_array_assignability() {
        let mut checker = AssignabilityChecker::new();

        let string_array = ResolvedType::Array(Box::new(ResolvedType::String));
        let number_array = ResolvedType::Array(Box::new(ResolvedType::Number));

        assert!(checker.is_assignable_to(&string_array, &string_array).is_assignable());
        assert!(!checker.is_assignable_to(&string_array, &number_array).is_assignable());
    }
}
