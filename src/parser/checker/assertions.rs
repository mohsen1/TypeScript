//! Type assertions checking
//!
//! This module implements type checking for TypeScript type assertions:
//! - `as T` syntax (modern TypeScript)
//! - `<T>` syntax (legacy/JSX-conflicting)
//! - `as const` assertions
//! - Non-null assertion operator (`!`)
//!
//! # Type Assertion Rules
//!
//! Type assertions are only allowed when there is sufficient "overlap" between
//! the source and target types. This prevents obviously incorrect assertions.
//!
//! ```typescript
//! // Valid assertions (types overlap)
//! const x = "hello" as string;  // literal -> primitive
//! const y = expr as unknown;     // anything -> unknown
//! const z = unknown as number;   // unknown -> anything
//!
//! // Invalid without going through unknown
//! const bad = "hello" as number; // Error: no overlap
//! const ok = "hello" as unknown as number; // OK: double assertion
//! ```

use crate::ast::{NodeId, Span};
use super::signatures::{TypeId, TypeFlags};

/// Error codes for type assertion checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AssertionErrorCode {
    /// Conversion between types may be a mistake
    ConversionMayBeMistake = 2352,
    /// Cannot use JSX syntax with type assertions
    JsxAssertionNotAllowed = 17008,
    /// Object is possibly null
    ObjectPossiblyNull = 2531,
    /// Object is possibly undefined
    ObjectPossiblyUndefined = 2532,
    /// Object is possibly null or undefined
    ObjectPossiblyNullOrUndefined = 2533,
    /// Type assertion to 'const' is only valid for literals
    ConstAssertionOnNonLiteral = 1355,
    /// Type assertion using 'as' is redundant
    RedundantAssertion = 2367,
}

impl AssertionErrorCode {
    pub fn code(self) -> u32 {
        self as u32
    }

    pub fn message(self) -> &'static str {
        match self {
            AssertionErrorCode::ConversionMayBeMistake => {
                "Conversion of type to type may be a mistake because neither type sufficiently overlaps with the other."
            }
            AssertionErrorCode::JsxAssertionNotAllowed => {
                "JSX element type assertions are not allowed in .tsx files."
            }
            AssertionErrorCode::ObjectPossiblyNull => {
                "Object is possibly 'null'."
            }
            AssertionErrorCode::ObjectPossiblyUndefined => {
                "Object is possibly 'undefined'."
            }
            AssertionErrorCode::ObjectPossiblyNullOrUndefined => {
                "Object is possibly 'null' or 'undefined'."
            }
            AssertionErrorCode::ConstAssertionOnNonLiteral => {
                "A 'const' assertion can only be applied to a string, number, bigint, boolean, array, or object literal."
            }
            AssertionErrorCode::RedundantAssertion => {
                "This assertion is unnecessary since it does not change the type of the expression."
            }
        }
    }
}

/// Error from type assertion checking
#[derive(Debug, Clone)]
pub struct AssertionError {
    pub code: AssertionErrorCode,
    pub span: Span,
    pub message: String,
    /// The source type being asserted from
    pub source_type: TypeId,
    /// The target type being asserted to
    pub target_type: TypeId,
}

impl AssertionError {
    pub fn new(code: AssertionErrorCode, span: Span, source: TypeId, target: TypeId) -> Self {
        AssertionError {
            code,
            span,
            message: code.message().to_string(),
            source_type: source,
            target_type: target,
        }
    }

    pub fn with_message(
        code: AssertionErrorCode,
        span: Span,
        source: TypeId,
        target: TypeId,
        message: impl Into<String>,
    ) -> Self {
        AssertionError {
            code,
            span,
            message: message.into(),
            source_type: source,
            target_type: target,
        }
    }
}

/// Kind of type assertion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssertionKind {
    /// `expression as Type` syntax
    As,
    /// `<Type>expression` syntax (angle bracket)
    AngleBracket,
    /// `expression as const` assertion
    Const,
    /// `expression!` non-null assertion
    NonNull,
}

