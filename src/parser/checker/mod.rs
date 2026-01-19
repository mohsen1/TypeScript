//! TypeScript Type Checker
//!
//! This module implements type checking for TypeScript, focusing on:
//! - Function declarations and expressions
//! - Call expressions and new expressions
//! - Overload resolution
//! - Type signatures and inference
//!
//! # Architecture
//!
//! The type checker is organized into several sub-modules:
//!
//! - `signatures`: Type signatures, parameters, and type parameters
//! - `functions`: Function declaration and expression checking
//! - `calls`: Call expression checking and argument validation
//! - `overloads`: Overload resolution algorithm
//!
//! # Example
//!
//! ```
//! use ts_parser::checker::{
//!     SignatureStore, Signature, Parameter, TypeId,
//!     FunctionChecker, FunctionContext,
//!     CallChecker, CallContext, Argument,
//! };
//! use ts_parser::ast::{NodeId, StringId, Span};
//!
//! // Create a signature store
//! let mut store = SignatureStore::new();
//!
//! // Check a function declaration
//! let mut func_checker = FunctionChecker::new(&mut store);
//! let params = vec![
//!     Parameter::new(StringId::new(1), TypeId::STRING),
//!     Parameter::new(StringId::new(2), TypeId::NUMBER).optional(),
//! ];
//! let result = func_checker.check_function_declaration(
//!     NodeId::new(0),
//!     &[],  // no type parameters
//!     &params,
//!     Some(TypeId::BOOLEAN),
//! );
//!
//! assert!(result.is_ok());
//! ```

pub mod signatures;
pub mod functions;
pub mod calls;
pub mod overloads;

// Re-export commonly used types
pub use signatures::{
    TypeId, TypeFlags,
    SignatureId, Signature, SignatureFlags, SignatureStore,
    Parameter, ParameterFlags,
    TypeParameter,
};

pub use functions::{
    FunctionChecker, FunctionContext, FunctionCheckResult,
    FunctionError, FunctionErrorCode,
    is_callable, is_constructable,
    get_async_return_type, get_generator_return_type,
};

pub use calls::{
    CallChecker, CallContext, CallCheckResult,
    CallError, CallErrorCode,
    Argument, TypeArgument,
    check_optional_call,
};

pub use overloads::{
    OverloadResolver, OverloadContext, OverloadResolutionResult,
    OverloadMatchError, OverloadMismatchReason,
    collect_overload_signatures, check_overload_order,
};

/// Compiler options that affect type checking
#[derive(Debug, Clone)]
pub struct CheckerOptions {
    /// Enable strict null checks
    pub strict_null_checks: bool,
    /// Report errors on implicit any
    pub no_implicit_any: bool,
    /// Report errors when not all code paths return
    pub no_implicit_returns: bool,
    /// Enable strict mode for all files
    pub strict: bool,
    /// Allow unreachable code
    pub allow_unreachable_code: bool,
    /// Allow unused locals
    pub allow_unused_locals: bool,
    /// Always use strict mode
    pub always_strict: bool,
    /// Exact optional property types
    pub exact_optional_property_types: bool,
    /// No implicit override
    pub no_implicit_override: bool,
    /// No unchecked indexed access
    pub no_unchecked_indexed_access: bool,
    /// Use unknown in catch variables
    pub use_unknown_in_catch_variables: bool,
}

impl Default for CheckerOptions {
    fn default() -> Self {
        CheckerOptions {
            strict_null_checks: false,
            no_implicit_any: false,
            no_implicit_returns: false,
            strict: false,
            allow_unreachable_code: false,
            allow_unused_locals: false,
            always_strict: false,
            exact_optional_property_types: false,
            no_implicit_override: false,
            no_unchecked_indexed_access: false,
            use_unknown_in_catch_variables: false,
        }
    }
}

