//! Structural subtype checking.
//!
//! This module implements the core logic engine for TypeScript's structural
//! subtyping. It uses coinductive semantics to handle recursive types.
//!
//! Key features:
//! - O(1) equality check via TypeId comparison
//! - Cycle detection for recursive types (coinductive)
//! - Set-theoretic operations for unions and intersections

use std::collections::HashSet;
use crate::solver::types::*;
use crate::solver::intern::TypeInterner;

/// Result of a subtype check
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SubtypeResult {
    /// The relationship is definitely true
    True,
    /// The relationship is definitely false
    False,
    /// We're in a cycle and assuming true (provisional)
    Provisional,
}

impl SubtypeResult {
    pub fn is_true(self) -> bool {
        matches!(self, SubtypeResult::True | SubtypeResult::Provisional)
    }

    pub fn is_false(self) -> bool {
        matches!(self, SubtypeResult::False)
    }
}

/// Subtype checking context.
/// Maintains the "seen" set for cycle detection.
pub struct SubtypeChecker<'a> {
    interner: &'a TypeInterner,
    /// Active subtype pairs being checked (for cycle detection)
    in_progress: HashSet<(TypeId, TypeId)>,
}

impl<'a> SubtypeChecker<'a> {
    pub fn new(interner: &'a TypeInterner) -> Self {
        SubtypeChecker {
            interner,
            in_progress: HashSet::new(),
        }
    }

    /// Check if `source` is a subtype of `target`.
    /// This is the main entry point for subtype checking.
    pub fn is_subtype_of(&mut self, source: TypeId, target: TypeId) -> bool {
        self.check_subtype(source, target).is_true()
    }

    /// Check if `source` is assignable to `target`.
    /// In TypeScript, assignability is slightly looser than strict subtyping.
    pub fn is_assignable_to(&mut self, source: TypeId, target: TypeId) -> bool {
        // For now, treat assignability the same as subtyping
        // TODO: Handle bivariant function parameters, any, etc.
        self.is_subtype_of(source, target)
    }

    /// Internal subtype check with cycle detection
    fn check_subtype(&mut self, source: TypeId, target: TypeId) -> SubtypeResult {
        // =========================================================================
        // Fast paths
        // =========================================================================

        // Same type is always a subtype of itself
        if source == target {
            return SubtypeResult::True;
        }

        // Any is assignable to anything
        if source == TypeId::ANY {
            return SubtypeResult::True;
        }

        // Everything is assignable to any
        if target == TypeId::ANY {
            return SubtypeResult::True;
        }

        // Everything is assignable to unknown
        if target == TypeId::UNKNOWN {
            return SubtypeResult::True;
        }

        // Never is assignable to everything
        if source == TypeId::NEVER {
            return SubtypeResult::True;
        }

        // Nothing (except never) is assignable to never
        if target == TypeId::NEVER {
            return SubtypeResult::False;
        }

        // Error types are compatible with everything (for error recovery)
        if source == TypeId::ERROR || target == TypeId::ERROR {
            return SubtypeResult::True;
        }

        // =========================================================================
        // Cycle detection (coinduction)
        // =========================================================================

        let pair = (source, target);
        if self.in_progress.contains(&pair) {
            // We're in a cycle - return provisional true
            // This implements coinductive semantics for recursive types
            return SubtypeResult::Provisional;
        }

        // Mark as in-progress
        self.in_progress.insert(pair);

        // Do the actual check
        let result = self.check_subtype_inner(source, target);

        // Remove from in-progress
        self.in_progress.remove(&pair);

        result
    }

