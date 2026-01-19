//! Variance annotations and checking
//!
//! This module implements variance checking for TypeScript type parameters:
//! - `in` modifier for contravariant type parameters
//! - `out` modifier for covariant type parameters
//! - `in out` for invariant type parameters
//! - Variance inference when not explicitly annotated
//!
//! # Variance Explained
//!
//! ```typescript
//! // Covariant (out): Type flows "out" (return positions)
//! interface Producer<out T> { get(): T; }
//!
//! // Contravariant (in): Type flows "in" (parameter positions)
//! interface Consumer<in T> { set(value: T): void; }
//!
//! // Invariant (in out): Type flows both ways
//! interface Container<in out T> { get(): T; set(value: T): void; }
//!
//! // Bivariant: No variance restriction (legacy behavior)
//! interface Handler<T> { handle(x: T): T; }
//! ```

use crate::ast::{NodeId, StringId, Span};
use super::signatures::{TypeId, TypeFlags};

/// Variance annotation for type parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Variance {
    /// No explicit variance (will be inferred or bivariant)
    #[default]
    None = 0,
    /// Covariant: `out T` - subtype relationships preserved
    Covariant = 1,
    /// Contravariant: `in T` - subtype relationships reversed
    Contravariant = 2,
    /// Invariant: `in out T` - exact type match required
    Invariant = 3,
    /// Bivariant: Both co- and contra-variant (unsound but sometimes necessary)
    Bivariant = 4,
}

impl Variance {
    /// Check if this variance is covariant
    #[inline]
    pub const fn is_covariant(self) -> bool {
        matches!(self, Variance::Covariant | Variance::Bivariant)
    }

    /// Check if this variance is contravariant
    #[inline]
    pub const fn is_contravariant(self) -> bool {
        matches!(self, Variance::Contravariant | Variance::Bivariant)
    }

    /// Check if this variance is invariant
    #[inline]
    pub const fn is_invariant(self) -> bool {
        matches!(self, Variance::Invariant)
    }

    /// Combine two variances (for nested type positions)
    pub const fn combine(self, other: Variance) -> Variance {
        match (self, other) {
            // None propagates
            (Variance::None, v) | (v, Variance::None) => v,
            // Invariant stays invariant
            (Variance::Invariant, _) | (_, Variance::Invariant) => Variance::Invariant,
            // Bivariant stays bivariant
            (Variance::Bivariant, _) | (_, Variance::Bivariant) => Variance::Bivariant,
            // Same variance stays same
            (Variance::Covariant, Variance::Covariant) => Variance::Covariant,
            (Variance::Contravariant, Variance::Contravariant) => Variance::Covariant, // Double flip
            // Opposite variances: flip
            (Variance::Covariant, Variance::Contravariant) => Variance::Contravariant,
            (Variance::Contravariant, Variance::Covariant) => Variance::Contravariant,
        }
    }

    /// Flip the variance (for contravariant positions)
    pub const fn flip(self) -> Variance {
        match self {
            Variance::Covariant => Variance::Contravariant,
            Variance::Contravariant => Variance::Covariant,
            v => v,
        }
    }
}

impl std::fmt::Display for Variance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variance::None => write!(f, ""),
            Variance::Covariant => write!(f, "out"),
            Variance::Contravariant => write!(f, "in"),
            Variance::Invariant => write!(f, "in out"),
            Variance::Bivariant => write!(f, "bivariant"),
        }
    }
}

/// Error codes for variance checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum VarianceErrorCode {
    /// Type parameter used in invalid position for declared variance
    InvalidVarianceUsage = 2636,
    /// Covariant type parameter used in contravariant position
    CovariantInContravariantPosition = 2637,
    /// Contravariant type parameter used in covariant position
    ContravariantInCovariantPosition = 2638,
    /// Invariant type parameter has conflicting usages
    InvariantMismatch = 2639,
    /// Variance annotation conflicts with inferred variance
    VarianceAnnotationMismatch = 2640,
    /// Cannot use variance annotation on this declaration
    InvalidVarianceAnnotationContext = 1273,
}

