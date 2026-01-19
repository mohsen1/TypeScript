//! Assertion functions implementation for TypeScript.
//!
//! This module handles assertion functions like:
//! - asserts x is T
//! - asserts x
//! - assert(x) throws

use std::sync::Arc;

use super::type_defs::{Type, CallSignature, ParameterSignature};
use super::type_guards::TypeGuardEvaluator;

/// An assertion predicate (asserts x is T)
#[derive(Debug, Clone)]
pub struct AssertionPredicate {
    /// The parameter being asserted
    pub parameter_name: String,
    /// The parameter index in the signature
    pub parameter_index: usize,
    /// The type being asserted (None for "asserts x" without type)
    pub asserted_type: Option<Arc<Type>>,
}

/// Kind of assertion
#[derive(Debug, Clone)]
pub enum AssertionKind {
    /// asserts x is T - narrows x to T
    TypeAssertion(AssertionPredicate),
    /// asserts x - asserts truthiness
    Truthiness(AssertionPredicate),
    /// Function throws if assertion fails
    Throws,
}

/// Represents an assertion function signature
#[derive(Debug, Clone)]
pub struct AssertionSignature {
    /// The base call signature
    pub signature: CallSignature,
    /// The assertion predicate
    pub assertion: AssertionKind,
}

impl AssertionSignature {
    /// Create a new type assertion signature (asserts x is T)
    pub fn new_type_assertion(
        signature: CallSignature,
        parameter_name: String,
        parameter_index: usize,
        asserted_type: Arc<Type>,
    ) -> Self {
        Self {
            signature,
            assertion: AssertionKind::TypeAssertion(AssertionPredicate {
                parameter_name,
                parameter_index,
                asserted_type: Some(asserted_type),
            }),
        }
    }

    /// Create a new truthiness assertion signature (asserts x)
    pub fn new_truthiness_assertion(
        signature: CallSignature,
        parameter_name: String,
        parameter_index: usize,
    ) -> Self {
        Self {
            signature,
            assertion: AssertionKind::Truthiness(AssertionPredicate {
                parameter_name,
                parameter_index,
                asserted_type: None,
            }),
        }
    }

    /// Create a throws assertion (function throws on failure)
    pub fn new_throws_assertion(signature: CallSignature) -> Self {
        Self {
            signature,
            assertion: AssertionKind::Throws,
        }
    }
}

/// Evaluator for assertion functions
pub struct AssertionEvaluator;

impl AssertionEvaluator {
    /// Apply an assertion to narrow a type
    ///
    /// Returns the narrowed type after the assertion passes
    pub fn apply_assertion(
        assertion: &AssertionSignature,
        argument_types: &[Arc<Type>],
    ) -> Option<NarrowingEffect> {
        match &assertion.assertion {
            AssertionKind::TypeAssertion(predicate) => {
                Self::apply_type_assertion(predicate, argument_types)
            }
            AssertionKind::Truthiness(predicate) => {
                Self::apply_truthiness_assertion(predicate, argument_types)
            }
            AssertionKind::Throws => {
                // Throws assertions don't narrow types directly
                None
            }
        }
    }

    /// Apply a type assertion (asserts x is T)
    fn apply_type_assertion(
        predicate: &AssertionPredicate,
        argument_types: &[Arc<Type>],
    ) -> Option<NarrowingEffect> {
        let arg_type = argument_types.get(predicate.parameter_index)?;
        let asserted = predicate.asserted_type.as_ref()?;

        // Narrow the argument type to the asserted type
        let result = TypeGuardEvaluator::narrow_user_defined(
            arg_type.as_ref(),
            asserted.as_ref(),
            false,
        );

        Some(NarrowingEffect {
            parameter_index: predicate.parameter_index,
            parameter_name: predicate.parameter_name.clone(),
            narrowed_type: result.true_type,
        })
    }

    /// Apply a truthiness assertion (asserts x)
    fn apply_truthiness_assertion(
        predicate: &AssertionPredicate,
        argument_types: &[Arc<Type>],
    ) -> Option<NarrowingEffect> {
        let arg_type = argument_types.get(predicate.parameter_index)?;

        // Narrow by truthiness - removes null, undefined, false, 0, ""
        let result = TypeGuardEvaluator::narrow_truthiness(arg_type.as_ref(), false);

        Some(NarrowingEffect {
            parameter_index: predicate.parameter_index,
            parameter_name: predicate.parameter_name.clone(),
            narrowed_type: result.true_type,
        })
    }

    /// Check if a call signature has an assertion predicate
    pub fn has_assertion_predicate(signature: &AssertionSignature) -> bool {
        !matches!(signature.assertion, AssertionKind::Throws)
    }

    /// Get the assertion predicate if present
    pub fn get_assertion_predicate(signature: &AssertionSignature) -> Option<&AssertionPredicate> {
        match &signature.assertion {
            AssertionKind::TypeAssertion(p) | AssertionKind::Truthiness(p) => Some(p),
            AssertionKind::Throws => None,
        }
    }
}

/// Effect of narrowing from an assertion
#[derive(Debug, Clone)]
pub struct NarrowingEffect {
    /// Index of the parameter being narrowed
    pub parameter_index: usize,
    /// Name of the parameter
    pub parameter_name: String,
    /// The narrowed type
    pub narrowed_type: Arc<Type>,
}

