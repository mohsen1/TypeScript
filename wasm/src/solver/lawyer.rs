//! The "Lawyer" layer for TypeScript compatibility.
//!
//! This module implements the compatibility layer that sits between the public API
//! and the core structural subtype checking ("Judge" layer). It applies TypeScript-
//! specific business logic, including nuanced rules for `any` propagation.
//!
//! ## Judge vs. Lawyer Architecture (SOLVER.md Section 8)
//!
//! - **Judge (SubtypeChecker):** Implements strict, sound set theory semantics.
//!   It knows nothing about TypeScript legacy behavior.
//! - **Lawyer (AnyPropagationRules + CompatChecker):** Applies TypeScript-specific
//!   rules and delegates to the Judge with appropriate configuration.
//!
//! ## TypeScript Quirks Handled
//!
//! ### A. `any` Propagation (The Black Hole)
//! `any` violates the partial order of sets - it's both a subtype and supertype
//! of everything. The `AnyPropagationRules` struct handles this short-circuit.
//!
//! ### B. Function Variance
//! - **Strict mode (strictFunctionTypes):** Parameters are contravariant (sound)
//! - **Legacy mode:** Parameters are bivariant (unsound but backward-compatible)
//! - **Methods:** Always bivariant regardless of strictFunctionTypes
//!
//! ### C. Freshness (Excess Property Checking)
//! Object literals are "fresh" and trigger excess property checking.
//! Once assigned to a variable, they lose freshness and allow width subtyping.
//! The `FreshnessTracker` provides this functionality.
//!
//! ### D. The Void Exception
//! TypeScript allows `() => void` to match `() => T` for any T, because
//! the caller promises to ignore the return value.
//!
//! ### E. Weak Type Detection (TS2559)
//! Types with only optional properties require at least one common property
//! with the source type to prevent accidental assignment mistakes.
//!
//! The key principle is that `any` should NOT silence structural mismatches.
//! While `any` is TypeScript's escape hatch, we still want to catch real errors
//! even when `any` is involved.

use crate::solver::TypeDatabase;
use crate::solver::types::{TypeId, TypeKey};

/// Rules for `any` propagation in type checking.
///
/// In TypeScript, `any` is both a top type (everything is assignable to `any`)
/// and a bottom type (`any` is assignable to everything). However, `any` should
/// not be used to silence real structural mismatches.
///
/// This struct encapsulates the nuanced rules for when `any` is allowed to
/// suppress type errors and when it isn't.
pub struct AnyPropagationRules {
    /// Whether to allow `any` to silence structural mismatches.
    /// When false, `any` is treated more strictly and structural errors
    /// are still reported even when `any` is involved.
    pub(crate) allow_any_suppression: bool,
}

impl AnyPropagationRules {
    /// Create a new `AnyPropagationRules` with default settings.
    ///
    /// By default, `any` suppression is enabled for backward compatibility
    /// with existing TypeScript behavior.
    pub fn new() -> Self {
        AnyPropagationRules {
            allow_any_suppression: true,
        }
    }

    /// Create strict `AnyPropagationRules` where `any` does not silence
    /// structural mismatches.
    ///
    /// In strict mode, even when `any` is involved, the type checker will
    /// perform structural checking and report mismatches.
    pub fn strict() -> Self {
        AnyPropagationRules {
            allow_any_suppression: false,
        }
    }

    /// Set whether `any` is allowed to suppress structural mismatches.
    pub fn set_allow_any_suppression(&mut self, allow: bool) {
        self.allow_any_suppression = allow;
    }

