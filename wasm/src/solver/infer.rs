//! Type inference engine using Union-Find.
//!
//! This module implements type inference for generic functions using
//! the `ena` crate's Union-Find data structure.
//!
//! Key features:
//! - Inference variables for generic type parameters
//! - Constraint collection during type checking
//! - Efficient unification with path compression

use ena::unify::{InPlaceUnificationTable, UnifyKey, UnifyValue, NoError};
use std::sync::Arc;
use crate::solver::types::*;
use crate::solver::intern::TypeInterner;

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
#[derive(Copy, Clone, Debug)]
pub enum InferenceError {
    /// Two incompatible types were unified
    Conflict(TypeId, TypeId),
    /// Inference variable was not resolved
    Unresolved(InferenceVar),
}

/// Type inference context for a single function call or expression.
pub struct InferenceContext<'a> {
    interner: &'a TypeInterner,
    /// Unification table for inference variables
    table: InPlaceUnificationTable<InferenceVar>,
    /// Map from type parameter names to inference variables
    type_params: Vec<(Arc<str>, InferenceVar)>,
}

impl<'a> InferenceContext<'a> {
    pub fn new(interner: &'a TypeInterner) -> Self {
        InferenceContext {
            interner,
            table: InPlaceUnificationTable::new(),
            type_params: Vec::new(),
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
    pub fn interner(&self) -> &TypeInterner {
        self.interner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_basic() {
        let interner = TypeInterner::new();
        let mut ctx = InferenceContext::new(&interner);

        // Create inference variable
        let var = ctx.fresh_var();

        // Should start unresolved
        assert!(ctx.probe(var).is_none());

        // Unify with string
        ctx.unify_var_type(var, TypeId::STRING).unwrap();

        // Should now be string
        assert_eq!(ctx.probe(var), Some(TypeId::STRING));
    }

    #[test]
    fn test_inference_type_param() {
        let interner = TypeInterner::new();
        let mut ctx = InferenceContext::new(&interner);

        // Create type parameter T
        let var_t = ctx.fresh_type_param(Arc::from("T"));

        // Look it up
        let found = ctx.find_type_param("T");
        assert_eq!(found, Some(var_t));

        // Not found
        let not_found = ctx.find_type_param("U");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_inference_conflict() {
        let interner = TypeInterner::new();
        let mut ctx = InferenceContext::new(&interner);

        let var = ctx.fresh_var();

        // Unify with string
        ctx.unify_var_type(var, TypeId::STRING).unwrap();

        // Try to unify with number - should fail
        let result = ctx.unify_var_type(var, TypeId::NUMBER);
        assert!(result.is_err());
    }
}