impl VarianceErrorCode {
    pub fn code(self) -> u32 {
        self as u32
    }

    pub fn message(self) -> &'static str {
        match self {
            VarianceErrorCode::InvalidVarianceUsage => {
                "Type parameter is used in a position that conflicts with its variance."
            }
            VarianceErrorCode::CovariantInContravariantPosition => {
                "Type parameter declared as 'out' is used in an 'in' position."
            }
            VarianceErrorCode::ContravariantInCovariantPosition => {
                "Type parameter declared as 'in' is used in an 'out' position."
            }
            VarianceErrorCode::InvariantMismatch => {
                "Type parameter declared as 'in out' requires exact type match."
            }
            VarianceErrorCode::VarianceAnnotationMismatch => {
                "Variance annotation does not match the type parameter's actual usage."
            }
            VarianceErrorCode::InvalidVarianceAnnotationContext => {
                "Variance annotations are only allowed on type parameters of interfaces and type aliases."
            }
        }
    }
}

/// Error from variance checking
#[derive(Debug, Clone)]
pub struct VarianceError {
    pub code: VarianceErrorCode,
    pub span: Span,
    pub message: String,
    /// The type parameter that has the variance issue
    pub type_param_name: StringId,
    /// Declared variance
    pub declared_variance: Variance,
    /// Required variance at the usage site
    pub required_variance: Variance,
}

impl VarianceError {
    pub fn new(
        code: VarianceErrorCode,
        span: Span,
        type_param_name: StringId,
        declared: Variance,
        required: Variance,
    ) -> Self {
        VarianceError {
            code,
            span,
            message: code.message().to_string(),
            type_param_name,
            declared_variance: declared,
            required_variance: required,
        }
    }
}

/// Position in a type where a type parameter appears
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypePosition {
    /// Covariant position (return types, property types)
    Covariant,
    /// Contravariant position (parameter types)
    Contravariant,
    /// Invariant position (both read and write)
    Invariant,
}

impl TypePosition {
    /// Get the variance required at this position
    pub fn required_variance(self) -> Variance {
        match self {
            TypePosition::Covariant => Variance::Covariant,
            TypePosition::Contravariant => Variance::Contravariant,
            TypePosition::Invariant => Variance::Invariant,
        }
    }

    /// Flip the position (for nested function types)
    pub fn flip(self) -> TypePosition {
        match self {
            TypePosition::Covariant => TypePosition::Contravariant,
            TypePosition::Contravariant => TypePosition::Covariant,
            TypePosition::Invariant => TypePosition::Invariant,
        }
    }
}

/// Type parameter with variance information
#[derive(Debug, Clone, Copy)]
pub struct VariantTypeParameter {
    /// Name of the type parameter
    pub name: StringId,
    /// Type ID for this parameter
    pub type_id: TypeId,
    /// Declared variance annotation
    pub declared_variance: Variance,
    /// Inferred variance from usage (if not explicitly declared)
    pub inferred_variance: Variance,
    /// Constraint type (extends clause)
    pub constraint: TypeId,
    /// Default type
    pub default: TypeId,
    /// Source location
    pub span: Span,
}

impl VariantTypeParameter {
    pub fn new(name: StringId, type_id: TypeId) -> Self {
        VariantTypeParameter {
            name,
            type_id,
            declared_variance: Variance::None,
            inferred_variance: Variance::None,
            constraint: TypeId::NONE,
            default: TypeId::NONE,
            span: Span::new(0, 0),
        }
    }

    pub fn with_variance(mut self, variance: Variance) -> Self {
        self.declared_variance = variance;
        self
    }

    pub fn covariant(mut self) -> Self {
        self.declared_variance = Variance::Covariant;
        self
    }