    /// Check if `any` is allowed to suppress a type mismatch between
    /// `source` and `target`.
    ///
    /// This function implements the nuanced rules for when `any` should
    /// and should not silence errors.
    ///
    /// ## Rule: Any should NOT silence structural mismatches
    ///
    /// The key insight is that `any` is TypeScript's escape hatch, but we
    /// still want to catch real errors. This function determines if a
    /// specific case should allow `any` to suppress the error.
    ///
    /// ### Cases where `any` CAN suppress:
    /// - Direct assignment: `let x: any = someValue`
    /// - Direct assignment from `any`: `let x: SomeType = anyValue`
    /// - When explicitly opted-in via compiler flags
    ///
    /// ### Cases where `any` CANNOT suppress:
    /// - Property access mismatches on objects with `any` properties
    /// - Function call arguments when parameter is `any` but structural
    ///   mismatch exists in other arguments
    /// - Array/tuple element mismatches when container has `any` elements
    pub fn is_any_allowed_to_suppress(
        &self,
        source: TypeId,
        target: TypeId,
        interner: &dyn TypeDatabase,
    ) -> bool {
        // If suppression is globally disabled, `any` never suppresses
        if !self.allow_any_suppression {
            return false;
        }

        // Fast path: neither type is `any`
        let source_is_any = source == TypeId::ANY;
        let target_is_any = target == TypeId::ANY;
        if !source_is_any && !target_is_any {
            return false;
        }

        // At this point, at least one of the types is `any`
        // Now we need to check if this is a case where we should
        // allow suppression or if there's a structural mismatch we
        // want to catch anyway

        // Case 1: Direct assignment to/from `any` - allow suppression
        // This is the standard TypeScript behavior and is expected
        if source_is_any || target_is_any {
            // Check if there's a non-trivial structure that might indicate
            // a real error we want to catch despite the `any`
            if self.has_structural_mismatch_despite_any(source, target, interner) {
                // There's a structural mismatch we should report
                return false;
            }
            // Direct `any` involvement is OK
            return true;
        }

        false
    }

    /// Check if there's a structural mismatch that should be reported
    /// even though `any` is involved.
    ///
    /// This implements the core logic of "Any should NOT silence
    /// structural mismatches."
    ///
    /// In default mode (legacy TypeScript behavior), `any` is allowed
    /// to suppress most errors. In strict mode, `any` does not suppress
    /// structural errors for complex types.
    fn has_structural_mismatch_despite_any(
        &self,
        source: TypeId,
        target: TypeId,
        interner: &dyn TypeDatabase,
    ) -> bool {
        // If both are `any`, there's no mismatch to report
        if source == TypeId::ANY && target == TypeId::ANY {
            return false;
        }

        // In non-strict mode, we allow `any` to suppress errors
        // (legacy TypeScript behavior)
        if self.allow_any_suppression {
            return false;
        }

        // In strict mode, check if the non-`any` type has interesting structure
        // that we should validate
        let non_any_type = if source == TypeId::ANY {
            target
        } else {
            source
        };

        // Look at the structure of the non-`any` type
        match interner.lookup(non_any_type) {
            Some(TypeKey::Object(shape_id)) => {
                let shape = interner.object_shape(shape_id);
                // Objects with properties should be validated even with `any`
                !shape.properties.is_empty()
            }
            Some(TypeKey::ObjectWithIndex(shape_id)) => {
                let shape = interner.object_shape(shape_id);
                // Objects with index signatures might still have structure to check
                !shape.properties.is_empty()
                    || shape.string_index.is_some()
                    || shape.number_index.is_some()
            }
            Some(TypeKey::Array(_)) => {
                // Arrays have structure (element type) that matters
                true
            }
            Some(TypeKey::Tuple(_)) => {
                // Tuples have significant structure
                true
            }
            Some(TypeKey::Function(_)) | Some(TypeKey::Callable(_)) => {
                // Functions have signatures that matter
                true
            }
            _ => false,
        }
    }

    /// Get the subtype check result considering `any` propagation rules.
    ///
    /// This is the main entry point that decides whether to:
    /// 1. Allow `any` to suppress the check (return true)
    /// 2. Delegate to the Judge for structural checking
    ///
    /// Returns `Some(result)` if the `any` rules decide the outcome,
    /// or `None` if the check should be delegated to the structural checker.
    pub fn check_any_propagation(
        &self,
        source: TypeId,
        target: TypeId,
        interner: &dyn TypeDatabase,
    ) -> Option<bool> {
        // Check if either type is `any`
        let source_is_any = source == TypeId::ANY;
        let target_is_any = target == TypeId::ANY;

        if !source_is_any && !target_is_any {
            // No `any` involved - delegate to structural checker
            return None;
        }

        // `any` is involved - check if suppression is allowed
        if self.is_any_allowed_to_suppress(source, target, interner) {
            Some(true)
        } else {
            // `any` is present but shouldn't suppress - delegate to structural checker
            None
        }
    }
}

