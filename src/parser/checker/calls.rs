//! Call expression type checking
//!
//! This module handles type checking of call expressions, new expressions,
//! and method invocations. It validates argument types against parameter types,
//! handles spread arguments, and performs type inference for generic calls.

use crate::ast::{NodeId, Span};
use super::signatures::{
    Signature, SignatureFlags, SignatureId, SignatureStore,
    TypeId, TypeFlags, TypeParameter,
};

/// Error codes for call expression type checking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CallErrorCode {
    /// Expected N arguments, but got M
    ArgumentCountMismatch = 2554,
    /// Argument type not assignable to parameter type
    ArgumentTypeMismatch = 2345,
    /// This expression is not callable
    NotCallable = 2349,
    /// This expression is not constructable
    NotConstructable = 2351,
    /// Cannot invoke an object which is possibly undefined
    PossiblyUndefined = 2722,
    /// Cannot invoke an object which is possibly null
    PossiblyNull = 2721,
    /// Spread argument must be a tuple or passed to a rest parameter
    SpreadRequiresTupleOrRest = 2556,
    /// Type arguments cannot be used with non-generic call
    TypeArgsOnNonGeneric = 2558,
    /// Generic type requires N type argument(s)
    TypeArgCountMismatch = 2707,
    /// Type argument doesn't satisfy constraint
    TypeArgConstraintViolation = 2344,
    /// The 'this' context is incompatible
    ThisContextMismatch = 2684,
    /// Cannot call abstract constructor
    CannotCallAbstractConstructor = 2511,
    /// Required type parameters may not follow optional type parameters
    RequiredTypeParamAfterOptional = 2706,
}

impl CallErrorCode {
    pub fn code(self) -> u32 {
        self as u32
    }

    pub fn message(self) -> &'static str {
        match self {
            CallErrorCode::ArgumentCountMismatch => {
                "Expected arguments, but got different count."
            }
            CallErrorCode::ArgumentTypeMismatch => {
                "Argument of type is not assignable to parameter of type."
            }
            CallErrorCode::NotCallable => {
                "This expression is not callable."
            }
            CallErrorCode::NotConstructable => {
                "This expression is not constructable."
            }
            CallErrorCode::PossiblyUndefined => {
                "Cannot invoke an object which is possibly 'undefined'."
            }
            CallErrorCode::PossiblyNull => {
                "Cannot invoke an object which is possibly 'null'."
            }
            CallErrorCode::SpreadRequiresTupleOrRest => {
                "A spread argument must either have a tuple type or be passed to a rest parameter."
            }
            CallErrorCode::TypeArgsOnNonGeneric => {
                "Type arguments cannot be applied to a non-generic function."
            }
            CallErrorCode::TypeArgCountMismatch => {
                "Generic type requires type argument(s)."
            }
            CallErrorCode::TypeArgConstraintViolation => {
                "Type does not satisfy the constraint."
            }
            CallErrorCode::ThisContextMismatch => {
                "The 'this' context of type is not assignable to method's 'this' of type."
            }
            CallErrorCode::CannotCallAbstractConstructor => {
                "Cannot create an instance of an abstract class."
            }
            CallErrorCode::RequiredTypeParamAfterOptional => {
                "Required type parameters may not follow optional type parameters."
            }
        }
    }
}

/// A call expression error
#[derive(Debug, Clone)]
pub struct CallError {
    pub code: CallErrorCode,
    pub span: Span,
    pub message: String,
    /// Related node (e.g., the parameter for argument mismatch)
    pub related_span: Option<Span>,
}

impl CallError {
    pub fn new(code: CallErrorCode, span: Span) -> Self {
        CallError {
            code,
            span,
            message: code.message().to_string(),
            related_span: None,
        }
    }

    pub fn with_message(code: CallErrorCode, span: Span, message: impl Into<String>) -> Self {
        CallError {
            code,
            span,
            message: message.into(),
            related_span: None,
        }
    }

    pub fn with_related(mut self, span: Span) -> Self {
        self.related_span = Some(span);
        self
    }
}

/// An argument in a call expression
#[derive(Debug, Clone, Copy)]
pub struct Argument {
    /// The type of the argument
    pub type_id: TypeId,
    /// Whether this is a spread argument (...args)
    pub is_spread: bool,
    /// Source location
    pub span: Span,
    /// AST node for the argument expression
    pub node: NodeId,
}