/// Result of checking a type assertion
#[derive(Debug, Clone)]
pub struct AssertionCheckResult {
    /// The resulting type after assertion
    pub result_type: TypeId,
    /// Kind of assertion used
    pub kind: AssertionKind,
    /// Whether the assertion is valid
    pub is_valid: bool,
    /// Any errors encountered
    pub errors: Vec<AssertionError>,
    /// The AST node of the expression
    pub expression_node: NodeId,
}

impl AssertionCheckResult {
    pub fn success(result_type: TypeId, kind: AssertionKind, node: NodeId) -> Self {
        AssertionCheckResult {
            result_type,
            kind,
            is_valid: true,
            errors: Vec::new(),
            expression_node: node,
        }
    }

    pub fn failure(
        result_type: TypeId,
        kind: AssertionKind,
        node: NodeId,
        errors: Vec<AssertionError>,
    ) -> Self {
        AssertionCheckResult {
            result_type,
            kind,
            is_valid: false,
            errors,
            expression_node: node,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.is_valid && self.errors.is_empty()
    }
}

/// Context for assertion checking
#[derive(Debug, Clone, Copy, Default)]
pub struct AssertionContext {
    /// Whether strict null checks are enabled
    pub strict_null_checks: bool,
    /// Whether in a TSX file (affects angle bracket assertions)
    pub is_tsx: bool,
    /// Whether to warn about redundant assertions
    pub warn_redundant: bool,
}

/// Type assertion checker
pub struct AssertionChecker {
    /// Context for checking
    context: AssertionContext,
    /// Collected errors
    errors: Vec<AssertionError>,
}

impl AssertionChecker {
    pub fn new() -> Self {
        AssertionChecker {
            context: AssertionContext::default(),
            errors: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: AssertionContext) -> Self {
        self.context = context;
        self
    }

    /// Check a type assertion: `expression as TargetType`
    pub fn check_as_assertion(
        &mut self,
        source_type: TypeId,
        target_type: TypeId,
        expression_node: NodeId,
        span: Span,
    ) -> AssertionCheckResult {
        self.check_assertion_impl(source_type, target_type, expression_node, span, AssertionKind::As)
    }

    /// Check an angle bracket assertion: `<TargetType>expression`
    pub fn check_angle_bracket_assertion(
        &mut self,
        source_type: TypeId,
        target_type: TypeId,
        expression_node: NodeId,
        span: Span,
    ) -> AssertionCheckResult {
        // Check if in TSX file - angle bracket assertions not allowed
        if self.context.is_tsx {
            let error = AssertionError::new(
                AssertionErrorCode::JsxAssertionNotAllowed,
                span,
                source_type,
                target_type,
            );
            self.errors.push(error.clone());

            return AssertionCheckResult::failure(
                target_type,
                AssertionKind::AngleBracket,
                expression_node,
                vec![error],
            );
        }

        self.check_assertion_impl(
            source_type,
            target_type,
            expression_node,
            span,
            AssertionKind::AngleBracket,
        )
    }

    /// Check a const assertion: `expression as const`
    pub fn check_const_assertion(
        &mut self,
        source_type: TypeId,
        source_flags: TypeFlags,
        expression_node: NodeId,
        span: Span,
    ) -> AssertionCheckResult {
        // const assertion is only valid for literal expressions
        if !self.is_const_assertable(source_flags) {
            let error = AssertionError::new(
                AssertionErrorCode::ConstAssertionOnNonLiteral,
                span,
                source_type,
                source_type,
            );
            self.errors.push(error.clone());

            return AssertionCheckResult::failure(
                source_type,
                AssertionKind::Const,
                expression_node,
                vec![error],
            );
        }

        // const assertion preserves the literal type (makes it readonly and literal)
        // The result type is the source type but with literal flags
        AssertionCheckResult::success(source_type, AssertionKind::Const, expression_node)
    }

    /// Check a non-null assertion: `expression!`
    pub fn check_non_null_assertion(
        &mut self,
        source_type: TypeId,
        source_flags: TypeFlags,
        expression_node: NodeId,
        span: Span,
    ) -> AssertionCheckResult {
        // Check if source type includes null or undefined
        let has_null = source_flags.contains(TypeFlags::NULL) || source_type == TypeId::NULL;
        let has_undefined = source_flags.contains(TypeFlags::UNDEFINED) || source_type == TypeId::UNDEFINED;

        // If strict null checks are enabled and type doesn't include null/undefined,
        // the non-null assertion is redundant
        if self.context.strict_null_checks && !has_null && !has_undefined {
            if self.context.warn_redundant {
                let error = AssertionError::new(
                    AssertionErrorCode::RedundantAssertion,
                    span,
                    source_type,
                    source_type,
                );
                self.errors.push(error);
            }
        }

        // Calculate result type (source with null/undefined removed)
        let result_type = self.remove_null_undefined(source_type, source_flags);

        AssertionCheckResult::success(result_type, AssertionKind::NonNull, expression_node)
    }

    /// Internal implementation for as/angle-bracket assertions
    fn check_assertion_impl(
        &mut self,
        source_type: TypeId,
        target_type: TypeId,
        expression_node: NodeId,
        span: Span,
        kind: AssertionKind,
    ) -> AssertionCheckResult {
        // Check for redundant assertion
        if source_type == target_type {
            if self.context.warn_redundant {
                let error = AssertionError::new(
                    AssertionErrorCode::RedundantAssertion,
                    span,
                    source_type,
                    target_type,
                );
                self.errors.push(error);
            }
            return AssertionCheckResult::success(target_type, kind, expression_node);
        }

        // Check assertion compatibility
        if self.is_assertion_valid(source_type, target_type) {
            AssertionCheckResult::success(target_type, kind, expression_node)
        } else {
            let error = AssertionError::with_message(
                AssertionErrorCode::ConversionMayBeMistake,
                span,
                source_type,
                target_type,
                format!(
                    "Conversion of type '{}' to type '{}' may be a mistake because neither type sufficiently overlaps with the other. If this was intentional, convert the expression to 'unknown' first.",
                    self.type_name(source_type),
                    self.type_name(target_type)
                ),
            );
            self.errors.push(error.clone());

            // Still return target type - assertion is applied but with error
            AssertionCheckResult::failure(
                target_type,
                kind,
                expression_node,
                vec![error],
            )
        }
    }

    /// Check if an assertion from source to target is valid
    ///
    /// TypeScript requires that either:
    /// - source is assignable to target, OR
    /// - target is assignable to source, OR
    /// - one of them is any/unknown
    fn is_assertion_valid(&self, source: TypeId, target: TypeId) -> bool {
        // Same type is always valid
        if source == target {
            return true;
        }

        // Assertions to/from any are always valid
        if source == TypeId::ANY || target == TypeId::ANY {
            return true;
        }

        // Assertions to/from unknown are always valid
        if source == TypeId::UNKNOWN || target == TypeId::UNKNOWN {
            return true;
        }

        // never can be asserted to anything
        if source == TypeId::NEVER {
            return true;
        }

        // anything can be asserted to never (though this is usually a mistake)
        if target == TypeId::NEVER {
            return true;
        }

        // Check if types have sufficient overlap
        self.types_have_overlap(source, target)
    }

    /// Check if two types have sufficient overlap for assertion
    fn types_have_overlap(&self, source: TypeId, target: TypeId) -> bool {
        // Both primitives - check if same primitive category
        if source.is_primitive() && target.is_primitive() {
            return self.primitive_overlap(source, target);
        }

        // Object types are considered to potentially overlap
        if source == TypeId::OBJECT || target == TypeId::OBJECT {
            return true;
        }

        // In a full implementation, we'd check structural overlap
        // For now, assume types might overlap if not definitely incompatible
        true
    }

    /// Check if two primitive types have overlap
    fn primitive_overlap(&self, source: TypeId, target: TypeId) -> bool {
        // Same primitive type
        if source == target {
            return true;
        }

        // void and undefined overlap
        if (source == TypeId::VOID && target == TypeId::UNDEFINED)
            || (source == TypeId::UNDEFINED && target == TypeId::VOID)
        {
            return true;
        }

        // null and undefined overlap in non-strict mode
        if !self.context.strict_null_checks {
            if (source == TypeId::NULL || source == TypeId::UNDEFINED)
                && (target == TypeId::NULL || target == TypeId::UNDEFINED)
            {
                return true;
            }
        }

        // Different primitive types don't overlap
        // (e.g., string and number have no overlap)
        false
    }

    /// Check if type flags indicate a const-assertable expression
    fn is_const_assertable(&self, flags: TypeFlags) -> bool {
        // Literals are const-assertable
        if flags.intersects(TypeFlags::LITERAL) {
            return true;
        }

        // Object types are const-assertable
        if flags.contains(TypeFlags::OBJECT) {
            return true;
        }

        // String, number, boolean, bigint literals
        if flags.intersects(
            TypeFlags::STRING_LITERAL
                | TypeFlags::NUMBER_LITERAL
                | TypeFlags::BOOLEAN_LITERAL
                | TypeFlags::BIGINT_LITERAL,
        ) {
            return true;
        }

        false
    }

    /// Remove null and undefined from a type
    fn remove_null_undefined(&self, type_id: TypeId, _flags: TypeFlags) -> TypeId {
        // In a real implementation, we'd create a new type without null/undefined
        // For now, just return the type (assuming it's been narrowed)
        match type_id {
            TypeId::NULL | TypeId::UNDEFINED => TypeId::NEVER,
            _ => type_id,
        }
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
    pub fn take_errors(&mut self) -> Vec<AssertionError> {
        std::mem::take(&mut self.errors)
    }
}

impl Default for AssertionChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Check assertion to unknown - always valid
pub fn assert_to_unknown(source_type: TypeId) -> TypeId {
    // Any type can be asserted to unknown
    let _ = source_type;
    TypeId::UNKNOWN
}

/// Check assertion to any - always valid
pub fn assert_to_any(source_type: TypeId) -> TypeId {
    // Any type can be asserted to any
    let _ = source_type;
    TypeId::ANY
}

/// Check assertion from unknown - always valid
pub fn assert_from_unknown(target_type: TypeId) -> TypeId {
    // unknown can be asserted to any type
    target_type
}

/// Check assertion from any - always valid
pub fn assert_from_any(target_type: TypeId) -> TypeId {
    // any can be asserted to any type
    target_type
}

/// Double assertion pattern: `expr as unknown as TargetType`
///
/// This is the escape hatch for asserting between incompatible types.
pub fn double_assertion(source_type: TypeId, target_type: TypeId) -> (TypeId, TypeId) {
    // First assertion: source -> unknown
    let intermediate = assert_to_unknown(source_type);
    // Second assertion: unknown -> target
    let result = assert_from_unknown(target_type);
    (intermediate, result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_assertion_same_type() {
        let mut checker = AssertionChecker::new();

        let result = checker.check_as_assertion(
            TypeId::STRING,
            TypeId::STRING,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.result_type, TypeId::STRING);
        assert_eq!(result.kind, AssertionKind::As);
    }

    #[test]
    fn test_as_assertion_to_unknown() {
        let mut checker = AssertionChecker::new();

        let result = checker.check_as_assertion(
            TypeId::STRING,
            TypeId::UNKNOWN,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.result_type, TypeId::UNKNOWN);
    }

    #[test]
    fn test_as_assertion_from_unknown() {
        let mut checker = AssertionChecker::new();

        let result = checker.check_as_assertion(
            TypeId::UNKNOWN,
            TypeId::NUMBER,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.result_type, TypeId::NUMBER);
    }

    #[test]
    fn test_as_assertion_to_any() {
        let mut checker = AssertionChecker::new();

        let result = checker.check_as_assertion(
            TypeId::STRING,
            TypeId::ANY,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.result_type, TypeId::ANY);
    }

    #[test]
    fn test_as_assertion_incompatible() {
        let mut checker = AssertionChecker::new();

        // string to number - incompatible primitives
        let result = checker.check_as_assertion(
            TypeId::STRING,
            TypeId::NUMBER,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(!result.is_ok());
        assert!(!result.errors.is_empty());
        assert_eq!(result.errors[0].code, AssertionErrorCode::ConversionMayBeMistake);
    }

    #[test]
    fn test_angle_bracket_in_tsx() {
        let mut checker = AssertionChecker::new()
            .with_context(AssertionContext {
                is_tsx: true,
                ..Default::default()
            });

        let result = checker.check_angle_bracket_assertion(
            TypeId::STRING,
            TypeId::STRING,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(!result.is_ok());
        assert_eq!(result.errors[0].code, AssertionErrorCode::JsxAssertionNotAllowed);
    }

    #[test]
    fn test_angle_bracket_valid() {
        let mut checker = AssertionChecker::new()
            .with_context(AssertionContext {
                is_tsx: false,
                ..Default::default()
            });

        let result = checker.check_angle_bracket_assertion(
            TypeId::ANY,
            TypeId::STRING,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.kind, AssertionKind::AngleBracket);
    }

    #[test]
    fn test_const_assertion_valid() {
        let mut checker = AssertionChecker::new();

        let result = checker.check_const_assertion(
            TypeId::new(100), // Some literal type
            TypeFlags::STRING_LITERAL,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.kind, AssertionKind::Const);
    }

    #[test]
    fn test_const_assertion_on_non_literal() {
        let mut checker = AssertionChecker::new();

        let result = checker.check_const_assertion(
            TypeId::STRING,
            TypeFlags::STRING, // Not a literal flag
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(!result.is_ok());
        assert_eq!(result.errors[0].code, AssertionErrorCode::ConstAssertionOnNonLiteral);
    }

    #[test]
    fn test_non_null_assertion() {
        let mut checker = AssertionChecker::new()
            .with_context(AssertionContext {
                strict_null_checks: true,
                ..Default::default()
            });

        let result = checker.check_non_null_assertion(
            TypeId::new(100),
            TypeFlags::NULL | TypeFlags::STRING,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.kind, AssertionKind::NonNull);
    }

    #[test]
    fn test_non_null_removes_null() {
        let mut checker = AssertionChecker::new();

        // null! should result in never
        let result = checker.check_non_null_assertion(
            TypeId::NULL,
            TypeFlags::NULL,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
        assert_eq!(result.result_type, TypeId::NEVER);
    }

    #[test]
    fn test_double_assertion() {
        let (intermediate, result) = double_assertion(TypeId::STRING, TypeId::NUMBER);

        assert_eq!(intermediate, TypeId::UNKNOWN);
        assert_eq!(result, TypeId::NUMBER);
    }

    #[test]
    fn test_assert_to_unknown() {
        let result = assert_to_unknown(TypeId::STRING);
        assert_eq!(result, TypeId::UNKNOWN);
    }

    #[test]
    fn test_assert_to_any() {
        let result = assert_to_any(TypeId::STRING);
        assert_eq!(result, TypeId::ANY);
    }

    #[test]
    fn test_never_assertion() {
        let mut checker = AssertionChecker::new();

        // never can be asserted to anything
        let result = checker.check_as_assertion(
            TypeId::NEVER,
            TypeId::STRING,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_void_undefined_overlap() {
        let mut checker = AssertionChecker::new();

        // void and undefined have overlap
        let result = checker.check_as_assertion(
            TypeId::VOID,
            TypeId::UNDEFINED,
            NodeId::new(0),
            Span::new(0, 10),
        );

        assert!(result.is_ok());
    }
}
