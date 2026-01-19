//! Union Type Solver
//!
//! Handles union type operations:
//! - Union type creation and simplification
//! - Conditional type distribution over unions
//! - Large union optimization
//! - Union normalization

use crate::types::intersection::Type;
use std::collections::{HashMap, HashSet};

/// Union solver options
#[derive(Debug, Clone)]
pub struct UnionSolverOptions {
    /// Maximum number of union members before applying optimization
    pub large_union_threshold: usize,
    /// Whether to deduplicate types in unions
    pub deduplicate: bool,
    /// Whether to remove never types
    pub remove_never: bool,
    /// Whether to simplify literals with base types
    pub simplify_literals: bool,
}

impl Default for UnionSolverOptions {
    fn default() -> Self {
        UnionSolverOptions {
            large_union_threshold: 100,
            deduplicate: true,
            remove_never: true,
            simplify_literals: true,
        }
    }
}

/// Union type solver
pub struct UnionSolver {
    options: UnionSolverOptions,
    /// Cache for normalized unions (for large unions)
    cache: HashMap<String, Type>,
    /// Maximum recursion depth
    max_depth: usize,
    current_depth: usize,
}

impl UnionSolver {
    pub fn new(options: UnionSolverOptions) -> Self {
        UnionSolver {
            options,
            cache: HashMap::new(),
            max_depth: 50,
            current_depth: 0,
        }
    }

    /// Create a union type from multiple types
    pub fn create_union(&mut self, types: Vec<Type>) -> Type {
        if types.is_empty() {
            return Type::Never;
        }
        if types.len() == 1 {
            return types.into_iter().next().unwrap();
        }

        // For large unions, check cache first
        if types.len() > self.options.large_union_threshold {
            let cache_key = self.compute_cache_key(&types);
            if let Some(cached) = self.cache.get(&cache_key) {
                return cached.clone();
            }
        }

        let result = self.simplify_union(types);

        result
    }

    /// Simplify a union type
    pub fn simplify_union(&mut self, types: Vec<Type>) -> Type {
        if self.current_depth >= self.max_depth {
            return Type::Union(types);
        }
        self.current_depth += 1;

        // Step 1: Flatten nested unions
        let flattened = self.flatten_union(types);

        // Step 2: Remove never types
        let mut filtered = if self.options.remove_never {
            flattened.into_iter().filter(|t| !t.is_never()).collect()
        } else {
            flattened
        };

        // Step 3: Check for any/unknown absorption
        if filtered.iter().any(|t| t.is_any()) {
            self.current_depth -= 1;
            return Type::Any;
        }
        if filtered.iter().any(|t| t.is_unknown()) {
            self.current_depth -= 1;
            return Type::Unknown;
        }

        // Step 4: Deduplicate
        if self.options.deduplicate {
            filtered = self.deduplicate_types(filtered);
        }

        // Step 5: Simplify literals with base types
        if self.options.simplify_literals {
            filtered = self.simplify_literals(filtered);
        }

        // Step 6: For large unions, apply additional optimizations
        if filtered.len() > self.options.large_union_threshold {
            filtered = self.optimize_large_union(filtered);
        }

        self.current_depth -= 1;

        // Final result
        if filtered.is_empty() {
            Type::Never
        } else if filtered.len() == 1 {
            filtered.into_iter().next().unwrap()
        } else {
            Type::Union(filtered)
        }
    }

    /// Flatten nested unions
    fn flatten_union(&self, types: Vec<Type>) -> Vec<Type> {
        let mut result = Vec::new();
        for ty in types {
            match ty {
                Type::Union(inner) => result.extend(inner),
                other => result.push(other),
            }
        }
        result
    }

    /// Remove duplicate types
    fn deduplicate_types(&self, types: Vec<Type>) -> Vec<Type> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for ty in types {
            let key = self.type_key(&ty);
            if !seen.contains(&key) {
                seen.insert(key);
                result.push(ty);
            }
        }

