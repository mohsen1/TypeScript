//! TypeScript compatibility layer for assignability rules.

use crate::solver::intern::TypeInterner;
use crate::solver::subtype::{NoopResolver, SubtypeChecker, SubtypeFailureReason, TypeResolver};
use crate::solver::types::TypeId;
use rustc_hash::FxHashMap;

/// Compatibility checker that applies TypeScript's unsound rules
/// before delegating to the structural subtype engine.
pub struct CompatChecker<'a, R: TypeResolver = NoopResolver> {
    subtype: SubtypeChecker<'a, R>,
    strict_function_types: bool,
    cache: FxHashMap<(TypeId, TypeId), bool>,
}

impl<'a> CompatChecker<'a, NoopResolver> {
    /// Create a new compatibility checker without a resolver.
    pub fn new(interner: &'a TypeInterner) -> CompatChecker<'a, NoopResolver> {
        CompatChecker {
            subtype: SubtypeChecker::new(interner),
            strict_function_types: false,
            cache: FxHashMap::default(),
        }
    }
}

impl<'a, R: TypeResolver> CompatChecker<'a, R> {
    /// Create a new compatibility checker with a resolver.
    pub fn with_resolver(interner: &'a TypeInterner, resolver: &'a R) -> Self {
        CompatChecker {
            subtype: SubtypeChecker::with_resolver(interner, resolver),
            strict_function_types: false,
            cache: FxHashMap::default(),
        }
    }

    /// Configure strict function parameter checking.
    pub fn set_strict_function_types(&mut self, strict: bool) {
        if self.strict_function_types != strict {
            self.strict_function_types = strict;
            self.cache.clear();
        }
    }

    /// Check if `source` is assignable to `target` using TS compatibility rules.
    pub fn is_assignable(&mut self, source: TypeId, target: TypeId) -> bool {
        let key = (source, target);
        if let Some(&cached) = self.cache.get(&key) {
            return cached;
        }

        let result = if source == target {
            true
        } else if source == TypeId::ANY || target == TypeId::ANY {
            true
        } else if target == TypeId::UNKNOWN {
            true
        } else if source == TypeId::UNKNOWN {
            false
        } else {
            self.subtype.strict_function_types = self.strict_function_types;
            self.subtype.allow_void_return = true;
            self.subtype.is_subtype_of(source, target)
        };

        self.cache.insert(key, result);
        result
    }

    /// Explain why `source` is not assignable to `target` using TS compatibility rules.
    pub fn explain_failure(&mut self, source: TypeId, target: TypeId) -> Option<SubtypeFailureReason> {
        self.subtype.strict_function_types = self.strict_function_types;
        self.subtype.allow_void_return = true;
        self.subtype.explain_failure(source, target)
    }
}

#[cfg(test)]
#[path = "compat_tests.rs"]
mod tests;