/// Builder for assertion signatures
pub struct AssertionBuilder {
    parameters: Vec<ParameterSignature>,
    return_type: Arc<Type>,
}

impl AssertionBuilder {
    /// Create a new assertion builder
    pub fn new() -> Self {
        Self {
            parameters: Vec::new(),
            return_type: Arc::new(Type::Void),
        }
    }

    /// Add a parameter
    pub fn parameter(mut self, name: &str, type_: Arc<Type>) -> Self {
        self.parameters.push(ParameterSignature {
            name: name.to_string(),
            type_,
            optional: false,
            rest: false,
        });
        self
    }

    /// Add an optional parameter
    pub fn optional_parameter(mut self, name: &str, type_: Arc<Type>) -> Self {
        self.parameters.push(ParameterSignature {
            name: name.to_string(),
            type_,
            optional: true,
            rest: false,
        });
        self
    }

    /// Build a type assertion signature (asserts param is T)
    pub fn asserts_type(self, param_name: &str, asserted_type: Arc<Type>) -> AssertionSignature {
        let param_index = self
            .parameters
            .iter()
            .position(|p| p.name == param_name)
            .unwrap_or(0);

        let signature = CallSignature {
            type_parameters: Vec::new(),
            parameters: self.parameters,
            return_type: self.return_type,
        };

        AssertionSignature::new_type_assertion(
            signature,
            param_name.to_string(),
            param_index,
            asserted_type,
        )
    }

    /// Build a truthiness assertion signature (asserts param)
    pub fn asserts_truthy(self, param_name: &str) -> AssertionSignature {
        let param_index = self
            .parameters
            .iter()
            .position(|p| p.name == param_name)
            .unwrap_or(0);

        let signature = CallSignature {
            type_parameters: Vec::new(),
            parameters: self.parameters,
            return_type: self.return_type,
        };

        AssertionSignature::new_truthiness_assertion(signature, param_name.to_string(), param_index)
    }
}

impl Default for AssertionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_assertion() {
        // Build: function assert(value: unknown): asserts value is string
        let assertion = AssertionBuilder::new()
            .parameter("value", Arc::new(Type::Unknown))
            .asserts_type("value", Arc::new(Type::String));

        // Apply to unknown type
        let args = vec![Arc::new(Type::Unknown)];
        let effect = AssertionEvaluator::apply_assertion(&assertion, &args);

        assert!(effect.is_some());
        let effect = effect.unwrap();
        assert_eq!(effect.parameter_name, "value");
        match effect.narrowed_type.as_ref() {
            Type::String => (),
            _ => panic!("Expected String type"),
        }
    }

    #[test]
    fn test_truthiness_assertion() {
        // Build: function assertDefined<T>(value: T | null | undefined): asserts value
        let union = Type::Union(vec![
            Arc::new(Type::String),
            Arc::new(Type::Null),
            Arc::new(Type::Undefined),
        ]);

        let assertion = AssertionBuilder::new()
            .parameter("value", Arc::new(union.clone()))
            .asserts_truthy("value");

        // Apply to union type
        let args = vec![Arc::new(union)];
        let effect = AssertionEvaluator::apply_assertion(&assertion, &args);

        assert!(effect.is_some());
        let effect = effect.unwrap();
        // Should narrow to just String (removing null and undefined)
        match effect.narrowed_type.as_ref() {
            Type::String => (),
            _ => panic!("Expected String type after truthiness assertion"),
        }
    }

    #[test]
    fn test_assertion_with_union_narrowing() {
        // Build: function assertNumber(value: string | number): asserts value is number
        let union = Type::Union(vec![
            Arc::new(Type::String),
            Arc::new(Type::Number),
        ]);

        let assertion = AssertionBuilder::new()
            .parameter("value", Arc::new(union.clone()))
            .asserts_type("value", Arc::new(Type::Number));

        let args = vec![Arc::new(union)];
        let effect = AssertionEvaluator::apply_assertion(&assertion, &args);

        assert!(effect.is_some());
        let effect = effect.unwrap();
        match effect.narrowed_type.as_ref() {
            Type::Number => (),
            _ => panic!("Expected Number type"),
        }
    }

    #[test]
    fn test_has_assertion_predicate() {
        let type_assertion = AssertionBuilder::new()
            .parameter("value", Arc::new(Type::Unknown))
            .asserts_type("value", Arc::new(Type::String));

        assert!(AssertionEvaluator::has_assertion_predicate(&type_assertion));

        let throws_assertion = AssertionSignature::new_throws_assertion(CallSignature {
            type_parameters: Vec::new(),
            parameters: Vec::new(),
            return_type: Arc::new(Type::Never),
        });

        assert!(!AssertionEvaluator::has_assertion_predicate(&throws_assertion));
    }

    #[test]
    fn test_get_assertion_predicate() {
        let assertion = AssertionBuilder::new()
            .parameter("value", Arc::new(Type::Unknown))
            .asserts_type("value", Arc::new(Type::String));

        let predicate = AssertionEvaluator::get_assertion_predicate(&assertion);
        assert!(predicate.is_some());
        let predicate = predicate.unwrap();
        assert_eq!(predicate.parameter_name, "value");
        assert_eq!(predicate.parameter_index, 0);
        assert!(predicate.asserted_type.is_some());
    }
}
