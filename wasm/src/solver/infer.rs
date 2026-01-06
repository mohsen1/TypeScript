//! Type inference engine using Union-Find.
//!
//! This module implements type inference for generic functions using
//! the `ena` crate's Union-Find data structure.
//!
//! Key features:
//! - Inference variables for generic type parameters
//! - Constraint collection during type checking
//! - Bounds checking (L <: α <: U)
//! - Best common type calculation
//! - Efficient unification with path compression

use ena::unify::{InPlaceUnificationTable, UnifyKey, UnifyValue, NoError};
use std::sync::Arc;
use crate::solver::types::*;
use crate::solver::TypeDatabase;

#[cfg(test)]
use crate::solver::TypeInterner;

/// An inference variable representing an unknown type.
/// These are created when instantiating generic functions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InferenceVar(pub u32);

/// Wrapper for TypeId to implement UnifyValue (avoiding orphan rule)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct InferenceValue(pub Option<TypeId>);

impl UnifyKey for InferenceVar {
    type Value = InferenceValue;

    fn index(&self) -> u32 {
        self.0
    }

    fn from_index(u: u32) -> Self {
        InferenceVar(u)
    }

    fn tag() -> &'static str {
        "InferenceVar"
    }
}

impl UnifyValue for InferenceValue {
    type Error = NoError;

    fn unify_values(a: &Self, b: &Self) -> Result<Self, Self::Error> {
        match (a.0, b.0) {
            (None, None) => Ok(InferenceValue(None)),
            (Some(t), None) | (None, Some(t)) => Ok(InferenceValue(Some(t))),
            (Some(a), Some(b)) if a == b => Ok(InferenceValue(Some(a))),
            // When types conflict, prefer the first (or could return error)
            (Some(a), Some(_)) => Ok(InferenceValue(Some(a))),
        }
    }
}

/// Inference error
#[derive(Clone, Debug)]
pub enum InferenceError {
    /// Two incompatible types were unified
    Conflict(TypeId, TypeId),
    /// Inference variable was not resolved
    Unresolved(InferenceVar),
    /// Lower bound is not subtype of upper bound
    BoundsViolation {
        var: InferenceVar,
        lower: TypeId,
        upper: TypeId,
    },
}

/// Constraint set for an inference variable.
/// Tracks both lower bounds (L <: α) and upper bounds (α <: U).
#[derive(Clone, Debug, Default)]
pub struct ConstraintSet {
    /// Lower bounds: types that must be subtypes of this variable
    /// e.g., from argument types being assigned to a parameter
    pub lower_bounds: Vec<TypeId>,
    /// Upper bounds: types that this variable must be a subtype of
    /// e.g., from `extends` constraints on type parameters
    pub upper_bounds: Vec<TypeId>,
}

impl ConstraintSet {
    pub fn new() -> Self {
        ConstraintSet {
            lower_bounds: Vec::new(),
            upper_bounds: Vec::new(),
        }
    }

    /// Add a lower bound constraint: L <: α
    pub fn add_lower_bound(&mut self, ty: TypeId) {
        if !self.lower_bounds.contains(&ty) {
            self.lower_bounds.push(ty);
        }
    }

    /// Add an upper bound constraint: α <: U
    pub fn add_upper_bound(&mut self, ty: TypeId) {
        if !self.upper_bounds.contains(&ty) {
            self.upper_bounds.push(ty);
        }
    }

    /// Check if there are any constraints
    pub fn is_empty(&self) -> bool {
        self.lower_bounds.is_empty() && self.upper_bounds.is_empty()
    }
}

/// Type inference context for a single function call or expression.
pub struct InferenceContext<'a> {
    interner: &'a dyn TypeDatabase,
    /// Unification table for inference variables
    table: InPlaceUnificationTable<InferenceVar>,
    /// Map from type parameter names to inference variables
    type_params: Vec<(Arc<str>, InferenceVar)>,
    /// Constraints for each inference variable
    constraints: std::collections::HashMap<u32, ConstraintSet>,
}

impl<'a> InferenceContext<'a> {
    pub fn new(interner: &'a dyn TypeDatabase) -> Self {
        InferenceContext {
            interner,
            table: InPlaceUnificationTable::new(),
            type_params: Vec::new(),
            constraints: std::collections::HashMap::new(),
        }
    }