impl Default for AnyPropagationRules {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Function Parameter Bivariance Handling
// =============================================================================

/// Configuration for function parameter bivariance rules.
///
/// TypeScript has a complicated relationship with function parameter variance:
///
/// ## Sound Behavior (Contravariance)
/// In a sound type system, function parameters should be contravariant:
/// ```typescript
/// // (x: Animal) => void should NOT be assignable to (x: Dog) => void
/// // because the latter might be called with any Dog, but the former
/// // only handles general Animals
/// type Handler = (x: Dog) => void;
/// const animalHandler: (x: Animal) => void = (a) => { /* ... */ };
/// const handler: Handler = animalHandler; // UNSOUND if allowed!
/// ```
///
/// ## TypeScript's Legacy Behavior (Bivariance)
/// For backward compatibility, TypeScript defaults to bivariant function
/// parameters (both covariant AND contravariant), which is unsound but
/// allows more programs to compile.
///
/// ## strictFunctionTypes Flag
/// When `strictFunctionTypes` is enabled, TypeScript uses contravariance
/// for function type parameters, but NOT for method parameters.
///
/// ## Method Exception
/// Methods are ALWAYS bivariant regardless of `strictFunctionTypes`:
/// ```typescript
/// interface Animal {
///   speak(): void;
/// }
/// interface Dog extends Animal {
///   speak(): void;  // Method - always bivariant
/// }
/// ```
///
/// This is because methods are often used polymorphically in class hierarchies,
/// and strict contravariance would break too many existing patterns.
#[derive(Debug, Clone)]
pub struct FunctionBivarianceConfig {
    /// When true, use contravariance for function parameters (sound).
    /// When false, use bivariance (legacy TypeScript behavior).
    pub strict_function_types: bool,

    /// When true, methods use bivariance even when strict_function_types is enabled.
    /// This matches TypeScript's behavior where methods are always bivariant.
    /// Default: true (matches TypeScript semantics).
    pub methods_are_bivariant: bool,

    /// When true, callback parameters in method signatures also use bivariance.
    /// This handles cases like:
    /// ```typescript
    /// interface Array<T> {
    ///   forEach(callback: (item: T) => void): void;
    /// }
    /// ```
    /// The `callback` parameter is bivariant because `forEach` is a method.
    /// Default: true (matches TypeScript semantics).
    pub method_callbacks_are_bivariant: bool,
}

impl FunctionBivarianceConfig {
    /// Create a new config with TypeScript's default settings.
    /// - strict_function_types: false (legacy bivariance)
    /// - methods_are_bivariant: true
    /// - method_callbacks_are_bivariant: true
    pub fn new() -> Self {
        FunctionBivarianceConfig {
            strict_function_types: false,
            methods_are_bivariant: true,
            method_callbacks_are_bivariant: true,
        }
    }

    /// Create a config that matches TypeScript with `strictFunctionTypes: true`.
    /// - strict_function_types: true (contravariance for functions)
    /// - methods_are_bivariant: true (methods still bivariant)
    /// - method_callbacks_are_bivariant: true
    pub fn strict() -> Self {
        FunctionBivarianceConfig {
            strict_function_types: true,
            methods_are_bivariant: true,
            method_callbacks_are_bivariant: true,
        }
    }

    /// Create a fully sound config (not TypeScript compatible).
    /// - strict_function_types: true
    /// - methods_are_bivariant: false (methods are contravariant too)
    /// - method_callbacks_are_bivariant: false
    ///
    /// Warning: This mode will reject many valid TypeScript programs.
    pub fn fully_sound() -> Self {
        FunctionBivarianceConfig {
            strict_function_types: true,
            methods_are_bivariant: false,
            method_callbacks_are_bivariant: false,
        }
    }

    /// Determine if bivariance should be used for comparing parameter types.
    ///
    /// # Arguments
    /// * `is_method` - Whether the function being compared is a method
    /// * `is_callback_in_method` - Whether this is a callback parameter in a method signature
    ///
    /// # Returns
    /// `true` if bivariance should be used, `false` if contravariance should be used.
    pub fn should_use_bivariance(&self, is_method: bool, is_callback_in_method: bool) -> bool {
        // Legacy mode: always use bivariance
        if !self.strict_function_types {
            return true;
        }

        // In strict mode, check method exceptions
        if is_method && self.methods_are_bivariant {
            return true;
        }

        // Check callback-in-method exception
        if is_callback_in_method && self.method_callbacks_are_bivariant {
            return true;
        }

        // Strict mode for regular functions: use contravariance
        false
    }

    /// Check if a source parameter type is compatible with a target parameter type.
    ///
    /// This implements the variance check based on current configuration:
    /// - Bivariant: source <: target OR target <: source
    /// - Contravariant: target <: source
    ///
    /// # Arguments
    /// * `is_method` - Whether this is a method comparison
    /// * `source_subtype_of_target` - Result of checking source <: target
    /// * `target_subtype_of_source` - Result of checking target <: source
    ///
    /// # Returns
    /// `true` if the parameter types are compatible.
    pub fn are_parameters_compatible(
        &self,
        is_method: bool,
        source_subtype_of_target: bool,
        target_subtype_of_source: bool,
    ) -> bool {
        let use_bivariance = self.should_use_bivariance(is_method, false);

        if use_bivariance {
            // Bivariant: either direction works
            source_subtype_of_target || target_subtype_of_source
        } else {
            // Contravariant: target must be subtype of source
            // (x: Animal) => void <: (x: Dog) => void
            // requires Dog <: Animal (target <: source)
            target_subtype_of_source
        }
    }

