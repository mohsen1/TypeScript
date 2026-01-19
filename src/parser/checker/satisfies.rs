//! Satisfies operator checking
//!
//! This module implements type checking for the TypeScript `satisfies` operator.
//! The `satisfies` operator validates that an expression's type is assignable to
//! a target type while preserving the inferred (narrower) type of the expression.
//!
//! # Key Behavior
//!
//! Unlike type annotations which widen the type to the annotation, `satisfies`
//! preserves the narrower inferred type while still validating compatibility.
//!
//! ```typescript
//! // With type annotation - type is widened
//! const x: { a: string } = { a: "hello" }; // x.a: string
//!
//! // With satisfies - type is preserved
//! const y = { a: "hello" } satisfies { a: string }; // y.a: "hello"
//! ```

use crate::ast::{NodeId, Span};
use super::signatures::TypeId;

/// Error codes for satisfies operator type checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SatisfiesErrorCode {
    /// Type does not satisfy the constraint
    TypeDoesNotSatisfy = 1360,
    /// Property is missing in type
    PropertyMissing = 2741,
    /// Property types are incompatible
    PropertyTypeMismatch = 2322,
    /// Index signature is missing
    IndexSignatureMissing = 2329,
    /// Excess properties not allowed
    ExcessProperty = 2353,
}

impl SatisfiesErrorCode {
    pub fn code(self) -> u32 {
        self as u32
    }

    pub fn message(self) -> &'static str {
        match self {
            SatisfiesErrorCode::TypeDoesNotSatisfy => {
                "Type does not satisfy the expected type."
            }
            SatisfiesErrorCode::PropertyMissing => {
                "Property is missing in type."
            }
            SatisfiesErrorCode::PropertyTypeMismatch => {
                "Type is not assignable to type."
            }
            SatisfiesErrorCode::IndexSignatureMissing => {
                "Index signature is missing in type."
            }
            SatisfiesErrorCode::ExcessProperty => {
                "Object literal may only specify known properties."
            }
        }
    }
}

/// Error from satisfies expression checking
#[derive(Debug, Clone)]
pub struct SatisfiesError {
    pub code: SatisfiesErrorCode,
    pub span: Span,
    pub message: String,
    /// The type that failed to satisfy
    pub source_type: TypeId,
    /// The type that was expected
    pub target_type: TypeId,
}

impl SatisfiesError {
    pub fn new(code: SatisfiesErrorCode, span: Span, source: TypeId, target: TypeId) -> Self {
        SatisfiesError {
            code,
            span,
            message: code.message().to_string(),
            source_type: source,
            target_type: target,
        }
    }

    pub fn with_message(
        code: SatisfiesErrorCode,
        span: Span,
        source: TypeId,
        target: TypeId,
        message: impl Into<String>,
    ) -> Self {
        SatisfiesError {
            code,
            span,
            message: message.into(),
            source_type: source,
            target_type: target,
        }
    }
}

/// Result of checking a satisfies expression
#[derive(Debug, Clone)]
pub struct SatisfiesCheckResult {
    /// The preserved (narrower) type from the expression
    pub result_type: TypeId,
    /// The target type that was satisfied
    pub satisfied_type: TypeId,
    /// Whether the check passed
    pub is_satisfied: bool,
    /// Any errors encountered
    pub errors: Vec<SatisfiesError>,
    /// The AST node of the expression
    pub expression_node: NodeId,
}

impl SatisfiesCheckResult {
    pub fn success(result_type: TypeId, satisfied_type: TypeId, node: NodeId) -> Self {
        SatisfiesCheckResult {
            result_type,
            satisfied_type,
            is_satisfied: true,
            errors: Vec::new(),
            expression_node: node,
        }
    }