    /// Create a fresh inference variable
    pub fn fresh_var(&mut self) -> InferenceVar {
        self.table.new_key(InferenceValue(None))
    }

    /// Create an inference variable for a type parameter
    pub fn fresh_type_param(&mut self, name: Arc<str>) -> InferenceVar {
        let var = self.fresh_var();
        self.type_params.push((name, var));
        var
    }

    /// Look up an inference variable by type parameter name
    pub fn find_type_param(&self, name: &str) -> Option<InferenceVar> {
        self.type_params.iter()
            .find(|(n, _)| n.as_ref() == name)
            .map(|(_, v)| *v)
    }

    /// Probe the current value of an inference variable
    pub fn probe(&mut self, var: InferenceVar) -> Option<TypeId> {
        self.table.probe_value(var).0
    }

    /// Unify an inference variable with a concrete type
    pub fn unify_var_type(&mut self, var: InferenceVar, ty: TypeId) -> Result<(), InferenceError> {
        // Get the root variable
        let root = self.table.find(var);

        // Check current value
        match self.table.probe_value(root).0 {
            None => {
                // No value yet, assign it
                self.table.union_value(root, InferenceValue(Some(ty)));
                Ok(())
            }
            Some(existing) => {
                // Check compatibility
                if self.types_compatible(existing, ty) {
                    Ok(())
                } else {
                    Err(InferenceError::Conflict(existing, ty))
                }
            }
        }
    }

    /// Unify two inference variables
    pub fn unify_vars(&mut self, a: InferenceVar, b: InferenceVar) -> Result<(), InferenceError> {
        self.table.unify_var_var(a, b).map_err(|_| {
            InferenceError::Conflict(TypeId::ERROR, TypeId::ERROR)
        })?;
        Ok(())
    }

    /// Check if two types are compatible for unification
    fn types_compatible(&self, a: TypeId, b: TypeId) -> bool {
        if a == b {
            return true;
        }

        // Any is compatible with everything
        if a == TypeId::ANY || b == TypeId::ANY {
            return true;
        }

        // Unknown is compatible with everything
        if a == TypeId::UNKNOWN || b == TypeId::UNKNOWN {
            return true;
        }

        // Never is compatible with everything
        if a == TypeId::NEVER || b == TypeId::NEVER {
            return true;
        }

        false
    }

    /// Resolve all type parameters to concrete types
    pub fn resolve_all(&mut self) -> Result<Vec<(Arc<str>, TypeId)>, InferenceError> {
        // Clone type_params to avoid borrow conflict
        let type_params: Vec<_> = self.type_params.clone();
        let mut results = Vec::new();
        for (name, var) in type_params {
            match self.probe(var) {
                Some(ty) => results.push((name.clone(), ty)),
                None => return Err(InferenceError::Unresolved(var)),
            }
        }
        Ok(results)
    }

    /// Get the interner reference
    #[allow(dead_code)]
    pub fn interner(&self) -> &dyn TypeDatabase {
        self.interner
    }

    // =========================================================================
    // Constraint Collection
    // =========================================================================

    /// Add a lower bound constraint: ty <: var
    /// This is used when an argument type flows into a type parameter.
    pub fn add_lower_bound(&mut self, var: InferenceVar, ty: TypeId) {
        let root = self.table.find(var);
        self.constraints
            .entry(root.0)
            .or_insert_with(ConstraintSet::new)
            .add_lower_bound(ty);
    }

    /// Add an upper bound constraint: var <: ty
    /// This is used for `extends` constraints on type parameters.
    pub fn add_upper_bound(&mut self, var: InferenceVar, ty: TypeId) {
        let root = self.table.find(var);
        self.constraints
            .entry(root.0)
            .or_insert_with(ConstraintSet::new)
            .add_upper_bound(ty);
    }

    /// Get the constraints for a variable
    pub fn get_constraints(&mut self, var: InferenceVar) -> Option<&ConstraintSet> {
        let root = self.table.find(var);
        self.constraints.get(&root.0)
    }

    /// Collect a constraint from an assignment: source flows into target
    /// If target is an inference variable, source becomes a lower bound.
    /// If source is an inference variable, target becomes an upper bound.
    pub fn collect_constraint(&mut self, _source: TypeId, _target: TypeId) {
        // Check if target is an inference variable (via TypeKey lookup)
        // For now, we rely on the caller to call add_lower_bound/add_upper_bound directly
        // This is a placeholder for more sophisticated constraint collection
    }

