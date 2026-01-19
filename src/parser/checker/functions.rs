//! Function type checking
//!
//! This module handles type checking of function declarations, function expressions,
//! arrow functions, and method declarations. It validates parameter types, return types,
//! and contextual typing for function bodies.

use crate::ast::{NodeId, StringId, Span};
use super::signatures::{
    Parameter, ParameterFlags, Signature, SignatureFlags, SignatureId, SignatureStore,
    TypeId, TypeFlags, TypeParameter,
};

/// Error codes for function-related type checking errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FunctionErrorCode {
    /// Function lacks ending return statement
    NotAllCodePathsReturn = 2366,
    /// Parameter cannot have question mark and initializer
    ParamOptionalWithInitializer = 1015,
    /// A required parameter cannot follow an optional parameter
    RequiredAfterOptional = 1016,
    /// A rest parameter must be last
    RestParameterMustBeLast = 1014,
    /// A rest parameter cannot have an initializer
    RestCannotHaveInitializer = 1017,
    /// Duplicate parameter name
    DuplicateParameter = 2300,
    /// Parameter implicitly has 'any' type
    ImplicitAnyParameter = 7006,
    /// Function return type implicitly is 'any'
    ImplicitAnyReturn = 7010,
    /// Type mismatch in return statement
    ReturnTypeMismatch = 2322,
    /// Void function has return with value
    VoidFunctionReturnsValue = 2355,
    /// Non-void function lacks return
    NonVoidFunctionNoReturn = 2378,
    /// Generator must have return type iterable
    GeneratorReturnNotIterable = 2504,
    /// Async function must have valid return
    AsyncReturnNotPromise = 1055,
    /// This context is not callable
    NotCallable = 2349,
    /// This context is not constructable
    NotConstructable = 2351,
    /// Abstract method cannot have implementation
    AbstractMethodWithBody = 1245,
    /// Non-abstract class contains abstract method
    AbstractInNonAbstractClass = 2515,
    /// Constructor cannot have type parameters
    ConstructorNoTypeParams = 1092,
    /// Constructor cannot have return type
    ConstructorNoReturnType = 1093,
}

impl FunctionErrorCode {
    pub fn code(self) -> u32 {
        self as u32
    }

    pub fn message(self) -> &'static str {
        match self {
            FunctionErrorCode::NotAllCodePathsReturn => {
                "Not all code paths return a value."
            }
            FunctionErrorCode::ParamOptionalWithInitializer => {
                "Parameter cannot have question mark and initializer."
            }
            FunctionErrorCode::RequiredAfterOptional => {
                "A required parameter cannot follow an optional parameter."
            }
            FunctionErrorCode::RestParameterMustBeLast => {
                "A rest parameter must be last in a parameter list."
            }
            FunctionErrorCode::RestCannotHaveInitializer => {
                "A rest parameter cannot have an initializer."
            }
            FunctionErrorCode::DuplicateParameter => {
                "Duplicate identifier."
            }
            FunctionErrorCode::ImplicitAnyParameter => {
                "Parameter implicitly has an 'any' type."
            }
            FunctionErrorCode::ImplicitAnyReturn => {
                "Function return type implicitly is 'any'."
            }
            FunctionErrorCode::ReturnTypeMismatch => {
                "Type is not assignable to the function return type."
            }
            FunctionErrorCode::VoidFunctionReturnsValue => {
                "A void function cannot return a value."
            }
            FunctionErrorCode::NonVoidFunctionNoReturn => {
                "A function whose declared type is not 'void' must return a value."
            }
            FunctionErrorCode::GeneratorReturnNotIterable => {
                "Generator function must have a return type that is iterable."
            }
            FunctionErrorCode::AsyncReturnNotPromise => {
                "The return type of an async function must be a Promise."
            }
            FunctionErrorCode::NotCallable => {
                "This expression is not callable."
            }
            FunctionErrorCode::NotConstructable => {
                "This expression is not constructable."
            }
            FunctionErrorCode::AbstractMethodWithBody => {
                "Abstract method cannot have an implementation."
            }
            FunctionErrorCode::AbstractInNonAbstractClass => {
                "Abstract method can only appear within an abstract class."
            }
            FunctionErrorCode::ConstructorNoTypeParams => {
                "Type parameters cannot appear on a constructor declaration."
            }
            FunctionErrorCode::ConstructorNoReturnType => {
                "Type annotation cannot appear on a constructor declaration."
            }
        }
    }
}

/// A function type checking error
#[derive(Debug, Clone)]
pub struct FunctionError {
    pub code: FunctionErrorCode,
    pub span: Span,
    pub message: String,
}