    /// Inner subtype check (after cycle detection)
    fn check_subtype_inner(&mut self, source: TypeId, target: TypeId) -> SubtypeResult {
        // Look up the type keys
        let source_key = match self.interner.lookup(source) {
            Some(k) => k,
            None => return SubtypeResult::False,
        };
        let target_key = match self.interner.lookup(target) {
            Some(k) => k,
            None => return SubtypeResult::False,
        };

        // =========================================================================
        // Structural checks
        // =========================================================================

        match (&source_key, &target_key) {
            // Intrinsic to intrinsic
            (TypeKey::Intrinsic(s), TypeKey::Intrinsic(t)) => {
                self.check_intrinsic_subtype(*s, *t)
            }

            // Literal to intrinsic
            (TypeKey::Literal(lit), TypeKey::Intrinsic(t)) => {
                self.check_literal_to_intrinsic(lit, *t)
            }

            // Literal to literal
            (TypeKey::Literal(s), TypeKey::Literal(t)) => {
                if s == t {
                    SubtypeResult::True
                } else {
                    SubtypeResult::False
                }
            }

            // Union source: all members must be subtypes of target
            (TypeKey::Union(members), _) => {
                for &member in members {
                    if !self.check_subtype(member, target).is_true() {
                        return SubtypeResult::False;
                    }
                }
                SubtypeResult::True
            }

            // Union target: source must be subtype of at least one member
            (_, TypeKey::Union(members)) => {
                for &member in members {
                    if self.check_subtype(source, member).is_true() {
                        return SubtypeResult::True;
                    }
                }
                SubtypeResult::False
            }

            // Intersection source: source is subtype if any constituent is
            (TypeKey::Intersection(members), _) => {
                for &member in members {
                    if self.check_subtype(member, target).is_true() {
                        return SubtypeResult::True;
                    }
                }
                SubtypeResult::False
            }

            // Intersection target: all members must be satisfied
            (_, TypeKey::Intersection(members)) => {
                for &member in members {
                    if !self.check_subtype(source, member).is_true() {
                        return SubtypeResult::False;
                    }
                }
                SubtypeResult::True
            }

            // Array to array
            (TypeKey::Array(s_elem), TypeKey::Array(t_elem)) => {
                // Arrays are covariant in TypeScript
                self.check_subtype(*s_elem, *t_elem)
            }

            // Tuple to tuple
            (TypeKey::Tuple(s_elems), TypeKey::Tuple(t_elems)) => {
                self.check_tuple_subtype(s_elems, t_elems)
            }

            // Tuple to array
            (TypeKey::Tuple(elems), TypeKey::Array(t_elem)) => {
                // Tuple is subtype of array if all elements are subtypes
                for elem in elems {
                    if !self.check_subtype(elem.type_id, *t_elem).is_true() {
                        return SubtypeResult::False;
                    }
                }
                SubtypeResult::True
            }

            // Object to object
            (TypeKey::Object(s_props), TypeKey::Object(t_props)) => {
                self.check_object_subtype(s_props, t_props)
            }

            // Function to function
            (TypeKey::Function(s_fn), TypeKey::Function(t_fn)) => {
                self.check_function_subtype(s_fn, t_fn)
            }

            // Reference types (defer to symbol resolution)
            (TypeKey::Ref(s_sym), TypeKey::Ref(t_sym)) => {
                // Same symbol reference
                if s_sym == t_sym {
                    SubtypeResult::True
                } else {
                    // TODO: Resolve and compare declared types
                    SubtypeResult::False
                }
            }

            // Conditional types
            (TypeKey::Conditional(s_cond), TypeKey::Conditional(t_cond)) => {
                // TODO: Implement proper conditional type comparison
                if s_cond == t_cond {
                    SubtypeResult::True
                } else {
                    SubtypeResult::False
                }
            }

            // Index access types
            (TypeKey::IndexAccess(s_obj, s_idx), TypeKey::IndexAccess(t_obj, t_idx)) => {
                if self.check_subtype(*s_obj, *t_obj).is_true()
                    && self.check_subtype(*s_idx, *t_idx).is_true()
                {
                    SubtypeResult::True
                } else {
                    SubtypeResult::False
                }
            }

            // Default: not a subtype
            _ => SubtypeResult::False,
        }
    }