        result
    }

    /// Simplify literals when their base type is present
    fn simplify_literals(&self, types: Vec<Type>) -> Vec<Type> {
        let has_string = types.iter().any(|t| matches!(t, Type::String));
        let has_number = types.iter().any(|t| matches!(t, Type::Number));
        let has_boolean = types.iter().any(|t| matches!(t, Type::Boolean));
        let _has_bigint = types.iter().any(|t| matches!(t, Type::BigInt));

        types.into_iter().filter(|t| {
            match t {
                Type::StringLiteral(_) if has_string => false,
                Type::NumberLiteral(_) if has_number => false,
                Type::BooleanLiteral(_) if has_boolean => false,
                _ => true,
            }
        }).collect()
    }

    /// Optimize large unions
    fn optimize_large_union(&mut self, types: Vec<Type>) -> Vec<Type> {
        // Group by type kind for more efficient processing
        let mut primitives = Vec::new();
        let mut literals = Vec::new();
        let mut objects = Vec::new();
        let mut functions = Vec::new();
        let mut others = Vec::new();

        for ty in types {
            match ty {
                Type::String | Type::Number | Type::Boolean | Type::BigInt | Type::Symbol |
                Type::Null | Type::Undefined | Type::Void => primitives.push(ty),
                Type::StringLiteral(_) | Type::NumberLiteral(_) | Type::BooleanLiteral(_) => literals.push(ty),
                Type::Object(_) => objects.push(ty),
                Type::Function(_) => functions.push(ty),
                _ => others.push(ty),
            }
        }

        // Combine groups back together
        let mut result = Vec::new();
        result.extend(primitives);

        // For literals, check if we can collapse them
        if literals.len() > self.options.large_union_threshold / 2 {
            // Many string literals -> use string type
            let string_literal_count = literals.iter().filter(|t| matches!(t, Type::StringLiteral(_))).count();
            let number_literal_count = literals.iter().filter(|t| matches!(t, Type::NumberLiteral(_))).count();

            if string_literal_count > 50 && !result.iter().any(|t| matches!(t, Type::String)) {
                result.push(Type::String);
                literals.retain(|t| !matches!(t, Type::StringLiteral(_)));
            }
            if number_literal_count > 50 && !result.iter().any(|t| matches!(t, Type::Number)) {
                result.push(Type::Number);
                literals.retain(|t| !matches!(t, Type::NumberLiteral(_)));
            }
        }

        result.extend(literals);
        result.extend(objects);
        result.extend(functions);
        result.extend(others);

        // Final deduplication
        self.deduplicate_types(result)
    }

    /// Generate a cache key for a set of types
    fn compute_cache_key(&self, types: &[Type]) -> String {
        let mut keys: Vec<String> = types.iter().map(|t| self.type_key(t)).collect();
        keys.sort();
        keys.join("|")
    }

    /// Generate a unique key for a type
    fn type_key(&self, ty: &Type) -> String {
        format!("{:?}", ty)
    }

    /// Distribute conditional type over union
    /// T extends U ? X : Y where T = A | B becomes (A extends U ? X : Y) | (B extends U ? X : Y)
    pub fn distribute_conditional_over_union(
        &mut self,
        check_type: &Type,
        extends_type: &Type,
        true_type: &Type,
        false_type: &Type,
    ) -> Type {
        match check_type {
            Type::Union(members) => {
                // Distribute over each union member
                let distributed: Vec<Type> = members.iter().map(|member| {
                    self.evaluate_conditional(member, extends_type, true_type, false_type)
                }).collect();

                self.create_union(distributed)
            }
            _ => self.evaluate_conditional(check_type, extends_type, true_type, false_type),
        }
    }

    /// Evaluate a conditional type
    fn evaluate_conditional(
        &mut self,
        check_type: &Type,
        extends_type: &Type,
        true_type: &Type,
        false_type: &Type,
    ) -> Type {
        // Simple structural check - in a full implementation this would be more sophisticated
        if self.is_subtype(check_type, extends_type) {
            true_type.clone()
        } else if self.definitely_not_subtype(check_type, extends_type) {
            false_type.clone()
        } else {
            // Unknown - return conditional type
            Type::Conditional {
                check_type: Box::new(check_type.clone()),
                extends_type: Box::new(extends_type.clone()),
                true_type: Box::new(true_type.clone()),
                false_type: Box::new(false_type.clone()),
            }
        }
    }

    /// Simple subtype check
    fn is_subtype(&self, a: &Type, b: &Type) -> bool {
        if a == b {
            return true;
        }

        match (a, b) {
            // Never is subtype of everything
            (Type::Never, _) => true,
            // Everything is subtype of unknown
            (_, Type::Unknown) => true,
            // Any is weird but works both ways
            (Type::Any, _) | (_, Type::Any) => true,
            // Literals are subtypes of their base types
            (Type::StringLiteral(_), Type::String) => true,
            (Type::NumberLiteral(_), Type::Number) => true,
            (Type::BooleanLiteral(_), Type::Boolean) => true,
            // Undefined is subtype of void
            (Type::Undefined, Type::Void) => true,
            _ => false,
        }
    }

    /// Check if type is definitely NOT a subtype
    fn definitely_not_subtype(&self, a: &Type, b: &Type) -> bool {
        match (a, b) {
            // Different primitive types
            (Type::String, Type::Number) | (Type::Number, Type::String) => true,
            (Type::String, Type::Boolean) | (Type::Boolean, Type::String) => true,
            (Type::Number, Type::Boolean) | (Type::Boolean, Type::Number) => true,
            // String literal not subtype of number, etc.
            (Type::StringLiteral(_), Type::Number) => true,
            (Type::NumberLiteral(_), Type::String) => true,
            _ => false,
        }
    }

    /// Normalize a union type (canonical form)
    pub fn normalize_union(&mut self, ty: &Type) -> Type {
        match ty {
            Type::Union(types) => {
                let simplified = self.simplify_union(types.clone());
                match simplified {
                    Type::Union(mut types) => {
                        // Sort for canonical form
                        types.sort_by(|a, b| self.type_key(a).cmp(&self.type_key(b)));
                        Type::Union(types)
                    }
                    other => other,
                }
            }
            _ => ty.clone(),
        }
    }

    /// Widen a union of literals to their base types if there are many
    pub fn widen_union(&mut self, ty: &Type, threshold: usize) -> Type {
        match ty {
            Type::Union(types) => {
                let string_literal_count = types.iter().filter(|t| matches!(t, Type::StringLiteral(_))).count();
                let number_literal_count = types.iter().filter(|t| matches!(t, Type::NumberLiteral(_))).count();

                let mut result: Vec<Type> = types.iter().filter(|t| {
                    !matches!(t, Type::StringLiteral(_) | Type::NumberLiteral(_))
                }).cloned().collect();

                if string_literal_count >= threshold {
                    result.push(Type::String);
                } else {
                    result.extend(types.iter().filter(|t| matches!(t, Type::StringLiteral(_))).cloned());
                }

                if number_literal_count >= threshold {
                    result.push(Type::Number);
                } else {
                    result.extend(types.iter().filter(|t| matches!(t, Type::NumberLiteral(_))).cloned());
                }

                self.simplify_union(result)
            }
            _ => ty.clone(),
        }
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for UnionSolver {
    fn default() -> Self {
        Self::new(UnionSolverOptions::default())
    }
}

/// Create a union from two types
pub fn union_types(a: Type, b: Type) -> Type {
    let mut solver = UnionSolver::default();
    solver.create_union(vec![a, b])
}

/// Create a union from multiple types
pub fn union_all(types: Vec<Type>) -> Type {
    let mut solver = UnionSolver::default();
    solver.create_union(types)
}

/// Normalize a union type
pub fn normalize_union(ty: &Type) -> Type {
    let mut solver = UnionSolver::default();
    solver.normalize_union(ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_simple_union() {
        let mut solver = UnionSolver::default();
        let result = solver.create_union(vec![Type::String, Type::Number]);

        match result {
            Type::Union(types) => {
                assert_eq!(types.len(), 2);
                assert!(types.contains(&Type::String));
                assert!(types.contains(&Type::Number));
            }
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_empty_union_is_never() {
        let mut solver = UnionSolver::default();
        let result = solver.create_union(vec![]);
        assert!(result.is_never());
    }

    #[test]
    fn test_single_type_union() {
        let mut solver = UnionSolver::default();
        let result = solver.create_union(vec![Type::String]);
        assert_eq!(result, Type::String);
    }

    #[test]
    fn test_union_removes_never() {
        let mut solver = UnionSolver::default();
        let result = solver.create_union(vec![Type::String, Type::Never, Type::Number]);

        match result {
            Type::Union(types) => {
                assert_eq!(types.len(), 2);
                assert!(!types.contains(&Type::Never));
            }
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_union_any_absorbs() {
        let mut solver = UnionSolver::default();
        let result = solver.create_union(vec![Type::String, Type::Any, Type::Number]);
        assert!(result.is_any());
    }

    #[test]
    fn test_union_unknown_absorbs() {
        let mut solver = UnionSolver::default();
        let result = solver.create_union(vec![Type::String, Type::Unknown, Type::Number]);
        assert!(result.is_unknown());
    }

    #[test]
    fn test_union_deduplicates() {
        let mut solver = UnionSolver::default();
        let result = solver.create_union(vec![Type::String, Type::Number, Type::String]);

        match result {
            Type::Union(types) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_union_flattens() {
        let mut solver = UnionSolver::default();
        let inner = Type::Union(vec![Type::Number, Type::Boolean]);
        let result = solver.create_union(vec![Type::String, inner]);

        match result {
            Type::Union(types) => {
                assert_eq!(types.len(), 3);
            }
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_simplify_literals_with_base() {
        let mut solver = UnionSolver::default();
        let result = solver.create_union(vec![
            Type::String,
            Type::StringLiteral("hello".to_string()),
        ]);

        assert_eq!(result, Type::String);
    }

    #[test]
    fn test_distribute_conditional_over_union() {
        let mut solver = UnionSolver::default();

        // T extends string ? "yes" : "no" where T = string | number
        let result = solver.distribute_conditional_over_union(
            &Type::Union(vec![Type::String, Type::Number]),
            &Type::String,
            &Type::StringLiteral("yes".to_string()),
            &Type::StringLiteral("no".to_string()),
        );

        // Should be "yes" | "no"
        match result {
            Type::Union(types) => {
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_normalize_union() {
        let mut solver = UnionSolver::default();
        let union = Type::Union(vec![Type::Number, Type::String, Type::Number]);

        let normalized = solver.normalize_union(&union);

        match normalized {
            Type::Union(types) => {
                // Deduplicated and sorted
                assert_eq!(types.len(), 2);
            }
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_widen_union() {
        let mut solver = UnionSolver::default();

        // Create union with many string literals
        let mut types = vec![Type::Number];
        for i in 0..10 {
            types.push(Type::StringLiteral(format!("str{}", i)));
        }
        let union = Type::Union(types);

        let widened = solver.widen_union(&union, 5);

        match widened {
            Type::Union(types) => {
                // String literals should be widened to string
                assert!(types.contains(&Type::String));
                assert!(types.contains(&Type::Number));
                // No string literals should remain
                assert!(!types.iter().any(|t| matches!(t, Type::StringLiteral(_))));
            }
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_union_types_helper() {
        let result = union_types(Type::String, Type::Number);
        match result {
            Type::Union(types) => assert_eq!(types.len(), 2),
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_union_all_helper() {
        let result = union_all(vec![Type::String, Type::Number, Type::Boolean]);
        match result {
            Type::Union(types) => assert_eq!(types.len(), 3),
            _ => panic!("Expected union type"),
        }
    }

    #[test]
    fn test_only_never_becomes_never() {
        let mut solver = UnionSolver::default();
        let result = solver.create_union(vec![Type::Never, Type::Never]);
        assert!(result.is_never());
    }
}