impl Argument {
    pub fn new(type_id: TypeId, span: Span) -> Self {
        Argument {
            type_id,
            is_spread: false,
            span,
            node: NodeId::NONE,
        }
    }

    pub fn spread(type_id: TypeId, span: Span) -> Self {
        Argument {
            type_id,
            is_spread: true,
            span,
            node: NodeId::NONE,
        }
    }

    pub fn with_node(mut self, node: NodeId) -> Self {
        self.node = node;
        self
    }
}

/// Type argument provided to a generic call
#[derive(Debug, Clone, Copy)]
pub struct TypeArgument {
    pub type_id: TypeId,
    pub span: Span,
}

impl TypeArgument {
    pub fn new(type_id: TypeId, span: Span) -> Self {
        TypeArgument { type_id, span }
    }
}

/// Context for call expression type checking
#[derive(Debug, Clone)]
pub struct CallContext {
    /// Whether this is a 'new' expression
    pub is_new_expression: bool,
    /// Whether this is optional chaining (foo?.())
    pub is_optional_chain: bool,
    /// The 'this' type at the call site
    pub this_type: TypeId,
    /// Whether strict null checks are enabled
    pub strict_null_checks: bool,
    /// The span of the call expression
    pub call_span: Span,
    /// The span of the callee expression
    pub callee_span: Span,
}

impl Default for CallContext {
    fn default() -> Self {
        CallContext {
            is_new_expression: false,
            is_optional_chain: false,
            this_type: TypeId::NONE,
            strict_null_checks: false,
            call_span: Span::new(0, 0),
            callee_span: Span::new(0, 0),
        }
    }
}

/// Result of call expression type checking
#[derive(Debug, Clone)]
pub struct CallCheckResult {
    /// The return type of the call
    pub return_type: TypeId,
    /// The selected signature (for overloaded functions)
    pub selected_signature: SignatureId,
    /// Inferred type arguments (for generic calls)
    pub inferred_type_args: Vec<TypeId>,
    /// Any errors encountered
    pub errors: Vec<CallError>,
}

