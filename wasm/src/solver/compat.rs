//! TypeScript compatibility layer for assignability rules.

use crate::solver::subtype::{NoopResolver, SubtypeChecker, SubtypeFailureReason, TypeResolver};
use crate::solver::types::{PropertyInfo, TypeId, TypeKey};
use crate::solver::{AssignabilityChecker, TypeDatabase};
use rustc_hash::FxHashMap;

#[cfg(test)]
use crate::solver::TypeInterner;

/// Compatibility checker that applies TypeScript's unsound rules
/// before delegating to the structural subtype engine.
pub struct CompatChecker<'a, R: TypeResolver = NoopResolver> {
    interner: &'a dyn TypeDatabase,
    subtype: SubtypeChecker<'a, R>,
    strict_function_types: bool,
    strict_null_checks: bool,
    no_unchecked_indexed_access: bool,
    exact_optional_property_types: bool,
    cache: FxHashMap<(TypeId, TypeId), bool>,
}

impl<'a> CompatChecker<'a, NoopResolver> {
    /// Create a new compatibility checker without a resolver.
    pub fn new(interner: &'a dyn TypeDatabase) -> CompatChecker<'a, NoopResolver> {
        CompatChecker {
            interner,
            subtype: SubtypeChecker::new(interner),
            strict_function_types: false,
            strict_null_checks: true,
            no_unchecked_indexed_access: false,
            exact_optional_property_types: false,
            cache: FxHashMap::default(),
        }
    }
}

impl<'a, R: TypeResolver> CompatChecker<'a, R> {
    /// Create a new compatibility checker with a resolver.
    pub fn with_resolver(interner: &'a dyn TypeDatabase, resolver: &'a R) -> Self {
        CompatChecker {
            interner,
            subtype: SubtypeChecker::with_resolver(interner, resolver),
            strict_function_types: false,
            strict_null_checks: true,
            no_unchecked_indexed_access: false,
            exact_optional_property_types: false,
            cache: FxHashMap::default(),
        }
    }

    /// Configure strict function parameter checking.
    /// See https://github.com/microsoft/TypeScript/issues/18654.
    pub fn set_strict_function_types(&mut self, strict: bool) {
        if self.strict_function_types != strict {
            self.strict_function_types = strict;
            self.cache.clear();
        }
    }

    /// Configure strict null checks (legacy null/undefined assignability).
    pub fn set_strict_null_checks(&mut self, strict: bool) {
        if self.strict_null_checks != strict {
            self.strict_null_checks = strict;
            self.cache.clear();
        }
    }

    /// Configure unchecked indexed access (include `undefined` in `T[K]`).
    pub fn set_no_unchecked_indexed_access(&mut self, enabled: bool) {
        if self.no_unchecked_indexed_access != enabled {
            self.no_unchecked_indexed_access = enabled;
            self.cache.clear();
        }
    }

    /// Configure exact optional property types.
    /// See https://github.com/microsoft/TypeScript/issues/13195.
    pub fn set_exact_optional_property_types(&mut self, exact: bool) {
        if self.exact_optional_property_types != exact {
            self.exact_optional_property_types = exact;
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
            // `any` is the JS escape hatch (top + bottom). See https://github.com/microsoft/TypeScript/issues/10715.
            true
        } else if !self.strict_null_checks
            && (source == TypeId::NULL || source == TypeId::UNDEFINED)
        {
            true
        } else if target == TypeId::UNKNOWN {
            // `unknown` is top but not assignable to non-top types. See https://github.com/microsoft/TypeScript/issues/10715.
            true
        } else if source == TypeId::NEVER || source == TypeId::ERROR || target == TypeId::ERROR {
            true
        } else if source == TypeId::UNKNOWN {
            false
        } else if self.violates_weak_type(source, target) {
            false
        } else if self.is_empty_object_target(target) {
            // `{}` accepts any non-nullish value (including primitives). See https://github.com/microsoft/TypeScript/issues/60582.
            self.is_assignable_to_empty_object(source)
        } else {
            self.configure_subtype(self.strict_function_types);
            self.subtype.is_subtype_of(source, target)
        };

        self.cache.insert(key, result);
        result
    }

    fn is_assignable_strict(&mut self, source: TypeId, target: TypeId) -> bool {
        if source == target {
            return true;
        }
        if source == TypeId::ANY || target == TypeId::ANY {
            return true;
        }
        if !self.strict_null_checks && (source == TypeId::NULL || source == TypeId::UNDEFINED) {
            return true;
        }
        if target == TypeId::UNKNOWN {
            return true;
        }
        if source == TypeId::NEVER || source == TypeId::ERROR || target == TypeId::ERROR {
            return true;
        }
        if source == TypeId::UNKNOWN {
            return false;
        }
        if self.is_empty_object_target(target) {
            return self.is_assignable_to_empty_object(source);
        }

        let prev = self.subtype.strict_function_types;
        self.configure_subtype(true);
        let result = self.subtype.is_subtype_of(source, target);
        self.subtype.strict_function_types = prev;
        result
    }