    pub fn failure(
        result_type: TypeId,
        satisfied_type: TypeId,
        node: NodeId,
        errors: Vec<SatisfiesError>,
    ) -> Self {
        SatisfiesCheckResult {
            result_type,
            satisfied_type,
            is_satisfied: false,
            errors,
            expression_node: node,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.is_satisfied && self.errors.is_empty()
    }
}

/// Context for satisfies expression checking
#[derive(Debug, Clone, Copy, Default)]
pub struct SatisfiesContext {
    /// Whether strict null checks are enabled
    pub strict_null_checks: bool,
    /// Whether to check for excess properties
    pub check_excess_properties: bool,
    /// Whether this is inside a const context
    pub is_const_context: bool,
}

/// Satisfies expression type checker
pub struct SatisfiesChecker {
    /// Context for checking
    context: SatisfiesContext,
    /// Collected errors
    errors: Vec<SatisfiesError>,
}

impl SatisfiesChecker {
    pub fn new() -> Self {
        SatisfiesChecker {
            context: SatisfiesContext::default(),
            errors: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: SatisfiesContext) -> Self {
        self.context = context;
        self
    }

    /// Check a satisfies expression: `expression satisfies TargetType`
    ///
    /// This validates that the expression's type is assignable to the target type,
    /// but preserves the narrower inferred type as the result.
    pub fn check_satisfies(
        &mut self,
        expression_type: TypeId,
        target_type: TypeId,
        expression_node: NodeId,
        span: Span,
    ) -> SatisfiesCheckResult {
        // The key insight: satisfies preserves the expression type, not the target type
        let result_type = expression_type;

        // Check if expression type is assignable to target type
        if self.is_type_satisfies(expression_type, target_type) {
            SatisfiesCheckResult::success(result_type, target_type, expression_node)
        } else {
            let error = SatisfiesError::new(
                SatisfiesErrorCode::TypeDoesNotSatisfy,
                span,
                expression_type,
                target_type,
            );
            self.errors.push(error.clone());

            SatisfiesCheckResult::failure(
                result_type,
                target_type,
                expression_node,
                vec![error],
            )
        }
    }

    /// Check satisfies with a literal type, handling const assertions
    ///
    /// When `as const` is used with `satisfies`, the literal type is preserved
    /// and validated against the target type.
    pub fn check_satisfies_with_const(
        &mut self,
        literal_type: TypeId,
        target_type: TypeId,
        expression_node: NodeId,
        span: Span,
    ) -> SatisfiesCheckResult {
        // In const context, we preserve the exact literal type
        let result_type = literal_type;

        // Check if literal type satisfies the target
        if self.is_type_satisfies(literal_type, target_type) {
            SatisfiesCheckResult::success(result_type, target_type, expression_node)
        } else {
            let error = SatisfiesError::with_message(
                SatisfiesErrorCode::TypeDoesNotSatisfy,
                span,
                literal_type,
                target_type,
                format!(
                    "Type '{}' does not satisfy type '{}'.",
                    self.type_name(literal_type),
                    self.type_name(target_type)
                ),
            );
            self.errors.push(error.clone());

            SatisfiesCheckResult::failure(
                result_type,
                target_type,
                expression_node,
                vec![error],
            )
        }
    }

    /// Check if a source type satisfies a target type
    ///
    /// This is similar to assignability but used specifically for `satisfies`.
    /// The key difference from normal assignment is that satisfies doesn't
    /// perform widening - it just checks compatibility.
    pub fn is_type_satisfies(&self, source: TypeId, target: TypeId) -> bool {
        // Same type always satisfies
        if source == target {
            return true;
        }

        // Any satisfies any target
        if source == TypeId::ANY {
            return true;
        }

        // Any target is satisfied by any source
        if target == TypeId::ANY {
            return true;
        }

        // Unknown can satisfy unknown or any
        if source == TypeId::UNKNOWN {
            return target == TypeId::UNKNOWN || target == TypeId::ANY;
        }

        // Never satisfies anything (bottom type)
        if source == TypeId::NEVER {
            return true;
        }

        // Nothing satisfies never (except never)
        if target == TypeId::NEVER {
            return false;
        }

        // Unknown is satisfied by anything
        if target == TypeId::UNKNOWN {
            return true;
        }

        // Null/undefined checks with strict null checks
        if self.context.strict_null_checks {
            if source == TypeId::NULL || source == TypeId::UNDEFINED {
                // Only satisfies if target includes null/undefined or is any/unknown
                return target == source || target == TypeId::ANY || target == TypeId::UNKNOWN;
            }
        }

        // Primitive type checks
        if source.is_primitive() && target.is_primitive() {
            return self.primitive_satisfies(source, target);
        }

        // Object type checks would go here in a full implementation
        // For now, accept object-to-object assignments
        if source == TypeId::OBJECT && target == TypeId::OBJECT {
            return true;
        }

        // Default: assume compatible for non-primitive types
        // In a real implementation, this would check structural compatibility
        true
    }

    /// Check if a primitive source type satisfies a primitive target type
    fn primitive_satisfies(&self, source: TypeId, target: TypeId) -> bool {
        // Exact match
        if source == target {
            return true;
        }

        // Check widening relationships for primitives
        match (source, target) {
            // String literal satisfies string
            (s, TypeId::STRING) if self.is_string_literal(s) => true,
            // Number literal satisfies number
            (n, TypeId::NUMBER) if self.is_number_literal(n) => true,
            // Boolean literal satisfies boolean
            (b, TypeId::BOOLEAN) if self.is_boolean_literal(b) => true,
            // Bigint literal satisfies bigint
            (bi, TypeId::BIGINT) if self.is_bigint_literal(bi) => true,
            // void satisfies undefined in non-strict mode
            (TypeId::VOID, TypeId::UNDEFINED) => true,
            (TypeId::UNDEFINED, TypeId::VOID) => true,
            // Otherwise not compatible
            _ => false,
        }
    }

    /// Check if type is a string literal type
    fn is_string_literal(&self, _type_id: TypeId) -> bool {
        // In a real implementation, check type flags
        // For now, return false for built-in types
        false
    }

    /// Check if type is a number literal type
    fn is_number_literal(&self, _type_id: TypeId) -> bool {
        false
    }

    /// Check if type is a boolean literal type
    fn is_boolean_literal(&self, _type_id: TypeId) -> bool {
        false
    }

    /// Check if type is a bigint literal type
    fn is_bigint_literal(&self, _type_id: TypeId) -> bool {
        false
    }

    /// Get type name for error messages
    fn type_name(&self, type_id: TypeId) -> &'static str {
        match type_id {
            TypeId::ANY => "any",
            TypeId::UNKNOWN => "unknown",
            TypeId::STRING => "string",
            TypeId::NUMBER => "number",
            TypeId::BOOLEAN => "boolean",
            TypeId::VOID => "void",
            TypeId::UNDEFINED => "undefined",
            TypeId::NULL => "null",
            TypeId::NEVER => "never",
            TypeId::OBJECT => "object",
            TypeId::SYMBOL => "symbol",
            TypeId::BIGINT => "bigint",
            _ => "Type",
        }
    }