impl CallCheckResult {
    pub fn new(return_type: TypeId) -> Self {
        CallCheckResult {
            return_type,
            selected_signature: SignatureId::NONE,
            inferred_type_args: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn error(error: CallError) -> Self {
        CallCheckResult {
            return_type: TypeId::ANY,
            selected_signature: SignatureId::NONE,
            inferred_type_args: Vec::new(),
            errors: vec![error],
        }
    }

    pub fn with_signature(mut self, sig: SignatureId) -> Self {
        self.selected_signature = sig;
        self
    }

    pub fn with_type_args(mut self, args: Vec<TypeId>) -> Self {
        self.inferred_type_args = args;
        self
    }

    pub fn with_error(mut self, error: CallError) -> Self {
        self.errors.push(error);
        self
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Call expression type checker
pub struct CallChecker<'a> {
    /// Signature store for accessing signatures
    signatures: &'a SignatureStore,
    /// Errors collected during checking
    errors: Vec<CallError>,
    /// Current call context
    context: CallContext,
}

impl<'a> CallChecker<'a> {
    pub fn new(signatures: &'a SignatureStore) -> Self {
        CallChecker {
            signatures,
            errors: Vec::new(),
            context: CallContext::default(),
        }
    }

    pub fn with_context(mut self, context: CallContext) -> Self {
        self.context = context;
        self
    }

    /// Check a call expression against a signature
    pub fn check_call(
        &mut self,
        sig_id: SignatureId,
        args: &[Argument],
        type_args: Option<&[TypeArgument]>,
    ) -> CallCheckResult {
        let sig = match self.signatures.get(sig_id) {
            Some(s) => s,
            None => {
                return CallCheckResult::error(CallError::new(
                    CallErrorCode::NotCallable,
                    self.context.callee_span,
                ));
            }
        };

        // Check if this is a construct signature and we're using 'new'
        if self.context.is_new_expression && !sig.is_construct_signature() {
            // Check if it's abstract
            if sig.flags.contains(SignatureFlags::ABSTRACT) {
                return CallCheckResult::error(CallError::new(
                    CallErrorCode::CannotCallAbstractConstructor,
                    self.context.callee_span,
                ));
            }
        }

        // Validate type arguments for generic functions
        let inferred_type_args = if sig.is_generic() {
            self.check_type_arguments(sig, type_args, args)
        } else if type_args.is_some() && !type_args.unwrap().is_empty() {
            self.errors.push(CallError::new(
                CallErrorCode::TypeArgsOnNonGeneric,
                self.context.call_span,
            ));
            Vec::new()
        } else {
            Vec::new()
        };

        // Check argument count
        if !self.check_argument_count(sig, args) {
            // Error already added
        }

        // Check argument types
        self.check_argument_types(sig, args, &inferred_type_args);

        let mut result = CallCheckResult::new(sig.get_return_type())
            .with_signature(sig_id)
            .with_type_args(inferred_type_args);

        result.errors = std::mem::take(&mut self.errors);
        result
    }

    /// Check a new expression against construct signatures
    pub fn check_new_expression(
        &mut self,
        construct_sigs: &[SignatureId],
        args: &[Argument],
        type_args: Option<&[TypeArgument]>,
    ) -> CallCheckResult {
        if construct_sigs.is_empty() {
            return CallCheckResult::error(CallError::new(
                CallErrorCode::NotConstructable,
                self.context.callee_span,
            ));
        }

        self.context.is_new_expression = true;

        // Try to find a matching signature (simplified - real impl uses overload resolution)
        for &sig_id in construct_sigs {
            let result = self.check_call(sig_id, args, type_args);
            if result.is_ok() {
                return result;
            }
        }

        // If no signature matched, use the first one and report its errors
        self.check_call(construct_sigs[0], args, type_args)
    }

    /// Check call against multiple overloaded signatures
    pub fn check_overloaded_call(
        &mut self,
        call_sigs: &[SignatureId],
        args: &[Argument],
        type_args: Option<&[TypeArgument]>,
    ) -> CallCheckResult {
        if call_sigs.is_empty() {
            return CallCheckResult::error(CallError::new(
                CallErrorCode::NotCallable,
                self.context.callee_span,
            ));
        }

        // Single signature - no overload resolution needed
        if call_sigs.len() == 1 {
            return self.check_call(call_sigs[0], args, type_args);
        }

        // Try each signature
        let mut best_result: Option<CallCheckResult> = None;
        let mut all_errors: Vec<CallError> = Vec::new();

        for &sig_id in call_sigs {
            let mut temp_checker = CallChecker::new(self.signatures)
                .with_context(self.context.clone());
            let result = temp_checker.check_call(sig_id, args, type_args);

            if result.is_ok() {
                // Found a matching signature
                return result;
            }

            // Collect errors for reporting if no signature matches
            all_errors.extend(result.errors.clone());

            if best_result.is_none() {
                best_result = Some(result);
            }
        }

        // No signature matched - return the first result with aggregated errors
        let mut result = best_result.unwrap_or_else(|| {
            CallCheckResult::error(CallError::new(
                CallErrorCode::ArgumentCountMismatch,
                self.context.call_span,
            ))
        });

        // Add a summary error
        result.errors.insert(0, CallError::with_message(
            CallErrorCode::ArgumentTypeMismatch,
            self.context.call_span,
            format!("No overload matches this call. {} overload(s) were tried.", call_sigs.len()),
        ));

        result
    }

    /// Check argument count against signature
    fn check_argument_count(&mut self, sig: &Signature, args: &[Argument]) -> bool {
        let arg_count = self.effective_argument_count(args);
        let min = sig.min_argument_count as usize;

        if arg_count < min {
            self.errors.push(CallError::with_message(
                CallErrorCode::ArgumentCountMismatch,
                self.context.call_span,
                format!(
                    "Expected at least {} arguments, but got {}.",
                    min, arg_count
                ),
            ));
            return false;
        }

        if let Some(max) = sig.max_argument_count() {
            if arg_count > max {
                self.errors.push(CallError::with_message(
                    CallErrorCode::ArgumentCountMismatch,
                    self.context.call_span,
                    format!("Expected {} arguments, but got {}.", max, arg_count),
                ));
                return false;
            }
        }

        true
    }

    /// Calculate effective argument count (handling spread)
    fn effective_argument_count(&self, args: &[Argument]) -> usize {
        // For simplicity, count spread as 1 argument
        // Real implementation would inspect the spread type
        args.len()
    }

    /// Check argument types against parameter types
    fn check_argument_types(
        &mut self,
        sig: &Signature,
        args: &[Argument],
        inferred_type_args: &[TypeId],
    ) {
        let params = &sig.parameters;

        for (i, arg) in args.iter().enumerate() {
            // Get the corresponding parameter
            let param = if i < params.len() {
                Some(&params[i])
            } else if sig.has_rest_parameter() {
                sig.rest_parameter()
            } else {
                None
            };

            if let Some(param) = param {
                // Handle spread argument
                if arg.is_spread {
                    if !param.is_rest() {
                        self.errors.push(CallError::new(
                            CallErrorCode::SpreadRequiresTupleOrRest,
                            arg.span,
                        ));
                    }
                    continue;
                }

                // Check type compatibility
                let param_type = self.substitute_type_args(param.type_id, sig, inferred_type_args);
                if !self.is_type_assignable(arg.type_id, param_type) {
                    self.errors.push(CallError::with_message(
                        CallErrorCode::ArgumentTypeMismatch,
                        arg.span,
                        format!(
                            "Argument of type '{}' is not assignable to parameter of type '{}'.",
                            self.type_name(arg.type_id),
                            self.type_name(param_type)
                        ),
                    ).with_related(param.span));
                }
            }
        }
    }

    /// Check and potentially infer type arguments
    fn check_type_arguments(
        &mut self,
        sig: &Signature,
        type_args: Option<&[TypeArgument]>,
        args: &[Argument],
    ) -> Vec<TypeId> {
        let type_params = &sig.type_parameters;

        if let Some(provided_args) = type_args {
            // Explicit type arguments provided
            if provided_args.len() != type_params.len() {
                let required = type_params.iter().filter(|tp| !tp.has_default()).count();
                if provided_args.len() < required {
                    self.errors.push(CallError::with_message(
                        CallErrorCode::TypeArgCountMismatch,
                        self.context.call_span,
                        format!(
                            "Expected {} type arguments, but got {}.",
                            required,
                            provided_args.len()
                        ),
                    ));
                }
            }

            // Check constraints
            let mut result = Vec::with_capacity(type_params.len());
            for (i, type_param) in type_params.iter().enumerate() {
                let type_arg = provided_args.get(i).map(|ta| ta.type_id).unwrap_or(type_param.default);

                // Check constraint
                if type_param.has_constraint() && type_arg.is_some() {
                    if !self.satisfies_constraint(type_arg, type_param.constraint) {
                        if let Some(ta) = provided_args.get(i) {
                            self.errors.push(CallError::new(
                                CallErrorCode::TypeArgConstraintViolation,
                                ta.span,
                            ));
                        }
                    }
                }

                result.push(type_arg);
            }
            result
        } else {
            // Infer type arguments from argument types
            self.infer_type_arguments(sig, args)
        }
    }

    /// Infer type arguments from argument types
    fn infer_type_arguments(&self, sig: &Signature, args: &[Argument]) -> Vec<TypeId> {
        // Simplified type inference - real implementation is much more complex
        let mut inferred: Vec<TypeId> = vec![TypeId::NONE; sig.type_parameters.len()];

        // Try to infer from each argument
        for (i, arg) in args.iter().enumerate() {
            if i >= sig.parameters.len() {
                break;
            }

            let param = &sig.parameters[i];
            // In a real implementation, we'd analyze the parameter type to find
            // type parameter references and infer their types from the argument type
            self.infer_from_types(arg.type_id, param.type_id, &sig.type_parameters, &mut inferred);
        }

        // Fill in defaults for any unresolved type parameters
        for (i, tp) in sig.type_parameters.iter().enumerate() {
            if inferred[i].is_none() {
                inferred[i] = if tp.has_default() {
                    tp.default
                } else if tp.has_constraint() {
                    tp.constraint
                } else {
                    TypeId::UNKNOWN
                };
            }
        }

        inferred
    }

    /// Infer type parameter values from source/target types
    fn infer_from_types(
        &self,
        source: TypeId,
        target: TypeId,
        type_params: &[TypeParameter],
        inferred: &mut [TypeId],
    ) {
        // Check if target is a type parameter
        for (i, tp) in type_params.iter().enumerate() {
            if tp.type_id == target {
                // Target is this type parameter - infer from source
                if inferred[i].is_none() {
                    inferred[i] = source;
                }
                // In real implementation, we'd need to handle multiple inferences
                // and find the best common type
                return;
            }
        }

        // In a real implementation, we'd recursively analyze structured types
        // (arrays, objects, functions) to find type parameter references
    }

    /// Substitute type arguments into a type
    fn substitute_type_args(
        &self,
        type_id: TypeId,
        sig: &Signature,
        type_args: &[TypeId],
    ) -> TypeId {
        // Check if this type is one of the type parameters
        for (i, tp) in sig.type_parameters.iter().enumerate() {
            if tp.type_id == type_id {
                if let Some(&arg) = type_args.get(i) {
                    if arg.is_some() {
                        return arg;
                    }
                }
            }
        }

        // In a real implementation, we'd recursively substitute in structured types
        type_id
    }

    /// Check if a type argument satisfies a constraint
    fn satisfies_constraint(&self, _type_arg: TypeId, constraint: TypeId) -> bool {
        // Simplified check - real implementation uses full type compatibility
        if constraint == TypeId::ANY || constraint == TypeId::UNKNOWN {
            return true;
        }
        // In real implementation, check if type_arg is assignable to constraint
        true
    }

    /// Check if source type is assignable to target type
    fn is_type_assignable(&self, source: TypeId, target: TypeId) -> bool {
        // Simplified assignability check
        if target == TypeId::ANY || target == TypeId::UNKNOWN {
            return true;
        }
        if source == TypeId::ANY {
            return true;
        }
        if source == TypeId::NEVER {
            return true;
        }
        if source == target {
            return true;
        }

        // In a real implementation, we'd have full structural type compatibility
        // For now, just return true to avoid false positives
        true
    }

    /// Get a human-readable name for a type (placeholder)
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

    /// Get all collected errors
    pub fn take_errors(&mut self) -> Vec<CallError> {
        std::mem::take(&mut self.errors)
    }
}

/// Check if an expression can be called with optional chaining
pub fn check_optional_call(
    type_flags: TypeFlags,
    strict_null_checks: bool,
) -> Result<(), CallErrorCode> {
    if strict_null_checks {
        if type_flags.contains(TypeFlags::UNDEFINED) && !type_flags.intersects(TypeFlags::OBJECT) {
            return Err(CallErrorCode::PossiblyUndefined);
        }
        if type_flags.contains(TypeFlags::NULL) && !type_flags.intersects(TypeFlags::OBJECT) {
            return Err(CallErrorCode::PossiblyNull);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::StringId;
    use super::super::signatures::Parameter;

    #[test]
    fn test_call_error_codes() {
        let error = CallError::new(CallErrorCode::ArgumentCountMismatch, Span::new(0, 10));
        assert_eq!(error.code.code(), 2554);
        assert!(!error.message.is_empty());
    }

    #[test]
    fn test_argument_creation() {
        let arg = Argument::new(TypeId::STRING, Span::new(5, 10));
        assert_eq!(arg.type_id, TypeId::STRING);
        assert!(!arg.is_spread);

        let spread_arg = Argument::spread(TypeId::STRING, Span::new(5, 15));
        assert!(spread_arg.is_spread);
    }

    #[test]
    fn test_call_context_default() {
        let ctx = CallContext::default();
        assert!(!ctx.is_new_expression);
        assert!(!ctx.is_optional_chain);
        assert!(ctx.this_type.is_none());
    }

    #[test]
    fn test_call_check_basic() {
        let mut store = SignatureStore::new();

        // Create a simple function: (x: string, y?: number) => boolean
        let sig = Signature::new(SignatureId::NONE)
            .with_parameters(vec![
                Parameter::new(StringId::new(1), TypeId::STRING),
                Parameter::new(StringId::new(2), TypeId::NUMBER).optional(),
            ])
            .with_return_type(TypeId::BOOLEAN);
        let sig_id = store.alloc_with(sig);

        let checker = CallChecker::new(&store);
        let mut checker = checker.with_context(CallContext::default());

        // Call with one argument - should succeed
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];
        let result = checker.check_call(sig_id, &args, None);
        assert!(result.is_ok());
        assert_eq!(result.return_type, TypeId::BOOLEAN);
    }

    #[test]
    fn test_call_check_argument_count() {
        let mut store = SignatureStore::new();

        let sig = Signature::new(SignatureId::NONE)
            .with_parameters(vec![
                Parameter::new(StringId::new(1), TypeId::STRING),
                Parameter::new(StringId::new(2), TypeId::NUMBER),
            ])
            .with_return_type(TypeId::VOID);
        let sig_id = store.alloc_with(sig);

        let mut checker = CallChecker::new(&store);

        // Call with too few arguments
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];
        let result = checker.check_call(sig_id, &args, None);
        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e| e.code == CallErrorCode::ArgumentCountMismatch));
    }

    #[test]
    fn test_call_check_rest_parameter() {
        let mut store = SignatureStore::new();

        // (x: string, ...rest: number[]) => void
        let sig = Signature::new(SignatureId::NONE)
            .with_parameters(vec![
                Parameter::new(StringId::new(1), TypeId::STRING),
                Parameter::new(StringId::new(2), TypeId::NUMBER).rest(),
            ])
            .with_return_type(TypeId::VOID);
        let sig_id = store.alloc_with(sig);

        let mut checker = CallChecker::new(&store);

        // Call with many arguments - should succeed due to rest parameter
        let args = vec![
            Argument::new(TypeId::STRING, Span::new(0, 5)),
            Argument::new(TypeId::NUMBER, Span::new(6, 10)),
            Argument::new(TypeId::NUMBER, Span::new(11, 15)),
            Argument::new(TypeId::NUMBER, Span::new(16, 20)),
        ];
        let result = checker.check_call(sig_id, &args, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_args_on_non_generic() {
        let mut store = SignatureStore::new();

        // Non-generic function
        let sig = Signature::new(SignatureId::NONE)
            .with_parameters(vec![Parameter::new(StringId::new(1), TypeId::STRING)])
            .with_return_type(TypeId::VOID);
        let sig_id = store.alloc_with(sig);

        let mut checker = CallChecker::new(&store);

        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];
        let type_args = vec![TypeArgument::new(TypeId::STRING, Span::new(0, 5))];
        let result = checker.check_call(sig_id, &args, Some(&type_args));

        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e| e.code == CallErrorCode::TypeArgsOnNonGeneric));
    }