    /// Explain why `source` is not assignable to `target` using TS compatibility rules.
    pub fn explain_failure(&mut self, source: TypeId, target: TypeId) -> Option<SubtypeFailureReason> {
        if source == target || source == TypeId::ANY || target == TypeId::ANY || target == TypeId::UNKNOWN {
            return None;
        }
        if !self.strict_null_checks && (source == TypeId::NULL || source == TypeId::UNDEFINED) {
            return None;
        }
        if source == TypeId::NEVER || source == TypeId::ERROR || target == TypeId::ERROR {
            return None;
        }
        if self.violates_weak_type(source, target) {
            return Some(SubtypeFailureReason::NoCommonProperties {
                source_type: source,
                target_type: target,
            });
        }
        if self.is_empty_object_target(target) {
            if self.is_assignable_to_empty_object(source) {
                return None;
            }
        }

        self.configure_subtype(self.strict_function_types);
        self.subtype.explain_failure(source, target)
    }

    fn configure_subtype(&mut self, strict_function_types: bool) {
        self.subtype.strict_function_types = strict_function_types;
        self.subtype.allow_void_return = true;
        self.subtype.allow_bivariant_rest = true;
        self.subtype.exact_optional_property_types = self.exact_optional_property_types;
        self.subtype.strict_null_checks = self.strict_null_checks;
        self.subtype.no_unchecked_indexed_access = self.no_unchecked_indexed_access;
    }

    fn violates_weak_type(&self, source: TypeId, target: TypeId) -> bool {
        let target_key = match self.interner.lookup(target) {
            Some(key) => key,
            None => return false,
        };

        let target_shape = match &target_key {
            TypeKey::Object(shape_id) => self.interner.object_shape(*shape_id),
            TypeKey::ObjectWithIndex(shape_id) => {
                let shape = self.interner.object_shape(*shape_id);
                if shape.string_index.is_some() || shape.number_index.is_some() {
                    return false;
                }
                shape
            }
            _ => return false,
        };

        let target_props = target_shape.properties.as_slice();
        if target_props.is_empty() || target_props.iter().any(|prop| !prop.optional) {
            return false;
        }

        self.violates_weak_type_with_target_props(source, target_props)
    }

    fn violates_weak_type_with_target_props(
        &self,
        source: TypeId,
        target_props: &[PropertyInfo],
    ) -> bool {
        let source_key = match self.interner.lookup(source) {
            Some(key) => key,
            None => return false,
        };

        match &source_key {
            TypeKey::Object(shape_id) => {
                let shape = self.interner.object_shape(*shape_id);
                !self.has_common_property(shape.properties.as_slice(), target_props)
            }
            TypeKey::ObjectWithIndex(shape_id) => {
                let shape = self.interner.object_shape(*shape_id);
                !self.has_common_property(shape.properties.as_slice(), target_props)
            }
            TypeKey::Union(members) => {
                let members = self.interner.type_list(*members);
                members
                    .iter()
                    .any(|member| self.violates_weak_type_with_target_props(*member, target_props))
            }
            _ => false,
        }
    }

    fn has_common_property(&self, source_props: &[PropertyInfo], target_props: &[PropertyInfo]) -> bool {
        let mut source_idx = 0;
        let mut target_idx = 0;

        while source_idx < source_props.len() && target_idx < target_props.len() {
            let source_name = source_props[source_idx].name;
            let target_name = target_props[target_idx].name;
            if source_name == target_name {
                return true;
            }
            if source_name < target_name {
                source_idx += 1;
            } else {
                target_idx += 1;
            }
        }

        false
    }

    fn is_empty_object_target(&self, target: TypeId) -> bool {
        match self.interner.lookup(target) {
            Some(TypeKey::Object(shape_id)) => {
                let shape = self.interner.object_shape(shape_id);
                shape.properties.is_empty()
            }
            Some(TypeKey::ObjectWithIndex(shape_id)) => {
                let shape = self.interner.object_shape(shape_id);
                shape.properties.is_empty()
                    && shape.string_index.is_none()
                    && shape.number_index.is_none()
            }
            _ => false,
        }
    }

    fn is_assignable_to_empty_object(&self, source: TypeId) -> bool {
        if source == TypeId::ANY || source == TypeId::NEVER || source == TypeId::ERROR {
            return true;
        }
        if !self.strict_null_checks
            && (source == TypeId::NULL || source == TypeId::UNDEFINED)
        {
            return true;
        }
        if source == TypeId::UNKNOWN
            || source == TypeId::NULL
            || source == TypeId::UNDEFINED
            || source == TypeId::VOID
        {
            return false;
        }

        let key = match self.interner.lookup(source) {
            Some(key) => key,
            None => return false,
        };

        match &key {
            TypeKey::Union(members) => {
                let members = self.interner.type_list(*members);
                members
                    .iter()
                    .all(|member| self.is_assignable_to_empty_object(*member))
            }
            TypeKey::Intersection(members) => {
                let members = self.interner.type_list(*members);
                members
                    .iter()
                    .any(|member| self.is_assignable_to_empty_object(*member))
            }
            TypeKey::TypeParameter(param) => match param.constraint {
                Some(constraint) => self.is_assignable_to_empty_object(constraint),
                None => false,
            },
            _ => true,
        }
    }
}

impl<'a, R: TypeResolver> AssignabilityChecker for CompatChecker<'a, R> {
    fn is_assignable_to(&mut self, source: TypeId, target: TypeId) -> bool {
        self.is_assignable(source, target)
    }

    fn is_assignable_to_strict(&mut self, source: TypeId, target: TypeId) -> bool {
        self.is_assignable_strict(source, target)
    }
}

#[cfg(test)]
#[path = "compat_tests.rs"]
mod tests;