impl FunctionError {
    pub fn new(code: FunctionErrorCode, span: Span) -> Self {
        FunctionError {
            code,
            span,
            message: code.message().to_string(),
        }
    }

    pub fn with_message(code: FunctionErrorCode, span: Span, message: impl Into<String>) -> Self {
        FunctionError {
            code,
            span,
            message: message.into(),
        }
    }
}

/// Context for function type checking
#[derive(Debug, Clone, Copy)]
pub struct FunctionContext {
    /// Expected return type from contextual typing
    pub contextual_return_type: TypeId,
    /// Whether this is an async function
    pub is_async: bool,
    /// Whether this is a generator function
    pub is_generator: bool,
    /// Whether this is a constructor
    pub is_constructor: bool,
    /// Whether this is a method
    pub is_method: bool,
    /// Whether this is an arrow function
    pub is_arrow: bool,
    /// Whether noImplicitAny is enabled
    pub no_implicit_any: bool,
    /// Whether noImplicitReturns is enabled
    pub no_implicit_returns: bool,
    /// Whether strict null checks are enabled
    pub strict_null_checks: bool,
    /// The containing class type (for methods)
    pub containing_class: TypeId,
}

impl Default for FunctionContext {
    fn default() -> Self {
        FunctionContext {
            contextual_return_type: TypeId::NONE,
            is_async: false,
            is_generator: false,
            is_constructor: false,
            is_method: false,
            is_arrow: false,
            no_implicit_any: false,
            no_implicit_returns: false,
            strict_null_checks: false,
            containing_class: TypeId::NONE,
        }
    }
}

/// Result of function type checking
#[derive(Debug, Clone)]
pub struct FunctionCheckResult {
    /// The computed signature for the function
    pub signature: SignatureId,
    /// Any errors encountered
    pub errors: Vec<FunctionError>,
    /// Whether all code paths return a value
    pub all_paths_return: bool,
    /// The inferred return type
    pub inferred_return_type: TypeId,
}

impl FunctionCheckResult {
    pub fn new(signature: SignatureId) -> Self {
        FunctionCheckResult {
            signature,
            errors: Vec::new(),
            all_paths_return: true,
            inferred_return_type: TypeId::VOID,
        }
    }

    pub fn with_error(mut self, error: FunctionError) -> Self {
        self.errors.push(error);
        self
    }

    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Function type checker
pub struct FunctionChecker<'a> {
    /// Signature store for allocating signatures
    signatures: &'a mut SignatureStore,
    /// Errors collected during checking
    errors: Vec<FunctionError>,
    /// Current function context
    context: FunctionContext,
}

impl<'a> FunctionChecker<'a> {
    pub fn new(signatures: &'a mut SignatureStore) -> Self {
        FunctionChecker {
            signatures,
            errors: Vec::new(),
            context: FunctionContext::default(),
        }
    }

    pub fn with_context(mut self, context: FunctionContext) -> Self {
        self.context = context;
        self
    }

    /// Check a function declaration
    pub fn check_function_declaration(
        &mut self,
        node: NodeId,
        type_params: &[TypeParameter],
        params: &[Parameter],
        declared_return_type: Option<TypeId>,
    ) -> FunctionCheckResult {
        // Validate parameters
        self.validate_parameters(params);

        // Create signature
        let sig = self.create_signature(node, type_params, params, declared_return_type);

        // Check for implicit any
        if self.context.no_implicit_any {
            self.check_implicit_any(params, declared_return_type);
        }

        let mut result = FunctionCheckResult::new(sig);
        result.errors = std::mem::take(&mut self.errors);

        // Set inferred return type
        result.inferred_return_type = declared_return_type.unwrap_or(TypeId::VOID);

        result
    }

    /// Check a function expression
    pub fn check_function_expression(
        &mut self,
        node: NodeId,
        type_params: &[TypeParameter],
        params: &[Parameter],
        declared_return_type: Option<TypeId>,
    ) -> FunctionCheckResult {
        // Apply contextual typing to parameters if available
        let typed_params = if self.context.contextual_return_type.is_some() {
            self.apply_contextual_typing_to_params(params)
        } else {
            params.to_vec()
        };

        // Validate parameters
        self.validate_parameters(&typed_params);

        // Create signature
        let sig = self.create_signature(node, type_params, &typed_params, declared_return_type);

        let mut result = FunctionCheckResult::new(sig);
        result.errors = std::mem::take(&mut self.errors);

        // Use contextual return type if available
        result.inferred_return_type = declared_return_type
            .or(if self.context.contextual_return_type.is_some() {
                Some(self.context.contextual_return_type)
            } else {
                None
            })
            .unwrap_or(TypeId::VOID);

        result
    }

