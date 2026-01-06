//! Structural subtype checking.
//!
//! This module implements the core logic engine for TypeScript's structural
//! subtyping. It uses coinductive semantics to handle recursive types.
//!
//! Key features:
//! - O(1) equality check via TypeId comparison
//! - Cycle detection for recursive types (coinductive)
//! - Set-theoretic operations for unions and intersections
//! - TypeResolver trait for lazy symbol resolution

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

/// Trait for resolving type references to their structural types.
/// This allows the SubtypeChecker to lazily resolve Ref types
/// without being tightly coupled to the binder/checker.
pub trait TypeResolver {
    /// Resolve a symbol reference to its structural type.
    /// Returns None if the symbol cannot be resolved.
    fn resolve_ref(&self, symbol: SymbolRef, interner: &TypeInterner) -> Option<TypeId>;
}

/// A no-op resolver that doesn't resolve any references.
/// Useful for tests or when symbol resolution isn't needed.
pub struct NoopResolver;

impl TypeResolver for NoopResolver {
    fn resolve_ref(&self, _symbol: SymbolRef, _interner: &TypeInterner) -> Option<TypeId> {
        None
    }
}

/// A type environment that maps symbol refs to their resolved types.
/// This is populated before type checking and passed to the SubtypeChecker.
#[derive(Clone, Debug, Default)]
pub struct TypeEnvironment {
    /// Maps symbol references to their resolved structural types.
    types: std::collections::HashMap<u32, TypeId>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        TypeEnvironment {
            types: std::collections::HashMap::new(),
        }
    }

    /// Register a symbol's resolved type.
    pub fn insert(&mut self, symbol: SymbolRef, type_id: TypeId) {
        self.types.insert(symbol.0, type_id);
    }

    /// Get a symbol's resolved type.
    pub fn get(&self, symbol: SymbolRef) -> Option<TypeId> {
        self.types.get(&symbol.0).copied()
    }

    /// Check if the environment contains a symbol.
    pub fn contains(&self, symbol: SymbolRef) -> bool {
        self.types.contains_key(&symbol.0)
    }

    /// Number of resolved types.
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}

impl TypeResolver for TypeEnvironment {
    fn resolve_ref(&self, symbol: SymbolRef, _interner: &TypeInterner) -> Option<TypeId> {
        self.get(symbol)
    }
}

/// Subtype checking context.
/// Maintains the "seen" set for cycle detection.
pub struct SubtypeChecker<'a, R: TypeResolver = NoopResolver> {
    interner: &'a TypeInterner,
    resolver: &'a R,
    /// Active subtype pairs being checked (for cycle detection)
    in_progress: HashSet<(TypeId, TypeId)>,
    /// Cache of resolved Ref types (for future use in Ref resolution optimization)
    #[allow(dead_code)]
    ref_cache: HashSet<(SymbolRef, TypeId)>,
    /// Current recursion depth (for stack overflow prevention)
    depth: u32,
}