    /// Check intrinsic to intrinsic subtyping
    fn check_intrinsic_subtype(&self, source: IntrinsicKind, target: IntrinsicKind) -> SubtypeResult {
        if source == target {
            return SubtypeResult::True;
        }

        // null and undefined are subtypes of their non-strict counterparts
        match (source, target) {
            // void accepts undefined
            (IntrinsicKind::Undefined, IntrinsicKind::Void) => SubtypeResult::True,

            // object accepts non-primitive types (but we handle that separately)
            _ => SubtypeResult::False,
        }
    }

    /// Check literal to intrinsic subtyping
    fn check_literal_to_intrinsic(&self, literal: &LiteralValue, target: IntrinsicKind) -> SubtypeResult {
        let matches = match literal {
            LiteralValue::String(_) => target == IntrinsicKind::String,
            LiteralValue::Number(_) => target == IntrinsicKind::Number,
            LiteralValue::BigInt(_) => target == IntrinsicKind::Bigint,
            LiteralValue::Boolean(_) => target == IntrinsicKind::Boolean,
        };

        if matches {
            SubtypeResult::True
        } else {
            SubtypeResult::False
        }
    }

    /// Check tuple subtyping
    fn check_tuple_subtype(&mut self, source: &[TupleElement], target: &[TupleElement]) -> SubtypeResult {
        // Count required elements
        let source_required = source.iter().filter(|e| !e.optional && !e.rest).count();
        let target_required = target.iter().filter(|e| !e.optional && !e.rest).count();

        // Source must have at least as many required elements
        if source_required < target_required {
            return SubtypeResult::False;
        }

        // Check each element
        for (i, t_elem) in target.iter().enumerate() {
            if t_elem.rest {
                // Rest element: remaining source elements must match
                for s_elem in source.iter().skip(i) {
                    if !self.check_subtype(s_elem.type_id, t_elem.type_id).is_true() {
                        return SubtypeResult::False;
                    }
                }
                break;
            }

            if let Some(s_elem) = source.get(i) {
                if !self.check_subtype(s_elem.type_id, t_elem.type_id).is_true() {
                    return SubtypeResult::False;
                }
            } else if !t_elem.optional {
                // Missing required element
                return SubtypeResult::False;
            }
        }

        SubtypeResult::True
    }

    /// Check object subtyping (structural)
    fn check_object_subtype(&mut self, source: &[PropertyInfo], target: &[PropertyInfo]) -> SubtypeResult {
        // For each property in target, source must have a compatible property
        for t_prop in target {
            let s_prop = source.iter().find(|p| p.name == t_prop.name);

            match s_prop {
                Some(sp) => {
                    // Property exists, check type compatibility
                    if !self.check_subtype(sp.type_id, t_prop.type_id).is_true() {
                        return SubtypeResult::False;
                    }
                    // Check optional compatibility
                    // Optional in source can't satisfy required in target
                    if sp.optional && !t_prop.optional {
                        return SubtypeResult::False;
                    }
                }
                None => {
                    // Property missing
                    if !t_prop.optional {
                        return SubtypeResult::False;
                    }
                }
            }
        }

        SubtypeResult::True
    }

    /// Check function subtyping
    fn check_function_subtype(&mut self, source: &FunctionShape, target: &FunctionShape) -> SubtypeResult {
        // Constructor vs non-constructor
        if source.is_constructor != target.is_constructor {
            return SubtypeResult::False;
        }

        // Return type is covariant
        if !self.check_subtype(source.return_type, target.return_type).is_true() {
            return SubtypeResult::False;
        }

        // Parameters are contravariant (but TypeScript uses bivariance for methods)
        // For now, we use covariance for simplicity
        // Source can have fewer parameters (callback compatibility)
        if source.params.len() > target.params.len() {
            return SubtypeResult::False;
        }

        for (i, s_param) in source.params.iter().enumerate() {
            if let Some(t_param) = target.params.get(i) {
                // Bivariant: either direction works
                if !self.check_subtype(s_param.type_id, t_param.type_id).is_true()
                    && !self.check_subtype(t_param.type_id, s_param.type_id).is_true()
                {
                    return SubtypeResult::False;
                }
            }
        }

        // Check rest parameters
        // TODO: Handle rest parameter compatibility

        SubtypeResult::True
    }
}