    /// Check an arrow function
    pub fn check_arrow_function(
        &mut self,
        node: NodeId,
        type_params: &[TypeParameter],
        params: &[Parameter],
        declared_return_type: Option<TypeId>,
        is_concise: bool,
    ) -> FunctionCheckResult {
        self.context.is_arrow = true;

        // Apply contextual typing
        let typed_params = if self.context.contextual_return_type.is_some() {
            self.apply_contextual_typing_to_params(params)
        } else {
            params.to_vec()
        };

        // Validate parameters
        self.validate_parameters(&typed_params);

        // Create signature with async flag if needed
        let mut flags = SignatureFlags::NONE;
        if self.context.is_async {
            flags = flags | SignatureFlags::AWAIT;
        }

        let sig = self.create_signature_with_flags(
            node,
            type_params,
            &typed_params,
            declared_return_type,
            flags,
        );

        let mut result = FunctionCheckResult::new(sig);
        result.errors = std::mem::take(&mut self.errors);

        // Concise arrow functions always return (expression body)
        if is_concise {
            result.all_paths_return = true;
        }

        result
    }

    /// Check a method declaration
    pub fn check_method_declaration(
        &mut self,
        node: NodeId,
        type_params: &[TypeParameter],
        params: &[Parameter],
        declared_return_type: Option<TypeId>,
        is_abstract: bool,
        has_body: bool,
    ) -> FunctionCheckResult {
        self.context.is_method = true;

        // Abstract method cannot have implementation
        if is_abstract && has_body {
            self.errors.push(FunctionError::new(
                FunctionErrorCode::AbstractMethodWithBody,
                Span::new(0, 0), // Would need actual span
            ));
        }

        // Validate parameters
        self.validate_parameters(params);

        let mut flags = SignatureFlags::NONE;
        if is_abstract {
            flags = flags | SignatureFlags::ABSTRACT;
        }

        let sig = self.create_signature_with_flags(
            node,
            type_params,
            params,
            declared_return_type,
            flags,
        );

        let mut result = FunctionCheckResult::new(sig);
        result.errors = std::mem::take(&mut self.errors);
        result
    }

    /// Check a constructor declaration
    pub fn check_constructor(
        &mut self,
        node: NodeId,
        params: &[Parameter],
        has_explicit_return_type: bool,
    ) -> FunctionCheckResult {
        self.context.is_constructor = true;

        // Constructor cannot have return type annotation
        if has_explicit_return_type {
            self.errors.push(FunctionError::new(
                FunctionErrorCode::ConstructorNoReturnType,
                Span::new(0, 0),
            ));
        }

        // Validate parameters
        self.validate_parameters(params);

        // Create signature - constructor returns the containing class type
        let return_type = self.context.containing_class;
        let sig = self.create_signature_with_flags(
            node,
            &[],
            params,
            Some(return_type),
            SignatureFlags::CONSTRUCT_SIGNATURE,
        );

        let mut result = FunctionCheckResult::new(sig);
        result.errors = std::mem::take(&mut self.errors);
        result.inferred_return_type = return_type;
        result
    }

    /// Check a getter accessor
    pub fn check_getter(
        &mut self,
        node: NodeId,
        declared_return_type: Option<TypeId>,
    ) -> FunctionCheckResult {
        // Getter cannot have parameters
        let sig = self.create_signature(node, &[], &[], declared_return_type);

        let mut result = FunctionCheckResult::new(sig);
        result.inferred_return_type = declared_return_type.unwrap_or(TypeId::ANY);
        result
    }

    /// Check a setter accessor
    pub fn check_setter(
        &mut self,
        node: NodeId,
        param: Parameter,
    ) -> FunctionCheckResult {
        // Setter has exactly one parameter and returns void
        self.validate_parameters(&[param.clone()]);

        let sig = self.create_signature(node, &[], &[param], Some(TypeId::VOID));

        let mut result = FunctionCheckResult::new(sig);
        result.errors = std::mem::take(&mut self.errors);
        result.inferred_return_type = TypeId::VOID;
        result
    }