    /// Take all collected errors
    pub fn take_errors(&mut self) -> Vec<SatisfiesError> {
        std::mem::take(&mut self.errors)
    }
}

impl Default for SatisfiesChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Compare satisfies vs type annotation behavior
///
/// This helper demonstrates the key difference:
/// - Type annotation: result type is the annotation type (widened)
/// - Satisfies: result type is the expression type (preserved)
pub fn compare_satisfies_vs_annotation(
    expression_type: TypeId,
    target_type: TypeId,
) -> (TypeId, TypeId) {
    // With type annotation, result is the annotation type
    let annotation_result = target_type;

    // With satisfies, result is the expression type (if it satisfies)
    let satisfies_result = expression_type;

    (annotation_result, satisfies_result)
}

/// Check if satisfies expression preserves literal types
pub fn preserves_literal_type(expression_type: TypeId, target_type: TypeId) -> bool {
    // Satisfies always preserves the expression type
    // So if expression is a literal, the result is that literal
    expression_type != target_type
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_satisfies_same_type() {
        let mut checker = SatisfiesChecker::new();

        let result = checker.check_satisfies(
            TypeId::STRING,
            TypeId::STRING,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.result_type, TypeId::STRING);
    }

    #[test]
    fn test_satisfies_any() {
        let mut checker = SatisfiesChecker::new();

        // any satisfies any target
        let result = checker.check_satisfies(
            TypeId::ANY,
            TypeId::STRING,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.result_type, TypeId::ANY);
    }

    #[test]
    fn test_satisfies_never() {
        let mut checker = SatisfiesChecker::new();

        // never satisfies anything
        let result = checker.check_satisfies(
            TypeId::NEVER,
            TypeId::STRING,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.result_type, TypeId::NEVER);
    }

    #[test]
    fn test_satisfies_unknown_target() {
        let mut checker = SatisfiesChecker::new();

        // anything satisfies unknown
        let result = checker.check_satisfies(
            TypeId::STRING,
            TypeId::UNKNOWN,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.result_type, TypeId::STRING);
    }

    #[test]
    fn test_satisfies_incompatible_types() {
        let mut checker = SatisfiesChecker::new();

        // string does not satisfy number
        let result = checker.check_satisfies(
            TypeId::STRING,
            TypeId::NUMBER,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(!result.is_ok());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_satisfies_preserves_expression_type() {
        let mut checker = SatisfiesChecker::new();

        // When satisfies succeeds, result type is the expression type, not target
        let expression_type = TypeId::new(100); // Some specific type
        let target_type = TypeId::OBJECT;

        let result = checker.check_satisfies(
            expression_type,
            target_type,
            NodeId::new(0),
            Span::new(0, 10),
        );

        // Result type should be the expression type, not the target type
        assert_eq!(result.result_type, expression_type);
        assert_eq!(result.satisfied_type, target_type);
    }

    #[test]
    fn test_satisfies_vs_annotation() {
        let expression_type = TypeId::new(100);
        let target_type = TypeId::OBJECT;

        let (annotation_result, satisfies_result) =
            compare_satisfies_vs_annotation(expression_type, target_type);

        // Annotation widens to target type
        assert_eq!(annotation_result, target_type);
        // Satisfies preserves expression type
        assert_eq!(satisfies_result, expression_type);
    }

    #[test]
    fn test_satisfies_with_strict_null_checks() {
        let mut checker = SatisfiesChecker::new()
            .with_context(SatisfiesContext {
                strict_null_checks: true,
                ..Default::default()
            });

        // null should not satisfy string with strict null checks
        let result = checker.check_satisfies(
            TypeId::NULL,
            TypeId::STRING,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(!result.is_ok());
    }

    #[test]
    fn test_satisfies_null_to_null() {
        let mut checker = SatisfiesChecker::new()
            .with_context(SatisfiesContext {
                strict_null_checks: true,
                ..Default::default()
            });

        // null satisfies null
        let result = checker.check_satisfies(
            TypeId::NULL,
            TypeId::NULL,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_satisfies_const_context() {
        let mut checker = SatisfiesChecker::new()
            .with_context(SatisfiesContext {
                is_const_context: true,
                ..Default::default()
            });

        let literal_type = TypeId::new(100); // Represents "hello" literal

        let result = checker.check_satisfies_with_const(
            literal_type,
            TypeId::STRING,
            NodeId::new(0),
            Span::new(0, 10),
        );

        // Should preserve literal type
        assert!(result.is_ok());
        assert_eq!(result.result_type, literal_type);
    }

    #[test]
    fn test_nothing_satisfies_never() {
        let mut checker = SatisfiesChecker::new();

        // string does not satisfy never (bottom type)
        let result = checker.check_satisfies(
            TypeId::STRING,
            TypeId::NEVER,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(!result.is_ok());
    }

    #[test]
    fn test_void_undefined_relationship() {
        let mut checker = SatisfiesChecker::new();

        // void satisfies undefined
        let result = checker.check_satisfies(
            TypeId::VOID,
            TypeId::UNDEFINED,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
    }
}