    // =========================================================================
    // Bounds Checking and Resolution
    // =========================================================================

    /// Resolve an inference variable using its collected constraints.
    ///
    /// Algorithm:
    /// 1. If already unified to a concrete type, return that
    /// 2. Otherwise, compute the best common type from lower bounds
    /// 3. Validate against upper bounds
    /// 4. If no lower bounds, use the constraint (upper bound) or default
    pub fn resolve_with_constraints(&mut self, var: InferenceVar) -> Result<TypeId, InferenceError> {
        let root = self.table.find(var);

        // Check if already resolved
        if let Some(ty) = self.probe(var) {
            return Ok(ty);
        }

        // Get constraints
        let constraints = self.constraints.get(&root.0).cloned().unwrap_or_default();

        // Compute result from constraints
        let result = if !constraints.lower_bounds.is_empty() {
            // Best common type: union of all lower bounds
            self.best_common_type(&constraints.lower_bounds)
        } else if !constraints.upper_bounds.is_empty() {
            // No lower bounds, use first upper bound (constraint)
            constraints.upper_bounds[0]
        } else {
            // No constraints at all - return unknown
            TypeId::UNKNOWN
        };

        // Validate against upper bounds
        for &upper in &constraints.upper_bounds {
            if !self.is_subtype(result, upper) {
                return Err(InferenceError::BoundsViolation {
                    var,
                    lower: result,
                    upper,
                });
            }
        }

        // Store the result
        self.table.union_value(root, InferenceValue(Some(result)));

        Ok(result)
    }

    /// Resolve all type parameters using constraints.
    pub fn resolve_all_with_constraints(&mut self) -> Result<Vec<(Arc<str>, TypeId)>, InferenceError> {
        let type_params: Vec<_> = self.type_params.clone();
        let mut results = Vec::new();

        for (name, var) in type_params {
            let ty = self.resolve_with_constraints(var)?;
            results.push((name, ty));
        }

        Ok(results)
    }

    // =========================================================================
    // Best Common Type
    // =========================================================================

    /// Calculate the best common type from a set of types.
    /// This is the union of all types (widening).
    pub fn best_common_type(&self, types: &[TypeId]) -> TypeId {
        if types.is_empty() {
            return TypeId::UNKNOWN;
        }
        if types.len() == 1 {
            return types[0];
        }

        // Filter out duplicates and special types
        let mut unique: Vec<TypeId> = Vec::new();
        for &ty in types {
            if ty == TypeId::NEVER {
                continue; // never doesn't contribute to union
            }
            if !unique.contains(&ty) {
                unique.push(ty);
            }
        }

        if unique.is_empty() {
            return TypeId::NEVER;
        }
        if unique.len() == 1 {
            return unique[0];
        }

        // Create union of all types
        self.interner.union(unique)
    }

    /// Simple subtype check for bounds validation.
    /// Uses a simplified check - for full checking, use SubtypeChecker.
    fn is_subtype(&self, source: TypeId, target: TypeId) -> bool {
        // Same type
        if source == target {
            return true;
        }

        // never <: T for all T
        if source == TypeId::NEVER {
            return true;
        }

        // T <: unknown for all T
        if target == TypeId::UNKNOWN {
            return true;
        }

        // any <: T and T <: any
        if source == TypeId::ANY || target == TypeId::ANY {
            return true;
        }

        // Check if source is literal of target intrinsic
        if let Some(TypeKey::Literal(lit)) = self.interner.lookup(source) {
            match (lit, target) {
                (LiteralValue::String(_), t) if t == TypeId::STRING => return true,
                (LiteralValue::Number(_), t) if t == TypeId::NUMBER => return true,
                (LiteralValue::Boolean(_), t) if t == TypeId::BOOLEAN => return true,
                (LiteralValue::BigInt(_), t) if t == TypeId::BIGINT => return true,
                _ => {}
            }
        }

        // Check union membership
        if let Some(TypeKey::Union(members)) = self.interner.lookup(target) {
            return members.contains(&source);
        }

        false
    }
}

#[cfg(test)]
#[path = "infer_tests.rs"]
mod tests;