    /// Validate parameter list
    fn validate_parameters(&mut self, params: &[Parameter]) {
        let mut seen_optional = false;
        let mut seen_rest = false;
        let mut names: Vec<StringId> = Vec::new();

        for (_i, param) in params.iter().enumerate() {
            // Check for duplicate parameter names
            if !param.name.is_empty() && names.contains(&param.name) {
                self.errors.push(FunctionError::new(
                    FunctionErrorCode::DuplicateParameter,
                    param.span,
                ));
            }
            names.push(param.name);

            // Rest parameter must be last
            if seen_rest {
                self.errors.push(FunctionError::new(
                    FunctionErrorCode::RestParameterMustBeLast,
                    param.span,
                ));
            }

            if param.is_rest() {
                seen_rest = true;

                // Rest parameter cannot have initializer
                if param.flags.contains(ParameterFlags::HAS_DEFAULT) {
                    self.errors.push(FunctionError::new(
                        FunctionErrorCode::RestCannotHaveInitializer,
                        param.span,
                    ));
                }
            }

            // Required parameter cannot follow optional
            if seen_optional && !param.is_optional() && !param.is_rest() {
                self.errors.push(FunctionError::new(
                    FunctionErrorCode::RequiredAfterOptional,
                    param.span,
                ));
            }

            if param.is_optional() && !param.is_rest() {
                // Check for optional with initializer
                if param.flags.contains(ParameterFlags::OPTIONAL)
                    && param.flags.contains(ParameterFlags::HAS_DEFAULT)
                {
                    self.errors.push(FunctionError::new(
                        FunctionErrorCode::ParamOptionalWithInitializer,
                        param.span,
                    ));
                }
                seen_optional = true;
            }
        }
    }

    /// Check for implicit any in parameters and return type
    fn check_implicit_any(&mut self, params: &[Parameter], return_type: Option<TypeId>) {
        for param in params {
            if param.type_id == TypeId::ANY || param.type_id.is_none() {
                self.errors.push(FunctionError::new(
                    FunctionErrorCode::ImplicitAnyParameter,
                    param.span,
                ));
            }
        }

        if return_type.is_none() {
            self.errors.push(FunctionError::new(
                FunctionErrorCode::ImplicitAnyReturn,
                Span::new(0, 0),
            ));
        }
    }

    /// Apply contextual typing to parameters
    fn apply_contextual_typing_to_params(&self, params: &[Parameter]) -> Vec<Parameter> {
        // In a real implementation, this would infer parameter types from the
        // contextual type. For now, just return a copy.
        params.to_vec()
    }

    /// Create a signature from parameters
    fn create_signature(
        &mut self,
        node: NodeId,
        type_params: &[TypeParameter],
        params: &[Parameter],
        return_type: Option<TypeId>,
    ) -> SignatureId {
        self.create_signature_with_flags(node, type_params, params, return_type, SignatureFlags::NONE)
    }

    /// Create a signature with flags
    fn create_signature_with_flags(
        &mut self,
        node: NodeId,
        type_params: &[TypeParameter],
        params: &[Parameter],
        return_type: Option<TypeId>,
        flags: SignatureFlags,
    ) -> SignatureId {
        // Check for rest parameter
        let has_rest = params.last().map_or(false, |p| p.is_rest());
        let mut final_flags = flags;
        if has_rest {
            final_flags = final_flags | SignatureFlags::HAS_REST_PARAMETER;
        }

        let sig = Signature::new(SignatureId::NONE)
            .with_type_parameters(type_params.to_vec())
            .with_parameters(params.to_vec())
            .with_return_type(return_type.unwrap_or(TypeId::VOID))
            .with_flags(final_flags)
            .with_declaration(node);

        self.signatures.alloc_with(sig)
    }

    /// Check return statement compatibility
    pub fn check_return_statement(
        &mut self,
        return_type: TypeId,
        declared_return: TypeId,
        span: Span,
    ) -> bool {
        // void function returning value
        if declared_return == TypeId::VOID && return_type != TypeId::VOID && return_type != TypeId::UNDEFINED {
            self.errors.push(FunctionError::new(
                FunctionErrorCode::VoidFunctionReturnsValue,
                span,
            ));
            return false;
        }

        // In a real implementation, we'd check type compatibility here
        // For now, just check if types match exactly or if declared is 'any'
        if declared_return == TypeId::ANY || return_type == declared_return {
            return true;
        }

        // Type mismatch
        self.errors.push(FunctionError::new(
            FunctionErrorCode::ReturnTypeMismatch,
            span,
        ));
        false
    }

    /// Get all collected errors
    pub fn take_errors(&mut self) -> Vec<FunctionError> {
        std::mem::take(&mut self.errors)
    }
}

/// Check if a type is callable
pub fn is_callable(type_flags: TypeFlags) -> bool {
    type_flags.contains(TypeFlags::OBJECT) ||
        type_flags.contains(TypeFlags::ANY) ||
        type_flags.contains(TypeFlags::NEVER)
}