impl CheckerOptions {
    /// Create options with strict mode enabled
    pub fn strict() -> Self {
        CheckerOptions {
            strict: true,
            strict_null_checks: true,
            no_implicit_any: true,
            no_implicit_returns: true,
            always_strict: true,
            exact_optional_property_types: true,
            no_implicit_override: true,
            use_unknown_in_catch_variables: true,
            ..Default::default()
        }
    }
}

/// Unified type checker that coordinates all checking phases
pub struct TypeChecker {
    /// Signature storage
    pub signatures: SignatureStore,
    /// Checker options
    pub options: CheckerOptions,
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            signatures: SignatureStore::new(),
            options: CheckerOptions::default(),
        }
    }

    pub fn with_options(options: CheckerOptions) -> Self {
        TypeChecker {
            signatures: SignatureStore::new(),
            options,
        }
    }

    /// Create a function checker with current options
    pub fn function_checker(&mut self) -> FunctionChecker<'_> {
        let context = FunctionContext {
            no_implicit_any: self.options.no_implicit_any,
            no_implicit_returns: self.options.no_implicit_returns,
            strict_null_checks: self.options.strict_null_checks,
            ..Default::default()
        };
        FunctionChecker::new(&mut self.signatures).with_context(context)
    }

    /// Create a call checker with current options
    pub fn call_checker(&self) -> CallChecker<'_> {
        let context = CallContext {
            strict_null_checks: self.options.strict_null_checks,
            ..Default::default()
        };
        CallChecker::new(&self.signatures).with_context(context)
    }

    /// Create an overload resolver with current options
    pub fn overload_resolver(&self) -> OverloadResolver<'_> {
        OverloadResolver::new(&self.signatures)
    }

    /// Get a signature by ID
    pub fn get_signature(&self, id: SignatureId) -> Option<&Signature> {
        self.signatures.get(id)
    }

    /// Get the return type of a signature
    pub fn get_return_type(&self, sig_id: SignatureId) -> TypeId {
        self.signatures.get(sig_id)
            .map(|s| s.get_return_type())
            .unwrap_or(TypeId::ANY)
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{NodeId, StringId, Span};

    #[test]
    fn test_type_checker_creation() {
        let checker = TypeChecker::new();
        assert!(checker.signatures.is_empty());
    }

    #[test]
    fn test_type_checker_with_strict_options() {
        let checker = TypeChecker::with_options(CheckerOptions::strict());
        assert!(checker.options.strict);
        assert!(checker.options.strict_null_checks);
        assert!(checker.options.no_implicit_any);
    }

    #[test]
    fn test_function_checker_integration() {
        let mut checker = TypeChecker::new();

        let params = vec![
            Parameter::new(StringId::new(1), TypeId::STRING),
            Parameter::new(StringId::new(2), TypeId::NUMBER).optional(),
        ];

        let result = {
            let mut func_checker = checker.function_checker();
            func_checker.check_function_declaration(
                NodeId::new(0),
                &[],
                &params,
                Some(TypeId::BOOLEAN),
            )
        };

        assert!(result.is_ok());
        assert!(result.signature.is_some());

        let sig = checker.get_signature(result.signature).unwrap();
        assert_eq!(sig.parameters.len(), 2);
        assert_eq!(sig.min_argument_count, 1);
        assert_eq!(sig.return_type, TypeId::BOOLEAN);
    }

    #[test]
    fn test_call_checker_integration() {
        let mut checker = TypeChecker::new();

        // Create a signature
        let sig_id = {
            let sig = Signature::new(SignatureId::NONE)
                .with_parameters(vec![
                    Parameter::new(StringId::new(1), TypeId::STRING),
                ])
                .with_return_type(TypeId::NUMBER);
            checker.signatures.alloc_with(sig)
        };

        // Check a call
        let call_checker = checker.call_checker();
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];

        let mut call_checker = call_checker;
        let result = call_checker.check_call(sig_id, &args, None);

        assert!(result.is_ok());
        assert_eq!(result.return_type, TypeId::NUMBER);
    }

    #[test]
    fn test_overload_resolution_integration() {
        let mut checker = TypeChecker::new();

        // Create overloaded signatures
        let sig1_id = {
            let sig = Signature::new(SignatureId::NONE)
                .with_parameters(vec![Parameter::new(StringId::new(1), TypeId::STRING)])
                .with_return_type(TypeId::STRING);
            checker.signatures.alloc_with(sig)
        };

        let sig2_id = {
            let sig = Signature::new(SignatureId::NONE)
                .with_parameters(vec![Parameter::new(StringId::new(1), TypeId::NUMBER)])
                .with_return_type(TypeId::NUMBER);
            checker.signatures.alloc_with(sig)
        };

        // Resolve overloads
        let resolver = checker.overload_resolver();
        let args = vec![Argument::new(TypeId::NUMBER, Span::new(0, 5))];

        let result = resolver.resolve(&[sig1_id, sig2_id], &args, None);

        match result {
            OverloadResolutionResult::Success { signature, .. } => {
                assert_eq!(signature, sig2_id);
            }
            _ => panic!("Expected successful resolution"),
        }
    }

    #[test]
    fn test_generic_function_inference() {
        let mut checker = TypeChecker::new();

        // Create generic signature: <T>(x: T) => T
        let type_param_id = TypeId::new(100);
        let sig_id = {
            let sig = Signature::new(SignatureId::NONE)
                .with_type_parameters(vec![
                    TypeParameter::new(StringId::new(1), type_param_id)
                ])
                .with_parameters(vec![
                    Parameter::new(StringId::new(2), type_param_id)
                ])
                .with_return_type(type_param_id);
            checker.signatures.alloc_with(sig)
        };

        // Call with string argument
        let resolver = checker.overload_resolver();
        let args = vec![Argument::new(TypeId::STRING, Span::new(0, 5))];

        let result = resolver.resolve(&[sig_id], &args, None);

        match result {
            OverloadResolutionResult::Success { inferred_type_args, .. } => {
                assert_eq!(inferred_type_args.len(), 1);
                assert_eq!(inferred_type_args[0], TypeId::STRING);
            }
            _ => panic!("Expected successful inference"),
        }
    }

    #[test]
    fn test_rest_parameter_handling() {
        let mut checker = TypeChecker::new();

        // (x: string, ...rest: number[]) => void
        let sig_id = {
            let sig = Signature::new(SignatureId::NONE)
                .with_parameters(vec![
                    Parameter::new(StringId::new(1), TypeId::STRING),
                    Parameter::new(StringId::new(2), TypeId::NUMBER).rest(),
                ])
                .with_return_type(TypeId::VOID);
            checker.signatures.alloc_with(sig)
        };

        let sig = checker.get_signature(sig_id).unwrap();
        assert!(sig.has_rest_parameter());
        assert_eq!(sig.min_argument_count, 1);
        assert_eq!(sig.max_argument_count(), None); // Unlimited due to rest

        // Should accept many arguments
        assert!(sig.accepts_argument_count(1));
        assert!(sig.accepts_argument_count(5));
        assert!(sig.accepts_argument_count(100));
    }

    #[test]
    fn test_implicit_any_error() {
        let mut checker = TypeChecker::with_options(CheckerOptions {
            no_implicit_any: true,
            ..Default::default()
        });

        // Parameter without type annotation (simulated with ANY)
        let params = vec![
            Parameter::new(StringId::new(1), TypeId::ANY),
        ];

        let result = {
            let mut func_checker = checker.function_checker();
            func_checker.check_function_declaration(
                NodeId::new(0),
                &[],
                &params,
                None, // No return type - implicit any
            )
        };

        // Should have errors for implicit any
        assert!(!result.is_ok());
        assert!(result.errors.iter().any(|e|
            e.code == FunctionErrorCode::ImplicitAnyParameter ||
            e.code == FunctionErrorCode::ImplicitAnyReturn
        ));
    }
}