    pub fn contravariant(mut self) -> Self {
        self.declared_variance = Variance::Contravariant;
        self
    }

    pub fn invariant(mut self) -> Self {
        self.declared_variance = Variance::Invariant;
        self
    }

    pub fn with_constraint(mut self, constraint: TypeId) -> Self {
        self.constraint = constraint;
        self
    }

    pub fn with_default(mut self, default: TypeId) -> Self {
        self.default = default;
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    /// Get the effective variance (declared or inferred)
    pub fn effective_variance(&self) -> Variance {
        if self.declared_variance != Variance::None {
            self.declared_variance
        } else if self.inferred_variance != Variance::None {
            self.inferred_variance
        } else {
            // Default to bivariant if nothing specified (legacy TS behavior)
            Variance::Bivariant
        }
    }

    /// Check if variance is explicitly declared
    pub fn has_declared_variance(&self) -> bool {
        self.declared_variance != Variance::None
    }
}

/// Result of variance checking
#[derive(Debug, Clone)]
pub struct VarianceCheckResult {
    /// Whether the variance check passed
    pub is_valid: bool,
    /// Inferred variances for each type parameter
    pub inferred_variances: Vec<Variance>,
    /// Any errors encountered
    pub errors: Vec<VarianceError>,
}

impl VarianceCheckResult {
    pub fn success(inferred_variances: Vec<Variance>) -> Self {
        VarianceCheckResult {
            is_valid: true,
            inferred_variances,
            errors: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<VarianceError>) -> Self {
        VarianceCheckResult {
            is_valid: false,
            inferred_variances: Vec::new(),
            errors,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.is_valid && self.errors.is_empty()
    }
}

/// Tracks type parameter usages for variance inference
#[derive(Debug, Clone, Default)]
pub struct VarianceUsageTracker {
    /// Type parameters being tracked
    type_params: Vec<VariantTypeParameter>,
    /// Usage records: (type_param_index, position)
    usages: Vec<(usize, TypePosition)>,
}

impl VarianceUsageTracker {
    pub fn new() -> Self {
        VarianceUsageTracker {
            type_params: Vec::new(),
            usages: Vec::new(),
        }
    }

    /// Add a type parameter to track
    pub fn add_type_param(&mut self, param: VariantTypeParameter) -> usize {
        let index = self.type_params.len();
        self.type_params.push(param);
        index
    }

    /// Record a usage of a type parameter at a position
    pub fn record_usage(&mut self, param_index: usize, position: TypePosition) {
        self.usages.push((param_index, position));
    }

    /// Infer variances from recorded usages
    pub fn infer_variances(&mut self) -> Vec<Variance> {
        let mut variances: Vec<Option<Variance>> = vec![None; self.type_params.len()];

        for &(param_idx, position) in &self.usages {
            if param_idx >= variances.len() {
                continue;
            }

            let new_variance = match position {
                TypePosition::Covariant => Variance::Covariant,
                TypePosition::Contravariant => Variance::Contravariant,
                TypePosition::Invariant => Variance::Invariant,
            };

            variances[param_idx] = Some(match variances[param_idx] {
                None => new_variance,
                Some(Variance::Covariant) if new_variance == Variance::Contravariant => {
                    Variance::Invariant
                }
                Some(Variance::Contravariant) if new_variance == Variance::Covariant => {
                    Variance::Invariant
                }
                Some(v) if v == new_variance => v,
                Some(Variance::Invariant) => Variance::Invariant,
                Some(v) => v,
            });
        }

        // Set inferred variances on type params
        for (i, variance) in variances.iter().enumerate() {
            if let Some(v) = variance {
                if i < self.type_params.len() {
                    self.type_params[i].inferred_variance = *v;
                }
            }
        }

        variances
            .into_iter()
            .map(|v| v.unwrap_or(Variance::Bivariant))
            .collect()
    }
}

/// Variance checker
pub struct VarianceChecker {
    /// Collected errors
    errors: Vec<VarianceError>,
    /// Current position in the type
    current_position: TypePosition,
}

impl VarianceChecker {
    pub fn new() -> Self {
        VarianceChecker {
            errors: Vec::new(),
            current_position: TypePosition::Covariant,
        }
    }

    /// Check if a type parameter usage is valid for its declared variance
    pub fn check_usage(
        &mut self,
        param: &VariantTypeParameter,
        position: TypePosition,
    ) -> bool {
        let effective = param.effective_variance();

        let is_valid = match (effective, position) {
            // Bivariant is always valid
            (Variance::Bivariant, _) => true,
            // None is always valid (no checking)
            (Variance::None, _) => true,
            // Covariant must be in covariant position
            (Variance::Covariant, TypePosition::Covariant) => true,
            (Variance::Covariant, TypePosition::Contravariant) => false,
            // Contravariant must be in contravariant position
            (Variance::Contravariant, TypePosition::Contravariant) => true,
            (Variance::Contravariant, TypePosition::Covariant) => false,
            // Invariant is always valid (but requires exact match)
            (Variance::Invariant, _) => true,
            // For invariant positions, both covariant and contravariant params work
            (Variance::Covariant, TypePosition::Invariant) => true,
            (Variance::Contravariant, TypePosition::Invariant) => true,
        };

        if !is_valid {
            let error_code = match effective {
                Variance::Covariant => VarianceErrorCode::CovariantInContravariantPosition,
                Variance::Contravariant => VarianceErrorCode::ContravariantInCovariantPosition,
                _ => VarianceErrorCode::InvalidVarianceUsage,
            };

            self.errors.push(VarianceError::new(
                error_code,
                param.span,
                param.name,
                effective,
                position.required_variance(),
            ));
        }

        is_valid
    }

    /// Check that declared variance matches inferred variance
    pub fn check_variance_annotation(
        &mut self,
        param: &VariantTypeParameter,
        inferred: Variance,
    ) -> bool {
        if !param.has_declared_variance() {
            return true; // No declared variance to check
        }

        let declared = param.declared_variance;

        // Check compatibility
        let is_compatible = match (declared, inferred) {
            // Same variance is always fine
            (d, i) if d == i => true,
            // Declared bivariant is always fine
            (Variance::Bivariant, _) => true,
            // Inferred bivariant can match anything
            (_, Variance::Bivariant) => true,
            // Declared invariant when inferred covariant or contravariant is fine
            // (invariant is more restrictive)
            (Variance::Invariant, Variance::Covariant | Variance::Contravariant) => true,
            // Declared covariant when inferred invariant is an error
            (Variance::Covariant, Variance::Invariant) => false,
            // Declared contravariant when inferred invariant is an error
            (Variance::Contravariant, Variance::Invariant) => false,
            // Opposite variances are errors
            (Variance::Covariant, Variance::Contravariant) => false,
            (Variance::Contravariant, Variance::Covariant) => false,
            // None means no checking
            (Variance::None, _) | (_, Variance::None) => true,
        };

        if !is_compatible {
            self.errors.push(VarianceError::new(
                VarianceErrorCode::VarianceAnnotationMismatch,
                param.span,
                param.name,
                declared,
                inferred,
            ));
        }

        is_compatible
    }

    /// Check variance for a type parameter in a return type position
    pub fn check_in_return_position(&mut self, param: &VariantTypeParameter) -> bool {
        self.check_usage(param, TypePosition::Covariant)
    }

    /// Check variance for a type parameter in a parameter type position
    pub fn check_in_parameter_position(&mut self, param: &VariantTypeParameter) -> bool {
        self.check_usage(param, TypePosition::Contravariant)
    }

    /// Check variance for a type parameter in an invariant position (mutable ref)
    pub fn check_in_invariant_position(&mut self, param: &VariantTypeParameter) -> bool {
        self.check_usage(param, TypePosition::Invariant)
    }

    /// Enter a contravariant context (e.g., function parameter)
    pub fn enter_contravariant<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let old_position = self.current_position;
        self.current_position = self.current_position.flip();
        let result = f(self);
        self.current_position = old_position;
        result
    }

    /// Get the current type position
    pub fn current_position(&self) -> TypePosition {
        self.current_position
    }

    /// Take all collected errors
    pub fn take_errors(&mut self) -> Vec<VarianceError> {
        std::mem::take(&mut self.errors)
    }

    /// Check all type parameters in a declaration
    pub fn check_type_parameters(
        &mut self,
        params: &[VariantTypeParameter],
        tracker: &VarianceUsageTracker,
    ) -> VarianceCheckResult {
        let inferred = tracker.type_params.iter()
            .map(|p| p.inferred_variance)
            .collect::<Vec<_>>();

        // Check each declared variance against inferred
        for (i, param) in params.iter().enumerate() {
            if let Some(&inf) = inferred.get(i) {
                self.check_variance_annotation(param, inf);
            }
        }

        let errors = self.take_errors();
        if errors.is_empty() {
            VarianceCheckResult::success(inferred)
        } else {
            VarianceCheckResult::failure(errors)
        }
    }
}

impl Default for VarianceChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if variance allows subtype relationship
pub fn variance_allows_subtype(
    variance: Variance,
    subtype: TypeId,
    supertype: TypeId,
) -> bool {
    match variance {
        Variance::Covariant => {
            // For covariant, subtype must be subtype of supertype (normal)
            // This would need actual subtype checking
            true
        }
        Variance::Contravariant => {
            // For contravariant, relationship is flipped
            // supertype must be subtype of subtype
            true
        }
        Variance::Invariant => {
            // For invariant, types must be exactly equal
            subtype == supertype
        }
        Variance::Bivariant | Variance::None => {
            // Both relationships are allowed
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variance_combine() {
        // Covariant + Covariant = Covariant
        assert_eq!(Variance::Covariant.combine(Variance::Covariant), Variance::Covariant);

        // Contravariant + Contravariant = Covariant (double flip)
        assert_eq!(
            Variance::Contravariant.combine(Variance::Contravariant),
            Variance::Covariant
        );

        // Covariant + Contravariant = Contravariant
        assert_eq!(
            Variance::Covariant.combine(Variance::Contravariant),
            Variance::Contravariant
        );

        // Invariant stays invariant
        assert_eq!(Variance::Invariant.combine(Variance::Covariant), Variance::Invariant);
    }

    #[test]
    fn test_variance_flip() {
        assert_eq!(Variance::Covariant.flip(), Variance::Contravariant);
        assert_eq!(Variance::Contravariant.flip(), Variance::Covariant);
        assert_eq!(Variance::Invariant.flip(), Variance::Invariant);
        assert_eq!(Variance::Bivariant.flip(), Variance::Bivariant);
    }

    #[test]
    fn test_covariant_type_param() {
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .covariant();

        assert_eq!(param.declared_variance, Variance::Covariant);
        assert_eq!(param.effective_variance(), Variance::Covariant);
    }

    #[test]
    fn test_contravariant_type_param() {
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .contravariant();

        assert_eq!(param.declared_variance, Variance::Contravariant);
    }

    #[test]
    fn test_invariant_type_param() {
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .invariant();

        assert_eq!(param.declared_variance, Variance::Invariant);
    }

    #[test]
    fn test_covariant_in_return_position_valid() {
        let mut checker = VarianceChecker::new();
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .covariant();

        assert!(checker.check_in_return_position(&param));
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_covariant_in_parameter_position_invalid() {
        let mut checker = VarianceChecker::new();
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .covariant();

        assert!(!checker.check_in_parameter_position(&param));
        assert!(!checker.errors.is_empty());
        assert_eq!(
            checker.errors[0].code,
            VarianceErrorCode::CovariantInContravariantPosition
        );
    }

    #[test]
    fn test_contravariant_in_parameter_position_valid() {
        let mut checker = VarianceChecker::new();
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .contravariant();

        assert!(checker.check_in_parameter_position(&param));
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_contravariant_in_return_position_invalid() {
        let mut checker = VarianceChecker::new();
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .contravariant();

        assert!(!checker.check_in_return_position(&param));
        assert!(!checker.errors.is_empty());
        assert_eq!(
            checker.errors[0].code,
            VarianceErrorCode::ContravariantInCovariantPosition
        );
    }

    #[test]
    fn test_invariant_in_any_position_valid() {
        let mut checker = VarianceChecker::new();
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .invariant();

        assert!(checker.check_in_return_position(&param));
        assert!(checker.check_in_parameter_position(&param));
        assert!(checker.check_in_invariant_position(&param));
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_bivariant_anywhere_valid() {
        let mut checker = VarianceChecker::new();
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .with_variance(Variance::Bivariant);

        assert!(checker.check_in_return_position(&param));
        assert!(checker.check_in_parameter_position(&param));
        assert!(checker.errors.is_empty());
    }

    #[test]
    fn test_variance_inference() {
        let mut tracker = VarianceUsageTracker::new();

        // Add a type parameter
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100));
        let idx = tracker.add_type_param(param);

        // Use it in covariant position
        tracker.record_usage(idx, TypePosition::Covariant);

        let variances = tracker.infer_variances();
        assert_eq!(variances[0], Variance::Covariant);
    }

    #[test]
    fn test_variance_inference_invariant() {
        let mut tracker = VarianceUsageTracker::new();

        // Add a type parameter
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100));
        let idx = tracker.add_type_param(param);

        // Use it in both positions -> invariant
        tracker.record_usage(idx, TypePosition::Covariant);
        tracker.record_usage(idx, TypePosition::Contravariant);

        let variances = tracker.infer_variances();
        assert_eq!(variances[0], Variance::Invariant);
    }

    #[test]
    fn test_variance_annotation_check() {
        let mut checker = VarianceChecker::new();

        // Declared covariant, used only in covariant - OK
        let param = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .covariant();
        assert!(checker.check_variance_annotation(&param, Variance::Covariant));

        // Declared covariant, used in both positions (invariant) - Error
        let mut checker2 = VarianceChecker::new();
        assert!(!checker2.check_variance_annotation(&param, Variance::Invariant));
    }

    #[test]
    fn test_enter_contravariant() {
        let mut checker = VarianceChecker::new();
        assert_eq!(checker.current_position(), TypePosition::Covariant);

        checker.enter_contravariant(|c| {
            assert_eq!(c.current_position(), TypePosition::Contravariant);
        });

        assert_eq!(checker.current_position(), TypePosition::Covariant);
    }

    #[test]
    fn test_type_position_flip() {
        assert_eq!(TypePosition::Covariant.flip(), TypePosition::Contravariant);
        assert_eq!(TypePosition::Contravariant.flip(), TypePosition::Covariant);
        assert_eq!(TypePosition::Invariant.flip(), TypePosition::Invariant);
    }

    #[test]
    fn test_effective_variance() {
        // No declared variance, no inferred -> bivariant (default)
        let param1 = VariantTypeParameter::new(StringId::new(1), TypeId::new(100));
        assert_eq!(param1.effective_variance(), Variance::Bivariant);

        // Declared variance takes precedence
        let param2 = VariantTypeParameter::new(StringId::new(1), TypeId::new(100))
            .covariant();
        assert_eq!(param2.effective_variance(), Variance::Covariant);

        // Inferred when no declared
        let mut param3 = VariantTypeParameter::new(StringId::new(1), TypeId::new(100));
        param3.inferred_variance = Variance::Contravariant;
        assert_eq!(param3.effective_variance(), Variance::Contravariant);
    }
}
