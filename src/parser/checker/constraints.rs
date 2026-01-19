//! Type parameter constraints checking
//!
//! This module implements constraint checking for TypeScript type parameters:
//! - `extends` clause validation
//! - Default type parameter validation
//! - Circular constraint detection
//! - Constraint satisfaction checking
//!
//! # Examples
//!
//! ```typescript
//! // Simple constraint
//! function identity<T extends object>(x: T): T { return x; }
//!
//! // Multiple constraints via intersection
//! function process<T extends Serializable & Comparable>(x: T): string { ... }
//!
//! // Default type parameter
//! function create<T = string>(): T { ... }
//!
//! // Circular constraint (error)
//! type Bad<T extends T> = T; // Error: circular constraint
//! ```

use crate::ast::{NodeId, StringId, Span};
use super::signatures::{TypeId, TypeFlags};
use super::variance::{Variance, VariantTypeParameter};

/// Error codes for constraint checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ConstraintErrorCode {
    /// Type does not satisfy constraint
    TypeDoesNotSatisfyConstraint = 2344,
    /// Circular constraint detected
    CircularConstraint = 2313,
    /// Type parameter default does not satisfy constraint
    DefaultDoesNotSatisfyConstraint = 2344,
    /// Type parameter default references itself
    CircularDefault = 2716,
    /// Required type parameter cannot follow optional
    RequiredAfterOptional = 2706,
    /// Type argument not assignable to constraint
    TypeArgumentNotAssignable = 2344,
    /// Constraint references unknown type parameter
    UnknownTypeParameter = 2304,
    /// Constraint is too complex
    ConstraintTooComplex = 2321,
}

impl ConstraintErrorCode {
    pub fn code(self) -> u32 {
        self as u32
    }

    pub fn message(self) -> &'static str {
        match self {
            ConstraintErrorCode::TypeDoesNotSatisfyConstraint => {
                "Type does not satisfy the constraint."
            }
            ConstraintErrorCode::CircularConstraint => {
                "Type parameter has a circular constraint."
            }
            ConstraintErrorCode::DefaultDoesNotSatisfyConstraint => {
                "Type parameter default does not satisfy the constraint."
            }
            ConstraintErrorCode::CircularDefault => {
                "Type parameter default references itself."
            }
            ConstraintErrorCode::RequiredAfterOptional => {
                "Required type parameters may not follow optional type parameters."
            }
            ConstraintErrorCode::TypeArgumentNotAssignable => {
                "Type argument is not assignable to the constraint."
            }
            ConstraintErrorCode::UnknownTypeParameter => {
                "Cannot find name in constraint."
            }
            ConstraintErrorCode::ConstraintTooComplex => {
                "Type instantiation is excessively deep and possibly infinite."
            }
        }
    }
}

/// Error from constraint checking
#[derive(Debug, Clone)]
pub struct ConstraintError {
    pub code: ConstraintErrorCode,
    pub span: Span,
    pub message: String,
    /// The type that failed to satisfy the constraint
    pub type_id: TypeId,
    /// The constraint that was not satisfied
    pub constraint: TypeId,
}

impl ConstraintError {
    pub fn new(code: ConstraintErrorCode, span: Span, type_id: TypeId, constraint: TypeId) -> Self {
        ConstraintError {
            code,
            span,
            message: code.message().to_string(),
            type_id,
            constraint,
        }
    }

    pub fn with_message(
        code: ConstraintErrorCode,
        span: Span,
        type_id: TypeId,
        constraint: TypeId,
        message: impl Into<String>,
    ) -> Self {
        ConstraintError {
            code,
            span,
            message: message.into(),
            type_id,
            constraint,
        }
    }
}

/// A type parameter with constraint information
#[derive(Debug, Clone)]
pub struct ConstrainedTypeParameter {
    /// Name of the type parameter
    pub name: StringId,
    /// Type ID for this parameter
    pub type_id: TypeId,
    /// Constraint type (extends clause)
    pub constraint: TypeId,
    /// Default type
    pub default: TypeId,
    /// Variance annotation
    pub variance: Variance,
    /// Source location
    pub span: Span,
    /// Index in the type parameter list
    pub index: usize,
}

impl ConstrainedTypeParameter {
    pub fn new(name: StringId, type_id: TypeId, index: usize) -> Self {
        ConstrainedTypeParameter {
            name,
            type_id,
            constraint: TypeId::NONE,
            default: TypeId::NONE,
            variance: Variance::None,
            span: Span::new(0, 0),
            index,
        }
    }

    pub fn with_constraint(mut self, constraint: TypeId) -> Self {
        self.constraint = constraint;
        self
    }

