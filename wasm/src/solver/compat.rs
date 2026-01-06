//! TypeScript compatibility layer for assignability rules.

use crate::solver::intern::TypeInterner;
use crate::solver::subtype::{NoopResolver, SubtypeChecker, TypeResolver};
use crate::solver::types::TypeId;

/// Compatibility checker that applies TypeScript's unsound rules
/// before delegating to the structural subtype engine.
pub struct CompatChecker<'a, R: TypeResolver = NoopResolver> {
    subtype: SubtypeChecker<'a, R>,
    strict_function_types: bool,
}

impl<'a> CompatChecker<'a, NoopResolver> {
    /// Create a new compatibility checker without a resolver.
    pub fn new(interner: &'a TypeInterner) -> CompatChecker<'a, NoopResolver> {
        CompatChecker {
            subtype: SubtypeChecker::new(interner),
            strict_function_types: false,
        }
    }
}

impl<'a, R: TypeResolver> CompatChecker<'a, R> {
    /// Create a new compatibility checker with a resolver.
    pub fn with_resolver(interner: &'a TypeInterner, resolver: &'a R) -> Self {
        CompatChecker {
            subtype: SubtypeChecker::with_resolver(interner, resolver),
            strict_function_types: false,
        }
    }

    /// Configure strict function parameter checking.
    pub fn set_strict_function_types(&mut self, strict: bool) {
        self.strict_function_types = strict;
    }

    /// Check if `source` is assignable to `target` using TS compatibility rules.
    pub fn is_assignable(&mut self, source: TypeId, target: TypeId) -> bool {
        if source == target {
            return true;
        }

        if source == TypeId::ANY || target == TypeId::ANY {
            return true;
        }

        if target == TypeId::UNKNOWN {
            return true;
        }

        if source == TypeId::UNKNOWN {
            return false;
        }

        self.subtype.strict_function_types = self.strict_function_types;
        self.subtype.is_subtype_of(source, target)
    }
}

#[cfg(test)]
#[path = "compat_tests.rs"]
mod tests;