/// Convenience function for one-off subtype checks
pub fn is_subtype_of(interner: &TypeInterner, source: TypeId, target: TypeId) -> bool {
    let mut checker = SubtypeChecker::new(interner);
    checker.is_subtype_of(source, target)
}


// TODO: move tests to separate file

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intrinsic_subtyping() {
        let interner = TypeInterner::new();
        let mut checker = SubtypeChecker::new(&interner);

        // Same type
        assert!(checker.is_subtype_of(TypeId::STRING, TypeId::STRING));
        assert!(checker.is_subtype_of(TypeId::NUMBER, TypeId::NUMBER));

        // Different intrinsics
        assert!(!checker.is_subtype_of(TypeId::STRING, TypeId::NUMBER));

        // Any relations
        assert!(checker.is_subtype_of(TypeId::ANY, TypeId::STRING));
        assert!(checker.is_subtype_of(TypeId::STRING, TypeId::ANY));

        // Unknown relations
        assert!(checker.is_subtype_of(TypeId::STRING, TypeId::UNKNOWN));
        assert!(!checker.is_subtype_of(TypeId::UNKNOWN, TypeId::STRING));

        // Never relations
        assert!(checker.is_subtype_of(TypeId::NEVER, TypeId::STRING));
        assert!(!checker.is_subtype_of(TypeId::STRING, TypeId::NEVER));
    }

    #[test]
    fn test_literal_subtyping() {
        let interner = TypeInterner::new();
        let mut checker = SubtypeChecker::new(&interner);

        let hello = interner.literal_string("hello");
        let world = interner.literal_string("world");

        // Literal to same literal
        assert!(checker.is_subtype_of(hello, hello));

        // Literal to different literal
        assert!(!checker.is_subtype_of(hello, world));

        // Literal to intrinsic
        assert!(checker.is_subtype_of(hello, TypeId::STRING));
        assert!(!checker.is_subtype_of(hello, TypeId::NUMBER));
    }

    #[test]
    fn test_union_subtyping() {
        let interner = TypeInterner::new();
        let mut checker = SubtypeChecker::new(&interner);

        let string_or_number = interner.union(vec![TypeId::STRING, TypeId::NUMBER]);

        // Union member is subtype of union
        assert!(checker.is_subtype_of(TypeId::STRING, string_or_number));
        assert!(checker.is_subtype_of(TypeId::NUMBER, string_or_number));

        // Non-member is not subtype
        assert!(!checker.is_subtype_of(TypeId::BOOLEAN, string_or_number));

        // Union is subtype if all members are subtypes
        let just_string = interner.union(vec![TypeId::STRING]);
        assert!(checker.is_subtype_of(just_string, string_or_number));
    }

    #[test]
    fn test_object_subtyping() {
        use std::sync::Arc;

        let interner = TypeInterner::new();
        let mut checker = SubtypeChecker::new(&interner);

        // { x: number }
        let obj_x = interner.object(vec![
            PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
        ]);

        // { x: number, y: string }
        let obj_xy = interner.object(vec![
            PropertyInfo { name: Arc::from("x"), type_id: TypeId::NUMBER, optional: false, readonly: false },
            PropertyInfo { name: Arc::from("y"), type_id: TypeId::STRING, optional: false, readonly: false },
        ]);

        // Object with more properties is subtype
        assert!(checker.is_subtype_of(obj_xy, obj_x));

        // Object with fewer properties is not subtype
        assert!(!checker.is_subtype_of(obj_x, obj_xy));
    }

    #[test]
    fn test_array_subtyping() {
        let interner = TypeInterner::new();
        let mut checker = SubtypeChecker::new(&interner);

        let string_array = interner.array(TypeId::STRING);
        let number_array = interner.array(TypeId::NUMBER);
        let any_array = interner.array(TypeId::ANY);

        // Same element type
        assert!(checker.is_subtype_of(string_array, string_array));

        // Different element type
        assert!(!checker.is_subtype_of(string_array, number_array));

        // Covariance with any
        assert!(checker.is_subtype_of(string_array, any_array));
    }
}