impl<'a> SubtypeChecker<'a, NoopResolver> {
    /// Create a new SubtypeChecker without a resolver (basic mode).
    pub fn new(interner: &'a TypeInterner) -> SubtypeChecker<'a, NoopResolver> {
        static NOOP: NoopResolver = NoopResolver;
        SubtypeChecker {
            interner,
            resolver: &NOOP,
            in_progress: HashSet::new(),
            ref_cache: HashSet::new(),
            depth: 0,
        }
    }
}

impl<'a, R: TypeResolver> SubtypeChecker<'a, R> {
    /// Create a new SubtypeChecker with a custom resolver.
    pub fn with_resolver(interner: &'a TypeInterner, resolver: &'a R) -> Self {
        SubtypeChecker {
            interner,
            resolver,
            in_progress: HashSet::new(),
            ref_cache: HashSet::new(),
            depth: 0,
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
        // Depth Check (stack overflow prevention)
        // =========================================================================

        if self.depth > 100 {
            // Recursion too deep - return provisional true to prevent stack overflow
            // This is a safety measure for deeply nested or expanding recursive types
            return SubtypeResult::Provisional;
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

        // Mark as in-progress and increment depth
        self.in_progress.insert(pair);
        self.depth += 1;

        // Do the actual check
        let result = self.check_subtype_inner(source, target);

        // Remove from in-progress and decrement depth
        self.depth -= 1;
        self.in_progress.remove(&pair);

        result
    }

    /// Inner subtype check (after cycle detection)
    fn check_subtype_inner(&mut self, source: TypeId, target: TypeId) -> SubtypeResult {
        // Evaluate meta-types (conditionals, index access, etc.) before comparing
        let source_eval = self.evaluate_type(source);
        let target_eval = self.evaluate_type(target);

        // If evaluation changed anything, recurse with the simplified types
        if source_eval != source || target_eval != target {
            return self.check_subtype(source_eval, target_eval);
        }

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

            // Object with index to object with index
            (TypeKey::ObjectWithIndex(s_shape), TypeKey::ObjectWithIndex(t_shape)) => {
                self.check_object_with_index_subtype(s_shape, t_shape)
            }

            // Object with index to simple object (index signatures ignored)
            (TypeKey::ObjectWithIndex(s_shape), TypeKey::Object(t_props)) => {
                self.check_object_subtype(&s_shape.properties, t_props)
            }

            // Simple object to object with index
            (TypeKey::Object(s_props), TypeKey::ObjectWithIndex(t_shape)) => {
                // All source properties must satisfy target's index signature
                self.check_object_to_indexed(s_props, t_shape)
            }

            // Function to function
            (TypeKey::Function(s_fn), TypeKey::Function(t_fn)) => {
                self.check_function_subtype(s_fn, t_fn)
            }

            // Callable to callable (overloaded signatures)
            (TypeKey::Callable(s_callable), TypeKey::Callable(t_callable)) => {
                self.check_callable_subtype(s_callable, t_callable)
            }

            // Function to callable (single signature to overloaded)
            (TypeKey::Function(s_fn), TypeKey::Callable(t_callable)) => {
                // A single function can match a callable if it satisfies all target call signatures
                for t_sig in &t_callable.call_signatures {
                    if !self.check_call_signature_subtype_fn(s_fn, t_sig).is_true() {
                        return SubtypeResult::False;
                    }
                }
                SubtypeResult::True
            }

            // Callable to function (overloaded to single)
            (TypeKey::Callable(s_callable), TypeKey::Function(t_fn)) => {
                // At least one source signature must match the target function
                for s_sig in &s_callable.call_signatures {
                    if self.check_call_signature_subtype_to_fn(s_sig, t_fn).is_true() {
                        return SubtypeResult::True;
                    }
                }
                SubtypeResult::False
            }

            // Reference types - try to resolve and compare structurally
            (TypeKey::Ref(s_sym), TypeKey::Ref(t_sym)) => {
                // Same symbol reference - trivially equal
                if s_sym == t_sym {
                    return SubtypeResult::True;
                }

                // Try to resolve both refs and compare structurally
                let s_resolved = self.resolver.resolve_ref(*s_sym, self.interner);
                let t_resolved = self.resolver.resolve_ref(*t_sym, self.interner);

                match (s_resolved, t_resolved) {
                    (Some(s_type), Some(t_type)) => {
                        // Both resolved - compare structurally
                        self.check_subtype(s_type, t_type)
                    }
                    (Some(s_type), None) => {
                        // Only source resolved - compare source's structure to target ref
                        self.check_subtype(s_type, target)
                    }
                    (None, Some(t_type)) => {
                        // Only target resolved - compare source ref to target's structure
                        self.check_subtype(source, t_type)
                    }
                    (None, None) => {
                        // Neither resolved - fall back to identity
                        SubtypeResult::False
                    }
                }
            }

            // Source is Ref, target is structural - resolve and check
            (TypeKey::Ref(s_sym), _) => {
                if let Some(s_resolved) = self.resolver.resolve_ref(*s_sym, self.interner) {
                    self.check_subtype(s_resolved, target)
                } else {
                    // Can't resolve - assume not a subtype
                    SubtypeResult::False
                }
            }

            // Source is structural, target is Ref - resolve and check
            (_, TypeKey::Ref(t_sym)) => {
                if let Some(t_resolved) = self.resolver.resolve_ref(*t_sym, self.interner) {
                    self.check_subtype(source, t_resolved)
                } else {
                    // Can't resolve - assume not a subtype
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

            // Type query (typeof) - same symbol refs are equal
            (TypeKey::TypeQuery(s_sym), TypeKey::TypeQuery(t_sym)) => {
                if s_sym == t_sym {
                    SubtypeResult::True
                } else {
                    SubtypeResult::False
                }
            }

            // KeyOf types - keyof T <: keyof U if T :> U (contravariant)
            (TypeKey::KeyOf(s_inner), TypeKey::KeyOf(t_inner)) => {
                // keyof T <: keyof U when U <: T (contravariant in T)
                self.check_subtype(*t_inner, *s_inner)
            }
            // keyof T is a subtype of string | number | symbol
            (TypeKey::KeyOf(_), TypeKey::Intrinsic(IntrinsicKind::String)) => SubtypeResult::True,
            // Note: KeyOf vs Union is handled by the general Union target case above

            // Readonly types - readonly T[] <: readonly U[] if T <: U
            (TypeKey::ReadonlyType(s_inner), TypeKey::ReadonlyType(t_inner)) => {
                self.check_subtype(*s_inner, *t_inner)
            }

            // Unique symbol - only equal to itself
            (TypeKey::UniqueSymbol(s_sym), TypeKey::UniqueSymbol(t_sym)) => {
                if s_sym == t_sym {
                    SubtypeResult::True
                } else {
                    SubtypeResult::False
                }
            }
            // Unique symbol is a subtype of symbol
            (TypeKey::UniqueSymbol(_), TypeKey::Intrinsic(IntrinsicKind::Symbol)) => {
                SubtypeResult::True
            }

            // Infer types - identity only
            (TypeKey::Infer(s_info), TypeKey::Infer(t_info)) => {
                if s_info.name == t_info.name {
                    SubtypeResult::True
                } else {
                    SubtypeResult::False
                }
            }

            // This type - identity only
            (TypeKey::ThisType, TypeKey::ThisType) => SubtypeResult::True,

            // Template literal types - structural comparison
            (TypeKey::TemplateLiteral(s_spans), TypeKey::TemplateLiteral(t_spans)) => {
                if s_spans == t_spans {
                    SubtypeResult::True
                } else {
                    SubtypeResult::False
                }
            }
            // Template literal is a subtype of string
            (TypeKey::TemplateLiteral(_), TypeKey::Intrinsic(IntrinsicKind::String)) => {
                SubtypeResult::True
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
                // Target has rest element: remaining source elements must match
                // Unwrap the array type to get the element type
                let t_rest_elem_type = self.get_array_element_type(t_elem.type_id);

                for s_elem in source.iter().skip(i) {
                    if s_elem.rest {
                        // Source is also rest ...S[]
                        // S[] <: T[]
                        if !self.check_subtype(s_elem.type_id, t_elem.type_id).is_true() {
                            return SubtypeResult::False;
                        }
                    } else {
                        // Regular source element vs Target Rest Element Type
                        if !self.check_subtype(s_elem.type_id, t_rest_elem_type).is_true() {
                            return SubtypeResult::False;
                        }
                    }
                }
                // Target rest consumes everything
                return SubtypeResult::True;
            }

            // Target is not rest
            if let Some(s_elem) = source.get(i) {
                if s_elem.rest {
                    // Source has rest but target expects fixed element -> Mismatch
                    // e.g. Target: [number, number], Source: [number, ...number[]]
                    return SubtypeResult::False;
                }

                if !self.check_subtype(s_elem.type_id, t_elem.type_id).is_true() {
                    return SubtypeResult::False;
                }
            } else if !t_elem.optional {
                // Missing required element
                return SubtypeResult::False;
            }
        }

        // If we reached here, target has NO rest element (it is closed).
        // Ensure source has no extra elements.

        // 1. Source length check: Source cannot have more elements than Target
        if source.len() > target.len() {
            return SubtypeResult::False;
        }

        // 2. Source open check: Source cannot have a rest element if Target is closed
        for s_elem in source {
            if s_elem.rest {
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

    /// Check object with index signature subtyping
    fn check_object_with_index_subtype(&mut self, source: &ObjectShape, target: &ObjectShape) -> SubtypeResult {
        // First check named properties
        if !self.check_object_subtype(&source.properties, &target.properties).is_true() {
            return SubtypeResult::False;
        }

        // Check string index signature compatibility
        if let Some(ref t_string_idx) = target.string_index {
            match &source.string_index {
                Some(s_string_idx) => {
                    // Source string index must be subtype of target
                    if !self.check_subtype(s_string_idx.value_type, t_string_idx.value_type).is_true() {
                        return SubtypeResult::False;
                    }
                }
                None => {
                    // Target has string index, source doesn't
                    // All source properties must be compatible with target's string index
                    for prop in &source.properties {
                        if !self.check_subtype(prop.type_id, t_string_idx.value_type).is_true() {
                            return SubtypeResult::False;
                        }
                    }
                }
            }
        }

        // Check number index signature compatibility
        if let Some(ref t_number_idx) = target.number_index {
            match &source.number_index {
                Some(s_number_idx) => {
                    // Source number index must be subtype of target
                    if !self.check_subtype(s_number_idx.value_type, t_number_idx.value_type).is_true() {
                        return SubtypeResult::False;
                    }
                }
                None => {
                    // Target has number index but source doesn't - this is OK
                    // (number indexing is optional)
                }
            }
        }

        // If source has string index, all number-indexed properties must be compatible
        // (since number converts to string for property access)
        if let (Some(s_string_idx), Some(s_number_idx)) = (&source.string_index, &source.number_index) {
            if !self.check_subtype(s_number_idx.value_type, s_string_idx.value_type).is_true() {
                // This is a constraint violation in the source itself
                return SubtypeResult::False;
            }
        }

        SubtypeResult::True
    }

    /// Check simple object to object with index signature
    fn check_object_to_indexed(&mut self, source: &[PropertyInfo], target: &ObjectShape) -> SubtypeResult {
        // First check named properties match
        if !self.check_object_subtype(source, &target.properties).is_true() {
            return SubtypeResult::False;
        }

        // All source properties must satisfy target's string index signature
        if let Some(ref string_idx) = target.string_index {
            for prop in source {
                if !self.check_subtype(prop.type_id, string_idx.value_type).is_true() {
                    return SubtypeResult::False;
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

        // Check if target has a rest parameter
        let target_has_rest = target.params.last().map_or(false, |p| p.rest);
        let source_has_rest = source.params.last().map_or(false, |p| p.rest);

        // Count non-rest parameters
        let target_fixed_count = if target_has_rest { target.params.len().saturating_sub(1) } else { target.params.len() };
        let source_fixed_count = if source_has_rest { source.params.len().saturating_sub(1) } else { source.params.len() };

        // If target doesn't have a rest parameter, source can't have more params than target
        if !target_has_rest && source.params.len() > target.params.len() {
            return SubtypeResult::False;
        }

        // Compare fixed parameters
        let fixed_compare_count = std::cmp::min(source_fixed_count, target_fixed_count);
        for i in 0..fixed_compare_count {
            let s_param = &source.params[i];
            let t_param = &target.params[i];
            // Bivariant: either direction works
            if !self.check_subtype(s_param.type_id, t_param.type_id).is_true()
                && !self.check_subtype(t_param.type_id, s_param.type_id).is_true()
            {
                return SubtypeResult::False;
            }
        }

        // If target has rest parameter, check source's extra params against the rest type
        if target_has_rest {
            let rest_param = target.params.last().unwrap();
            // Get the element type of the rest array
            let rest_elem_type = self.get_array_element_type(rest_param.type_id);

            // Check source params that exceed target's fixed count against rest type
            for i in target_fixed_count..source_fixed_count {
                let s_param = &source.params[i];
                // Bivariant check against rest element type
                if !self.check_subtype(s_param.type_id, rest_elem_type).is_true()
                    && !self.check_subtype(rest_elem_type, s_param.type_id).is_true()
                {
                    return SubtypeResult::False;
                }
            }

            // If source also has a rest param, check it against target's rest
            if source_has_rest {
                let s_rest_param = source.params.last().unwrap();
                let s_rest_elem = self.get_array_element_type(s_rest_param.type_id);
                if !self.check_subtype(s_rest_elem, rest_elem_type).is_true()
                    && !self.check_subtype(rest_elem_type, s_rest_elem).is_true()
                {
                    return SubtypeResult::False;
                }
            }
        }

        SubtypeResult::True
    }

    /// Get the element type of an array type, or return the type itself for any[]
    fn get_array_element_type(&self, type_id: TypeId) -> TypeId {
        if type_id == TypeId::ANY {
            return TypeId::ANY;
        }
        match self.interner.lookup(type_id) {
            Some(TypeKey::Array(elem)) => elem,
            // For any[], the type itself is assignable from anything
            _ => type_id,
        }
    }

    /// Evaluate a meta-type (conditional, index access, mapped, etc.) to its concrete form.
    /// Uses TypeEvaluator to reduce types like `T extends U ? X : Y` to either X or Y.
    fn evaluate_type(&self, type_id: TypeId) -> TypeId {
        use crate::solver::evaluate::TypeEvaluator;
        let evaluator = TypeEvaluator::with_resolver(self.interner, self.resolver);
        evaluator.evaluate(type_id)
    }

    /// Check callable subtyping (overloaded signatures)
    fn check_callable_subtype(&mut self, source: &CallableShape, target: &CallableShape) -> SubtypeResult {
        // For each target call signature, at least one source signature must match
        for t_sig in &target.call_signatures {
            let mut found_match = false;
            for s_sig in &source.call_signatures {
                if self.check_call_signature_subtype(s_sig, t_sig).is_true() {
                    found_match = true;
                    break;
                }
            }
            if !found_match {
                return SubtypeResult::False;
            }
        }

        // For each target construct signature, at least one source signature must match
        for t_sig in &target.construct_signatures {
            let mut found_match = false;
            for s_sig in &source.construct_signatures {
                if self.check_call_signature_subtype(s_sig, t_sig).is_true() {
                    found_match = true;
                    break;
                }
            }
            if !found_match {
                return SubtypeResult::False;
            }
        }

        // Check properties (if any)
        if !self.check_object_subtype(&source.properties, &target.properties).is_true() {
            return SubtypeResult::False;
        }

        SubtypeResult::True
    }

    /// Check call signature subtyping
    fn check_call_signature_subtype(&mut self, source: &CallSignature, target: &CallSignature) -> SubtypeResult {
        // Return type is covariant
        if !self.check_subtype(source.return_type, target.return_type).is_true() {
            return SubtypeResult::False;
        }

        // Check if target has a rest parameter
        let target_has_rest = target.params.last().map_or(false, |p| p.rest);
        let source_has_rest = source.params.last().map_or(false, |p| p.rest);

        // Count non-rest parameters
        let target_fixed_count = if target_has_rest { target.params.len().saturating_sub(1) } else { target.params.len() };
        let source_fixed_count = if source_has_rest { source.params.len().saturating_sub(1) } else { source.params.len() };

        // If target doesn't have a rest parameter, source can't have more params than target
        if !target_has_rest && source.params.len() > target.params.len() {
            return SubtypeResult::False;
        }

        // Compare fixed parameters
        let fixed_compare_count = std::cmp::min(source_fixed_count, target_fixed_count);
        for i in 0..fixed_compare_count {
            let s_param = &source.params[i];
            let t_param = &target.params[i];
            // Bivariant: either direction works
            if !self.check_subtype(s_param.type_id, t_param.type_id).is_true()
                && !self.check_subtype(t_param.type_id, s_param.type_id).is_true()
            {
                return SubtypeResult::False;
            }
        }

        // If target has rest parameter, check source's extra params against the rest type
        if target_has_rest {
            let rest_param = target.params.last().unwrap();
            let rest_elem_type = self.get_array_element_type(rest_param.type_id);

            for i in target_fixed_count..source_fixed_count {
                let s_param = &source.params[i];
                if !self.check_subtype(s_param.type_id, rest_elem_type).is_true()
                    && !self.check_subtype(rest_elem_type, s_param.type_id).is_true()
                {
                    return SubtypeResult::False;
                }
            }

            if source_has_rest {
                let s_rest_param = source.params.last().unwrap();
                let s_rest_elem = self.get_array_element_type(s_rest_param.type_id);
                if !self.check_subtype(s_rest_elem, rest_elem_type).is_true()
                    && !self.check_subtype(rest_elem_type, s_rest_elem).is_true()
                {
                    return SubtypeResult::False;
                }
            }
        }

        SubtypeResult::True
    }

    /// Check call signature subtype to function shape
    fn check_call_signature_subtype_to_fn(&mut self, source: &CallSignature, target: &FunctionShape) -> SubtypeResult {
        // Return type is covariant
        if !self.check_subtype(source.return_type, target.return_type).is_true() {
            return SubtypeResult::False;
        }

        // Check if target has a rest parameter
        let target_has_rest = target.params.last().map_or(false, |p| p.rest);
        let source_has_rest = source.params.last().map_or(false, |p| p.rest);

        // Count non-rest parameters
        let target_fixed_count = if target_has_rest { target.params.len().saturating_sub(1) } else { target.params.len() };
        let source_fixed_count = if source_has_rest { source.params.len().saturating_sub(1) } else { source.params.len() };

        // If target doesn't have a rest parameter, source can't have more params than target
        if !target_has_rest && source.params.len() > target.params.len() {
            return SubtypeResult::False;
        }

        // Compare fixed parameters
        let fixed_compare_count = std::cmp::min(source_fixed_count, target_fixed_count);
        for i in 0..fixed_compare_count {
            let s_param = &source.params[i];
            let t_param = &target.params[i];
            // Bivariant
            if !self.check_subtype(s_param.type_id, t_param.type_id).is_true()
                && !self.check_subtype(t_param.type_id, s_param.type_id).is_true()
            {
                return SubtypeResult::False;
            }
        }

        // If target has rest parameter, check source's extra params against the rest type
        if target_has_rest {
            let rest_param = target.params.last().unwrap();
            let rest_elem_type = self.get_array_element_type(rest_param.type_id);

            for i in target_fixed_count..source_fixed_count {
                let s_param = &source.params[i];
                if !self.check_subtype(s_param.type_id, rest_elem_type).is_true()
                    && !self.check_subtype(rest_elem_type, s_param.type_id).is_true()
                {
                    return SubtypeResult::False;
                }
            }

            if source_has_rest {
                let s_rest_param = source.params.last().unwrap();
                let s_rest_elem = self.get_array_element_type(s_rest_param.type_id);
                if !self.check_subtype(s_rest_elem, rest_elem_type).is_true()
                    && !self.check_subtype(rest_elem_type, s_rest_elem).is_true()
                {
                    return SubtypeResult::False;
                }
            }
        }

        SubtypeResult::True
    }

    /// Check function shape subtype to call signature
    fn check_call_signature_subtype_fn(&mut self, source: &FunctionShape, target: &CallSignature) -> SubtypeResult {
        // Return type is covariant
        if !self.check_subtype(source.return_type, target.return_type).is_true() {
            return SubtypeResult::False;
        }

        // Check if target has a rest parameter
        let target_has_rest = target.params.last().map_or(false, |p| p.rest);
        let source_has_rest = source.params.last().map_or(false, |p| p.rest);

        // Count non-rest parameters
        let target_fixed_count = if target_has_rest { target.params.len().saturating_sub(1) } else { target.params.len() };
        let source_fixed_count = if source_has_rest { source.params.len().saturating_sub(1) } else { source.params.len() };

        // If target doesn't have a rest parameter, source can't have more params than target
        if !target_has_rest && source.params.len() > target.params.len() {
            return SubtypeResult::False;
        }

        // Compare fixed parameters
        let fixed_compare_count = std::cmp::min(source_fixed_count, target_fixed_count);
        for i in 0..fixed_compare_count {
            let s_param = &source.params[i];
            let t_param = &target.params[i];
            // Bivariant
            if !self.check_subtype(s_param.type_id, t_param.type_id).is_true()
                && !self.check_subtype(t_param.type_id, s_param.type_id).is_true()
            {
                return SubtypeResult::False;
            }
        }

        // If target has rest parameter, check source's extra params against the rest type
        if target_has_rest {
            let rest_param = target.params.last().unwrap();
            let rest_elem_type = self.get_array_element_type(rest_param.type_id);

            for i in target_fixed_count..source_fixed_count {
                let s_param = &source.params[i];
                if !self.check_subtype(s_param.type_id, rest_elem_type).is_true()
                    && !self.check_subtype(rest_elem_type, s_param.type_id).is_true()
                {
                    return SubtypeResult::False;
                }
            }

            if source_has_rest {
                let s_rest_param = source.params.last().unwrap();
                let s_rest_elem = self.get_array_element_type(s_rest_param.type_id);
                if !self.check_subtype(s_rest_elem, rest_elem_type).is_true()
                    && !self.check_subtype(rest_elem_type, s_rest_elem).is_true()
                {
                    return SubtypeResult::False;
                }
            }
        }

        SubtypeResult::True
    }
}

// =============================================================================
// Error Explanation API
// =============================================================================

/// Reason why a subtype check failed.
/// Used by `explain_failure` to provide detailed error messages.
#[derive(Clone, Debug)]
pub enum SubtypeFailureReason {
    /// A required property is missing in the source type.
    MissingProperty {
        property_name: std::sync::Arc<str>,
        source_type: TypeId,
        target_type: TypeId,
    },
    /// Property types are incompatible.
    PropertyTypeMismatch {
        property_name: std::sync::Arc<str>,
        source_property_type: TypeId,
        target_property_type: TypeId,
        nested_reason: Option<Box<SubtypeFailureReason>>,
    },
    /// Optional property cannot satisfy required property.
    OptionalPropertyRequired {
        property_name: std::sync::Arc<str>,
    },
    /// Return types are incompatible.
    ReturnTypeMismatch {
        source_return: TypeId,
        target_return: TypeId,
        nested_reason: Option<Box<SubtypeFailureReason>>,
    },
    /// Parameter types are incompatible.
    ParameterTypeMismatch {
        param_index: usize,
        source_param: TypeId,
        target_param: TypeId,
    },
    /// Too many parameters in source.
    TooManyParameters {
        source_count: usize,
        target_count: usize,
    },
    /// Tuple element count mismatch.
    TupleElementMismatch {
        source_count: usize,
        target_count: usize,
    },
    /// Tuple element type mismatch.
    TupleElementTypeMismatch {
        index: usize,
        source_element: TypeId,
        target_element: TypeId,
    },
    /// Array element type mismatch.
    ArrayElementMismatch {
        source_element: TypeId,
        target_element: TypeId,
    },
    /// Index signature value type mismatch.
    IndexSignatureMismatch {
        index_kind: &'static str, // "string" or "number"
        source_value_type: TypeId,
        target_value_type: TypeId,
    },
    /// No union member matches.
    NoUnionMemberMatches {
        source_type: TypeId,
        target_union_members: Vec<TypeId>,
    },
    /// Generic type mismatch (no more specific reason).
    TypeMismatch {
        source_type: TypeId,
        target_type: TypeId,
    },
}

impl<'a, R: TypeResolver> SubtypeChecker<'a, R> {
    /// Explain why `source` is not assignable to `target`.
    ///
    /// This is the "slow path" - called only when `is_assignable_to` returns false
    /// and we need to generate an error message. Re-runs the subtype logic with
    /// tracing enabled to produce a structured failure reason.
    ///
    /// Returns `None` if the types are actually compatible (shouldn't happen
    /// if called correctly after a failed check).
    pub fn explain_failure(&mut self, source: TypeId, target: TypeId) -> Option<SubtypeFailureReason> {
        // Fast path: if types are equal, no failure
        if source == target {
            return None;
        }

        // Check for any/unknown/never special cases
        if source == TypeId::ANY || target == TypeId::ANY || target == TypeId::UNKNOWN {
            return None;
        }
        if source == TypeId::NEVER {
            return None;
        }
        if source == TypeId::ERROR || target == TypeId::ERROR {
            return None;
        }

        // Look up the type keys
        let source_key = self.interner.lookup(source)?;
        let target_key = self.interner.lookup(target)?;

        self.explain_failure_inner(source, target, &source_key, &target_key)
    }

    fn explain_failure_inner(
        &mut self,
        source: TypeId,
        target: TypeId,
        source_key: &TypeKey,
        target_key: &TypeKey,
    ) -> Option<SubtypeFailureReason> {
        match (source_key, target_key) {
            // Object to object - find the specific missing/mismatched property
            (TypeKey::Object(s_props), TypeKey::Object(t_props)) => {
                self.explain_object_failure(source, target, s_props, t_props)
            }

            // Object with index to object with index
            (TypeKey::ObjectWithIndex(s_shape), TypeKey::ObjectWithIndex(t_shape)) => {
                self.explain_indexed_object_failure(source, target, s_shape, t_shape)
            }

            // Simple object to indexed object
            (TypeKey::Object(s_props), TypeKey::ObjectWithIndex(t_shape)) => {
                // First check properties
                if let Some(reason) = self.explain_object_failure(source, target, s_props, &t_shape.properties) {
                    return Some(reason);
                }
                // Then check index signature constraints
                if let Some(ref string_idx) = t_shape.string_index {
                    for prop in s_props {
                        if !self.check_subtype(prop.type_id, string_idx.value_type).is_true() {
                            return Some(SubtypeFailureReason::IndexSignatureMismatch {
                                index_kind: "string",
                                source_value_type: prop.type_id,
                                target_value_type: string_idx.value_type,
                            });
                        }
                    }
                }
                None
            }

            // Function to function
            (TypeKey::Function(s_fn), TypeKey::Function(t_fn)) => {
                self.explain_function_failure(s_fn, t_fn)
            }

            // Array to array
            (TypeKey::Array(s_elem), TypeKey::Array(t_elem)) => {
                if !self.check_subtype(*s_elem, *t_elem).is_true() {
                    Some(SubtypeFailureReason::ArrayElementMismatch {
                        source_element: *s_elem,
                        target_element: *t_elem,
                    })
                } else {
                    None
                }
            }

            // Tuple to tuple
            (TypeKey::Tuple(s_elems), TypeKey::Tuple(t_elems)) => {
                self.explain_tuple_failure(s_elems, t_elems)
            }

            // Union target - source must match at least one member
            (_, TypeKey::Union(members)) => {
                // If none match, explain which member was closest
                // For now, just report that no member matches
                Some(SubtypeFailureReason::NoUnionMemberMatches {
                    source_type: source,
                    target_union_members: members.clone(),
                })
            }

            // Default: generic type mismatch
            _ => Some(SubtypeFailureReason::TypeMismatch {
                source_type: source,
                target_type: target,
            }),
        }
    }

    /// Explain why an object type assignment failed.
    fn explain_object_failure(
        &mut self,
        source: TypeId,
        target: TypeId,
        source_props: &[PropertyInfo],
        target_props: &[PropertyInfo],
    ) -> Option<SubtypeFailureReason> {
        for t_prop in target_props {
            let s_prop = source_props.iter().find(|p| p.name == t_prop.name);

            match s_prop {
                Some(sp) => {
                    // Check optional/required mismatch
                    if sp.optional && !t_prop.optional {
                        return Some(SubtypeFailureReason::OptionalPropertyRequired {
                            property_name: t_prop.name.clone(),
                        });
                    }

                    // Check property type compatibility
                    if !self.check_subtype(sp.type_id, t_prop.type_id).is_true() {
                        // Recursively explain the nested failure
                        let nested = self.explain_failure(sp.type_id, t_prop.type_id);
                        return Some(SubtypeFailureReason::PropertyTypeMismatch {
                            property_name: t_prop.name.clone(),
                            source_property_type: sp.type_id,
                            target_property_type: t_prop.type_id,
                            nested_reason: nested.map(Box::new),
                        });
                    }
                }
                None => {
                    // Required property is missing
                    if !t_prop.optional {
                        return Some(SubtypeFailureReason::MissingProperty {
                            property_name: t_prop.name.clone(),
                            source_type: source,
                            target_type: target,
                        });
                    }
                }
            }
        }

        None
    }

    /// Explain why an indexed object type assignment failed.
    fn explain_indexed_object_failure(
        &mut self,
        source: TypeId,
        target: TypeId,
        source_shape: &ObjectShape,
        target_shape: &ObjectShape,
    ) -> Option<SubtypeFailureReason> {
        // First check properties
        if let Some(reason) = self.explain_object_failure(
            source,
            target,
            &source_shape.properties,
            &target_shape.properties,
        ) {
            return Some(reason);
        }

        // Check string index signature
        if let Some(ref t_string_idx) = target_shape.string_index {
            if let Some(ref s_string_idx) = source_shape.string_index {
                if !self.check_subtype(s_string_idx.value_type, t_string_idx.value_type).is_true() {
                    return Some(SubtypeFailureReason::IndexSignatureMismatch {
                        index_kind: "string",
                        source_value_type: s_string_idx.value_type,
                        target_value_type: t_string_idx.value_type,
                    });
                }
            }
        }

        // Check number index signature
        if let Some(ref t_number_idx) = target_shape.number_index {
            if let Some(ref s_number_idx) = source_shape.number_index {
                if !self.check_subtype(s_number_idx.value_type, t_number_idx.value_type).is_true() {
                    return Some(SubtypeFailureReason::IndexSignatureMismatch {
                        index_kind: "number",
                        source_value_type: s_number_idx.value_type,
                        target_value_type: t_number_idx.value_type,
                    });
                }
            }
        }

        None
    }

    /// Explain why a function type assignment failed.
    fn explain_function_failure(
        &mut self,
        source: &FunctionShape,
        target: &FunctionShape,
    ) -> Option<SubtypeFailureReason> {
        // Check return type
        if !self.check_subtype(source.return_type, target.return_type).is_true() {
            let nested = self.explain_failure(source.return_type, target.return_type);
            return Some(SubtypeFailureReason::ReturnTypeMismatch {
                source_return: source.return_type,
                target_return: target.return_type,
                nested_reason: nested.map(Box::new),
            });
        }

        // Check parameter count
        if source.params.len() > target.params.len() {
            return Some(SubtypeFailureReason::TooManyParameters {
                source_count: source.params.len(),
                target_count: target.params.len(),
            });
        }

        // Check parameter types
        for (i, s_param) in source.params.iter().enumerate() {
            if let Some(t_param) = target.params.get(i) {
                // Bivariant check
                if !self.check_subtype(s_param.type_id, t_param.type_id).is_true()
                    && !self.check_subtype(t_param.type_id, s_param.type_id).is_true()
                {
                    return Some(SubtypeFailureReason::ParameterTypeMismatch {
                        param_index: i,
                        source_param: s_param.type_id,
                        target_param: t_param.type_id,
                    });
                }
            }
        }

        None
    }

    /// Explain why a tuple type assignment failed.
    fn explain_tuple_failure(
        &mut self,
        source: &[TupleElement],
        target: &[TupleElement],
    ) -> Option<SubtypeFailureReason> {
        let source_required = source.iter().filter(|e| !e.optional && !e.rest).count();
        let target_required = target.iter().filter(|e| !e.optional && !e.rest).count();

        if source_required < target_required {
            return Some(SubtypeFailureReason::TupleElementMismatch {
                source_count: source.len(),
                target_count: target.len(),
            });
        }

        for (i, t_elem) in target.iter().enumerate() {
            if t_elem.rest {
                let t_rest_elem_type = self.get_array_element_type(t_elem.type_id);
                // Check rest elements
                for (j, s_elem) in source.iter().enumerate().skip(i) {
                    let target_type = if s_elem.rest { t_elem.type_id } else { t_rest_elem_type };
                    if !self.check_subtype(s_elem.type_id, target_type).is_true() {
                        return Some(SubtypeFailureReason::TupleElementTypeMismatch {
                            index: j,
                            source_element: s_elem.type_id,
                            target_element: target_type,
                        });
                    }
                }
                // Target rest consumes everything, so no length error possible here
                return None;
            }

            if let Some(s_elem) = source.get(i) {
                if s_elem.rest {
                    // Source has rest but target expects fixed element
                    return Some(SubtypeFailureReason::TupleElementMismatch {
                        source_count: source.len(), // Approximate "infinity"
                        target_count: target.len(),
                    });
                }

                if !self.check_subtype(s_elem.type_id, t_elem.type_id).is_true() {
                    return Some(SubtypeFailureReason::TupleElementTypeMismatch {
                        index: i,
                        source_element: s_elem.type_id,
                        target_element: t_elem.type_id,
                    });
                }
            } else if !t_elem.optional {
                return Some(SubtypeFailureReason::TupleElementMismatch {
                    source_count: source.len(),
                    target_count: target.len(),
                });
            }
        }

        // Target is closed. Check for extra elements in source.
        if source.len() > target.len() {
            return Some(SubtypeFailureReason::TupleElementMismatch {
                source_count: source.len(),
                target_count: target.len(),
            });
        }

        for s_elem in source {
            if s_elem.rest {
                return Some(SubtypeFailureReason::TupleElementMismatch {
                    source_count: source.len(), // implies open
                    target_count: target.len(),
                });
            }
        }

        None
    }
}

/// Convenience function for one-off subtype checks (without resolver)
pub fn is_subtype_of(interner: &TypeInterner, source: TypeId, target: TypeId) -> bool {
    let mut checker = SubtypeChecker::new(interner);
    checker.is_subtype_of(source, target)
}

/// Convenience function for one-off subtype checks with a resolver
pub fn is_subtype_of_with_resolver<R: TypeResolver>(
    interner: &TypeInterner,
    resolver: &R,
    source: TypeId,
    target: TypeId,
) -> bool {
    let mut checker = SubtypeChecker::with_resolver(interner, resolver);
    checker.is_subtype_of(source, target)
}

#[cfg(test)]
#[path = "subtype_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "index_signature_tests.rs"]
mod index_signature_tests;

#[cfg(test)]
#[path = "callable_tests.rs"]
mod callable_tests;