    /// Get a human-readable description of the current variance mode.
    pub fn describe_mode(&self) -> &'static str {
        if !self.strict_function_types {
            "bivariant (legacy TypeScript)"
        } else if self.methods_are_bivariant {
            "contravariant for functions, bivariant for methods (strictFunctionTypes)"
        } else {
            "fully contravariant (sound, non-TypeScript-compatible)"
        }
    }
}

impl Default for FunctionBivarianceConfig {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Freshness Tracking for Excess Property Checking
// =============================================================================

/// Tracks "freshness" of object types for excess property checking.
///
/// In TypeScript, object literals are "fresh" and trigger excess property
/// checking. Once assigned to a variable or passed through a type assertion,
/// they lose their freshness and allow width subtyping (extra properties).
///
/// ## Example
/// ```typescript
/// interface Point { x: number; y: number }
///
/// // Fresh object literal - EXCESS property error for 'z'
/// const p: Point = { x: 1, y: 2, z: 3 }; // Error: 'z' does not exist
///
/// // Not fresh - assigned to a variable first, then passed
/// const temp = { x: 1, y: 2, z: 3 };
/// const p2: Point = temp; // OK - temp is not fresh
/// ```
///
/// ## Usage
/// The FreshnessTracker should be used by expression-level type checking,
/// not by the subtype checker. Freshness is an expression concept, not a
/// type concept.
#[derive(Debug, Default)]
pub struct FreshnessTracker {
    /// Set of TypeIds that are currently "fresh" (object literals).
    fresh_types: rustc_hash::FxHashSet<TypeId>,
}

impl FreshnessTracker {
    /// Create a new FreshnessTracker.
    pub fn new() -> Self {
        FreshnessTracker {
            fresh_types: rustc_hash::FxHashSet::default(),
        }
    }

    /// Mark a type as fresh (usually when creating an object literal).
    pub fn mark_fresh(&mut self, type_id: TypeId) {
        self.fresh_types.insert(type_id);
    }

    /// Remove freshness from a type (when assigned to a variable, etc.).
    pub fn remove_freshness(&mut self, type_id: TypeId) {
        self.fresh_types.remove(&type_id);
    }

    /// Check if a type is fresh.
    pub fn is_fresh(&self, type_id: TypeId) -> bool {
        self.fresh_types.contains(&type_id)
    }

    /// Clear all freshness tracking (e.g., when leaving a scope).
    pub fn clear(&mut self) {
        self.fresh_types.clear();
    }

    /// Check if excess property checking should be performed.
    pub fn should_check_excess_properties(&self, source: TypeId) -> bool {
        self.is_fresh(source)
    }
}

// =============================================================================
// TypeScript Quirks Summary
// =============================================================================

/// Summary of TypeScript quirks handled by the Lawyer layer.
///
/// This struct provides documentation and helper methods for understanding
/// and configuring the various TypeScript compatibility behaviors.
pub struct TypeScriptQuirks;

impl TypeScriptQuirks {
    /// List of all TypeScript quirks handled by the Lawyer layer.
    pub const QUIRKS: &'static [(&'static str, &'static str)] = &[
        (
            "any-propagation",
            "any is both top and bottom type (assignable to/from everything)",
        ),
        (
            "function-bivariance",
            "Function parameters are bivariant in legacy mode",
        ),
        (
            "method-bivariance",
            "Methods are always bivariant regardless of strictFunctionTypes",
        ),
        (
            "void-return",
            "() => void accepts () => T for any T",
        ),
        (
            "weak-types",
            "Objects with only optional properties require common properties (TS2559)",
        ),
        (
            "freshness",
            "Object literals trigger excess property checking",
        ),
        (
            "empty-object",
            "{} accepts any non-nullish value including primitives",
        ),
        (
            "null-undefined",
            "null and undefined are assignable to everything without strictNullChecks",
        ),
        (
            "bivariant-rest",
            "Rest parameters of any/unknown are treated as bivariant",
        ),
    ];
}

#[cfg(test)]
#[path = "lawyer_tests.rs"]
mod tests;