    pub fn with_default(mut self, default: TypeId) -> Self {
        self.default = default;
        self
    }

    pub fn with_variance(mut self, variance: Variance) -> Self {
        self.variance = variance;
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    /// Check if this parameter has a constraint
    pub fn has_constraint(&self) -> bool {
        self.constraint.is_some()
    }

    /// Check if this parameter has a default
    pub fn has_default(&self) -> bool {
        self.default.is_some()
    }

    /// Check if this parameter is optional (has default)
    pub fn is_optional(&self) -> bool {
        self.has_default()
    }
}

/// Result of constraint checking
#[derive(Debug, Clone)]
pub struct ConstraintCheckResult {
    /// Whether all constraints are satisfied
    pub is_satisfied: bool,
    /// Any errors encountered
    pub errors: Vec<ConstraintError>,
}

impl ConstraintCheckResult {
    pub fn success() -> Self {
        ConstraintCheckResult {
            is_satisfied: true,
            errors: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<ConstraintError>) -> Self {
        ConstraintCheckResult {
            is_satisfied: false,
            errors,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.is_satisfied && self.errors.is_empty()
    }
}

/// Context for constraint checking
#[derive(Debug, Clone, Default)]
pub struct ConstraintContext {
    /// Maximum recursion depth for constraint checking
    pub max_depth: usize,
    /// Current recursion depth
    pub current_depth: usize,
    /// Type parameters in scope (for circular detection)
    pub in_scope: Vec<TypeId>,
}

impl ConstraintContext {
    pub fn new() -> Self {
        ConstraintContext {
            max_depth: 100,
            current_depth: 0,
            in_scope: Vec::new(),
        }
    }

    /// Check if we've hit the recursion limit
    pub fn is_too_deep(&self) -> bool {
        self.current_depth >= self.max_depth
    }

    /// Enter a deeper level
    pub fn enter(&mut self) -> bool {
        if self.is_too_deep() {
            return false;
        }
        self.current_depth += 1;
        true
    }

    /// Leave a level
    pub fn leave(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }

    /// Check if a type is in scope (for circular detection)
    pub fn is_in_scope(&self, type_id: TypeId) -> bool {
        self.in_scope.contains(&type_id)
    }

    /// Add a type to scope
    pub fn add_to_scope(&mut self, type_id: TypeId) {
        self.in_scope.push(type_id);
    }

    /// Remove a type from scope
    pub fn remove_from_scope(&mut self, type_id: TypeId) {
        if let Some(pos) = self.in_scope.iter().position(|&t| t == type_id) {
            self.in_scope.remove(pos);
        }
    }
}

/// Constraint checker
pub struct ConstraintChecker {
    /// Context for checking
    context: ConstraintContext,
    /// Collected errors
    errors: Vec<ConstraintError>,
}

impl ConstraintChecker {
    pub fn new() -> Self {
        ConstraintChecker {
            context: ConstraintContext::new(),
            errors: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: ConstraintContext) -> Self {
        self.context = context;
        self
    }

    /// Check if a type satisfies a constraint
    pub fn check_constraint(
        &mut self,
        type_id: TypeId,
        constraint: TypeId,
        span: Span,
    ) -> bool {
        // No constraint means always satisfied
        if constraint.is_none() {
            return true;
        }

        // Check for recursion
        if !self.context.enter() {
            self.errors.push(ConstraintError::new(
                ConstraintErrorCode::ConstraintTooComplex,
                span,
                type_id,
                constraint,
            ));
            return false;
        }

        let result = self.is_type_assignable_to_constraint(type_id, constraint);

        self.context.leave();

        if !result {
            self.errors.push(ConstraintError::new(
                ConstraintErrorCode::TypeDoesNotSatisfyConstraint,
                span,
                type_id,
                constraint,
            ));
        }

        result
    }

    /// Check if a type is assignable to a constraint
    fn is_type_assignable_to_constraint(&self, type_id: TypeId, constraint: TypeId) -> bool {
        // Same type always satisfies
        if type_id == constraint {
            return true;
        }

        // any satisfies any constraint
        if type_id == TypeId::ANY {
            return true;
        }

        // never satisfies any constraint (bottom type)
        if type_id == TypeId::NEVER {
            return true;
        }

        // anything satisfies unknown constraint
        if constraint == TypeId::UNKNOWN || constraint == TypeId::ANY {
            return true;
        }

        // object constraint
        if constraint == TypeId::OBJECT {
            // Primitives don't satisfy object constraint
            return !type_id.is_primitive() || type_id == TypeId::OBJECT;
        }

        // In a real implementation, we'd check structural compatibility
        // For now, assume compatible for non-primitive types
        true
    }

    /// Check for circular constraints
    pub fn check_circular_constraint(
        &mut self,
        param: &ConstrainedTypeParameter,
    ) -> bool {
        if !param.has_constraint() {
            return true;
        }

        // Check if constraint references the parameter itself
        if self.context.is_in_scope(param.type_id) {
            self.errors.push(ConstraintError::new(
                ConstraintErrorCode::CircularConstraint,
                param.span,
                param.type_id,
                param.constraint,
            ));
            return false;
        }

        // Add to scope for nested checking
        self.context.add_to_scope(param.type_id);

        // In a real implementation, we'd recursively check the constraint type
        let result = self.check_constraint_references(param.constraint, param.type_id);

        self.context.remove_from_scope(param.type_id);

        result
    }

    /// Check if a constraint references a specific type parameter
    fn check_constraint_references(&self, constraint: TypeId, param_type: TypeId) -> bool {
        // Simple check: constraint directly references param
        if constraint == param_type {
            return false;
        }

        // In a real implementation, we'd recursively analyze the constraint type
        // to find any reference to the parameter
        true
    }

    /// Check that a default type satisfies its constraint
    pub fn check_default_constraint(
        &mut self,
        param: &ConstrainedTypeParameter,
    ) -> bool {
        if !param.has_default() {
            return true;
        }

        // Check for circular default reference
        if param.default == param.type_id {
            self.errors.push(ConstraintError::new(
                ConstraintErrorCode::CircularDefault,
                param.span,
                param.default,
                param.constraint,
            ));
            return false;
        }

        // Check default satisfies constraint
        if param.has_constraint() {
            return self.check_constraint(param.default, param.constraint, param.span);
        }

        true
    }

    /// Check type parameter ordering (required before optional)
    pub fn check_parameter_ordering(
        &mut self,
        params: &[ConstrainedTypeParameter],
    ) -> bool {
        let mut seen_optional = false;
        let mut all_valid = true;

        for param in params {
            if seen_optional && !param.is_optional() {
                self.errors.push(ConstraintError::with_message(
                    ConstraintErrorCode::RequiredAfterOptional,
                    param.span,
                    param.type_id,
                    TypeId::NONE,
                    "Required type parameters may not follow optional type parameters.",
                ));
                all_valid = false;
            }

            if param.is_optional() {
                seen_optional = true;
            }
        }

        all_valid
    }

    /// Check all constraints for a list of type parameters
    pub fn check_type_parameters(
        &mut self,
        params: &[ConstrainedTypeParameter],
    ) -> ConstraintCheckResult {
        // Check parameter ordering
        self.check_parameter_ordering(params);

        // Check each parameter
        for param in params {
            // Check for circular constraints
            self.check_circular_constraint(param);

            // Check default satisfies constraint
            self.check_default_constraint(param);
        }

        let errors = std::mem::take(&mut self.errors);
        if errors.is_empty() {
            ConstraintCheckResult::success()
        } else {
            ConstraintCheckResult::failure(errors)
        }
    }

    /// Check type arguments against constraints
    pub fn check_type_arguments(
        &mut self,
        params: &[ConstrainedTypeParameter],
        args: &[TypeId],
        span: Span,
    ) -> ConstraintCheckResult {
        // Check each provided argument
        for (i, &arg) in args.iter().enumerate() {
            if let Some(param) = params.get(i) {
                if param.has_constraint() {
                    self.check_constraint(arg, param.constraint, span);
                }
            }
        }

        // Check that required params have arguments
        for param in params.iter().skip(args.len()) {
            if !param.is_optional() {
                // Missing required type argument - would be caught elsewhere
            }
        }

        let errors = std::mem::take(&mut self.errors);
        if errors.is_empty() {
            ConstraintCheckResult::success()
        } else {
            ConstraintCheckResult::failure(errors)
        }
    }

    /// Take all collected errors
    pub fn take_errors(&mut self) -> Vec<ConstraintError> {
        std::mem::take(&mut self.errors)
    }
}

impl Default for ConstraintChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve default type parameters
pub fn resolve_defaults(
    params: &[ConstrainedTypeParameter],
    provided_args: &[TypeId],
) -> Vec<TypeId> {
    let mut result = Vec::with_capacity(params.len());

    for (i, param) in params.iter().enumerate() {
        if let Some(&arg) = provided_args.get(i) {
            result.push(arg);
        } else if param.has_default() {
            result.push(param.default);
        } else {
            // No argument and no default - use constraint or unknown
            if param.has_constraint() {
                result.push(param.constraint);
            } else {
                result.push(TypeId::UNKNOWN);
            }
        }
    }

    result
}

/// Check if minimum type arguments are provided
pub fn check_min_type_arguments(
    params: &[ConstrainedTypeParameter],
    args: &[TypeId],
) -> bool {
    let required = params.iter().filter(|p| !p.is_optional()).count();
    args.len() >= required
}

/// Get the minimum and maximum number of type arguments
pub fn type_argument_bounds(params: &[ConstrainedTypeParameter]) -> (usize, usize) {
    let min = params.iter().filter(|p| !p.is_optional()).count();
    let max = params.len();
    (min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_check_no_constraint() {
        let mut checker = ConstraintChecker::new();

        // No constraint means always satisfied
        assert!(checker.check_constraint(
            TypeId::STRING,
            TypeId::NONE,
            Span::new(0, 10),
        ));
    }

    #[test]
    fn test_constraint_check_same_type() {
        let mut checker = ConstraintChecker::new();

        // Same type satisfies
        assert!(checker.check_constraint(
            TypeId::STRING,
            TypeId::STRING,
            Span::new(0, 10),
        ));
    }

    #[test]
    fn test_constraint_check_any() {
        let mut checker = ConstraintChecker::new();

        // any satisfies any constraint
        assert!(checker.check_constraint(
            TypeId::ANY,
            TypeId::OBJECT,
            Span::new(0, 10),
        ));
    }

    #[test]
    fn test_constraint_check_never() {
        let mut checker = ConstraintChecker::new();

        // never satisfies any constraint
        assert!(checker.check_constraint(
            TypeId::NEVER,
            TypeId::STRING,
            Span::new(0, 10),
        ));
    }

    #[test]
    fn test_constraint_check_unknown_constraint() {
        let mut checker = ConstraintChecker::new();

        // anything satisfies unknown
        assert!(checker.check_constraint(
            TypeId::STRING,
            TypeId::UNKNOWN,
            Span::new(0, 10),
        ));
    }

    #[test]
    fn test_object_constraint() {
        let mut checker = ConstraintChecker::new();

        // object satisfies object
        assert!(checker.check_constraint(
            TypeId::OBJECT,
            TypeId::OBJECT,
            Span::new(0, 10),
        ));

        // primitive doesn't satisfy object
        assert!(!checker.check_constraint(
            TypeId::STRING,
            TypeId::OBJECT,
            Span::new(0, 10),
        ));
    }

    #[test]
    fn test_circular_constraint_detection() {
        let mut checker = ConstraintChecker::new();

        // Add the param to scope to simulate circular reference
        let param = ConstrainedTypeParameter::new(StringId::new(1), TypeId::new(100), 0)
            .with_constraint(TypeId::new(100)); // References itself

        checker.context.add_to_scope(TypeId::new(100));

        assert!(!checker.check_circular_constraint(&param));
    }

    #[test]
    fn test_default_constraint_check() {
        let mut checker = ConstraintChecker::new();

        // Default satisfies constraint
        let param = ConstrainedTypeParameter::new(StringId::new(1), TypeId::new(100), 0)
            .with_constraint(TypeId::OBJECT)
            .with_default(TypeId::OBJECT);

        assert!(checker.check_default_constraint(&param));
    }

    #[test]
    fn test_circular_default_detection() {
        let mut checker = ConstraintChecker::new();

        // Default references itself
        let param = ConstrainedTypeParameter::new(StringId::new(1), TypeId::new(100), 0)
            .with_default(TypeId::new(100)); // References itself

        assert!(!checker.check_default_constraint(&param));
        assert!(!checker.errors.is_empty());
        assert_eq!(checker.errors[0].code, ConstraintErrorCode::CircularDefault);
    }

    #[test]
    fn test_parameter_ordering_valid() {
        let mut checker = ConstraintChecker::new();

        let params = vec![
            // Required first
            ConstrainedTypeParameter::new(StringId::new(1), TypeId::new(100), 0),
            // Optional second
            ConstrainedTypeParameter::new(StringId::new(2), TypeId::new(101), 1)
                .with_default(TypeId::STRING),
        ];

        assert!(checker.check_parameter_ordering(&params));
    }

    #[test]
    fn test_parameter_ordering_invalid() {
        let mut checker = ConstraintChecker::new();

        let params = vec![
            // Optional first
            ConstrainedTypeParameter::new(StringId::new(1), TypeId::new(100), 0)
                .with_default(TypeId::STRING),
            // Required second - error!
            ConstrainedTypeParameter::new(StringId::new(2), TypeId::new(101), 1),
        ];

        assert!(!checker.check_parameter_ordering(&params));
        assert!(!checker.errors.is_empty());
        assert_eq!(checker.errors[0].code, ConstraintErrorCode::RequiredAfterOptional);
    }

    #[test]
    fn test_resolve_defaults() {
        let params = vec![
            ConstrainedTypeParameter::new(StringId::new(1), TypeId::new(100), 0),
            ConstrainedTypeParameter::new(StringId::new(2), TypeId::new(101), 1)
                .with_default(TypeId::STRING),
            ConstrainedTypeParameter::new(StringId::new(3), TypeId::new(102), 2)
                .with_default(TypeId::NUMBER),
        ];

        // Only provide first argument
        let provided = vec![TypeId::BOOLEAN];
        let resolved = resolve_defaults(&params, &provided);

        assert_eq!(resolved.len(), 3);
        assert_eq!(resolved[0], TypeId::BOOLEAN); // Provided
        assert_eq!(resolved[1], TypeId::STRING);  // Default
        assert_eq!(resolved[2], TypeId::NUMBER);  // Default
    }

    #[test]
    fn test_check_min_type_arguments() {
        let params = vec![
            // Required
            ConstrainedTypeParameter::new(StringId::new(1), TypeId::new(100), 0),
            // Optional
            ConstrainedTypeParameter::new(StringId::new(2), TypeId::new(101), 1)
                .with_default(TypeId::STRING),
        ];

        // At least 1 required
        assert!(check_min_type_arguments(&params, &[TypeId::BOOLEAN]));
        assert!(check_min_type_arguments(&params, &[TypeId::BOOLEAN, TypeId::NUMBER]));
        assert!(!check_min_type_arguments(&params, &[]));
    }

    #[test]
    fn test_type_argument_bounds() {
        let params = vec![
            ConstrainedTypeParameter::new(StringId::new(1), TypeId::new(100), 0),
            ConstrainedTypeParameter::new(StringId::new(2), TypeId::new(101), 1)
                .with_default(TypeId::STRING),
            ConstrainedTypeParameter::new(StringId::new(3), TypeId::new(102), 2)
                .with_default(TypeId::NUMBER),
        ];

        let (min, max) = type_argument_bounds(&params);
        assert_eq!(min, 1); // 1 required
        assert_eq!(max, 3); // 3 total
    }

    #[test]
    fn test_type_arguments_check() {
        let mut checker = ConstraintChecker::new();

        let params = vec![
            ConstrainedTypeParameter::new(StringId::new(1), TypeId::new(100), 0)
                .with_constraint(TypeId::OBJECT),
        ];

        // Provide object type - satisfies constraint
        let result = checker.check_type_arguments(
            &params,
            &[TypeId::OBJECT],
            Span::new(0, 10),
        );
        assert!(result.is_ok());

        // Provide primitive - fails constraint
        let mut checker2 = ConstraintChecker::new();
        let result2 = checker2.check_type_arguments(
            &params,
            &[TypeId::STRING],
            Span::new(0, 10),
        );
        assert!(!result2.is_ok());
    }

    #[test]
    fn test_constraint_context_depth() {
        let mut ctx = ConstraintContext::new();
        ctx.max_depth = 3;

        assert!(ctx.enter());
        assert!(ctx.enter());
        assert!(ctx.enter());
        assert!(!ctx.enter()); // Too deep

        ctx.leave();
        assert!(ctx.enter()); // OK again
    }

    #[test]
    fn test_constraint_context_scope() {
        let mut ctx = ConstraintContext::new();

        let type_id = TypeId::new(100);

        assert!(!ctx.is_in_scope(type_id));

        ctx.add_to_scope(type_id);
        assert!(ctx.is_in_scope(type_id));

        ctx.remove_from_scope(type_id);
        assert!(!ctx.is_in_scope(type_id));
    }

    #[test]
    fn test_constrained_type_parameter() {
        let param = ConstrainedTypeParameter::new(StringId::new(1), TypeId::new(100), 0)
            .with_constraint(TypeId::OBJECT)
            .with_default(TypeId::OBJECT)
            .with_variance(Variance::Covariant)
            .with_span(Span::new(5, 15));

        assert!(param.has_constraint());
        assert!(param.has_default());
        assert!(param.is_optional());
        assert_eq!(param.constraint, TypeId::OBJECT);
        assert_eq!(param.default, TypeId::OBJECT);
        assert_eq!(param.variance, Variance::Covariant);
    }
}