/// Check if a type is constructable
pub fn is_constructable(type_flags: TypeFlags) -> bool {
    type_flags.contains(TypeFlags::OBJECT) ||
        type_flags.contains(TypeFlags::ANY)
}

/// Wrap return type for async function
pub fn get_async_return_type(inner_type: TypeId) -> TypeId {
    // In a real implementation, this would wrap the type in Promise<T>
    // For now, just return the inner type as a placeholder
    inner_type
}

/// Wrap return type for generator function
pub fn get_generator_return_type(_yield_type: TypeId, return_type: TypeId) -> TypeId {
    // In a real implementation, this would create Generator<T, TReturn, TNext>
    // For now, just return the return type as a placeholder
    return_type
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_error_codes() {
        let error = FunctionError::new(FunctionErrorCode::RequiredAfterOptional, Span::new(0, 10));
        assert_eq!(error.code.code(), 1016);
        assert!(!error.message.is_empty());
    }

    #[test]
    fn test_function_context_default() {
        let ctx = FunctionContext::default();
        assert!(!ctx.is_async);
        assert!(!ctx.is_generator);
        assert!(!ctx.is_constructor);
        assert!(ctx.contextual_return_type.is_none());
    }

    #[test]
    fn test_parameter_validation_rest_not_last() {
        let mut store = SignatureStore::new();
        let mut checker = FunctionChecker::new(&mut store);

        let params = vec![
            Parameter::new(StringId::new(1), TypeId::STRING).rest(),
            Parameter::new(StringId::new(2), TypeId::NUMBER),
        ];

        checker.validate_parameters(&params);
        let errors = checker.take_errors();

        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.code == FunctionErrorCode::RestParameterMustBeLast));
    }

    #[test]
    fn test_parameter_validation_required_after_optional() {
        let mut store = SignatureStore::new();
        let mut checker = FunctionChecker::new(&mut store);

        let params = vec![
            Parameter::new(StringId::new(1), TypeId::STRING).optional(),
            Parameter::new(StringId::new(2), TypeId::NUMBER), // required after optional
        ];

        checker.validate_parameters(&params);
        let errors = checker.take_errors();

        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.code == FunctionErrorCode::RequiredAfterOptional));
    }

    #[test]
    fn test_parameter_validation_duplicate_name() {
        let mut store = SignatureStore::new();
        let mut checker = FunctionChecker::new(&mut store);

        let params = vec![
            Parameter::new(StringId::new(1), TypeId::STRING),
            Parameter::new(StringId::new(1), TypeId::NUMBER), // same name
        ];

        checker.validate_parameters(&params);
        let errors = checker.take_errors();

        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.code == FunctionErrorCode::DuplicateParameter));
    }

    #[test]
    fn test_function_declaration_check() {
        let mut store = SignatureStore::new();
        let mut checker = FunctionChecker::new(&mut store);

        let params = vec![
            Parameter::new(StringId::new(1), TypeId::STRING),
            Parameter::new(StringId::new(2), TypeId::NUMBER).optional(),
        ];

        let result = checker.check_function_declaration(
            NodeId::new(0),
            &[],
            &params,
            Some(TypeId::BOOLEAN),
        );

        assert!(result.is_ok());
        assert!(result.signature.is_some());

        let sig = store.get(result.signature).unwrap();
        assert_eq!(sig.parameters.len(), 2);
        assert_eq!(sig.min_argument_count, 1);
        assert_eq!(sig.return_type, TypeId::BOOLEAN);
    }

    #[test]
    fn test_constructor_no_return_type() {
        let mut store = SignatureStore::new();
        let mut checker = FunctionChecker::new(&mut store);

        let result = checker.check_constructor(
            NodeId::new(0),
            &[],
            true, // has explicit return type - error!
        );

        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e| e.code == FunctionErrorCode::ConstructorNoReturnType));
    }

    #[test]
    fn test_return_statement_void_function() {
        let mut store = SignatureStore::new();
        let mut checker = FunctionChecker::new(&mut store);

        let valid = checker.check_return_statement(
            TypeId::STRING,
            TypeId::VOID,
            Span::new(0, 10),
        );

        assert!(!valid);
        let errors = checker.take_errors();
        assert!(errors.iter().any(|e| e.code == FunctionErrorCode::VoidFunctionReturnsValue));
    }

    #[test]
    fn test_is_callable() {
        assert!(is_callable(TypeFlags::OBJECT));
        assert!(is_callable(TypeFlags::ANY));
        assert!(!is_callable(TypeFlags::STRING));
        assert!(!is_callable(TypeFlags::NUMBER));
    }
}