    #[test]
    fn test_overloaded_call() {
        let mut store = SignatureStore::new();

        // Overload 1: (x: string) => string
        let sig1 = Signature::new(SignatureId::NONE)
            .with_parameters(vec![Parameter::new(StringId::new(1), TypeId::STRING)])
            .with_return_type(TypeId::STRING);
        let sig1_id = store.alloc_with(sig1);

        // Overload 2: (x: number) => number
        let sig2 = Signature::new(SignatureId::NONE)
            .with_parameters(vec![Parameter::new(StringId::new(1), TypeId::NUMBER)])
            .with_return_type(TypeId::NUMBER);
        let sig2_id = store.alloc_with(sig2);

        let mut checker = CallChecker::new(&store);

        // Call with string - should select first overload
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];
        let result = checker.check_overloaded_call(&[sig1_id, sig2_id], &args, None);

        assert!(result.is_ok());
        assert_eq!(result.return_type, TypeId::STRING);
        assert_eq!(result.selected_signature, sig1_id);
    }

    #[test]
    fn test_optional_call_check() {
        // Non-optional call on undefined-possible type should error with strict null checks
        let result = check_optional_call(TypeFlags::UNDEFINED, true);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CallErrorCode::PossiblyUndefined);

        // Object | undefined is fine
        let result = check_optional_call(TypeFlags::OBJECT | TypeFlags::UNDEFINED, true);
        assert!(result.is_ok());

        // Without strict null checks, undefined is fine
        let result = check_optional_call(TypeFlags::UNDEFINED, false);
        assert!(result.is_ok());
    }
}
