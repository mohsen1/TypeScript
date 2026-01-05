//! Type operations and expression evaluation.
//!
//! This module contains the "brain" of the type system - all the logic for
//! evaluating expressions, resolving calls, accessing properties, etc.
//!
//! ## Architecture Principle
//!
//! The Solver handles **WHAT** (type operations and relations), while the
//! Checker handles **WHERE** (AST traversal, scoping, control flow).
//!
//! All functions here:
//! - Take `TypeId` as input (not AST nodes)
//! - Return structured results (not formatted error strings)
//! - Are pure logic (no side effects, no diagnostic formatting)
//!
//! This allows the Solver to be:
//! - Unit tested without AST nodes
//! - Reused across different checkers
//! - Optimized independently

use crate::solver::types::*;
use crate::solver::intern::TypeInterner;
use crate::solver::subtype::SubtypeChecker;
use crate::solver::diagnostics::PendingDiagnostic;

// =============================================================================
// Function Call Resolution
// =============================================================================

/// Result of attempting to call a function type.
#[derive(Clone, Debug)]
pub enum CallResult {
    /// Call succeeded, returns the result type
    Success(TypeId),

    /// Not a callable type
    NotCallable { type_id: TypeId },

    /// Argument count mismatch
    ArgumentCountMismatch {
        expected_min: usize,
        expected_max: Option<usize>,
        actual: usize,
    },

    /// Argument type mismatch at specific position
    ArgumentTypeMismatch {
        index: usize,
        expected: TypeId,
        actual: TypeId,
    },

    /// No overload matched (for overloaded functions)
    NoOverloadMatch {
        func_type: TypeId,
        arg_types: Vec<TypeId>,
        failures: Vec<PendingDiagnostic>,
    },
}

/// Evaluates function calls.
pub struct CallEvaluator<'a> {
    interner: &'a TypeInterner,
    subtype: &'a mut SubtypeChecker<'a>,
}

impl<'a> CallEvaluator<'a> {
    pub fn new(interner: &'a TypeInterner, subtype: &'a mut SubtypeChecker<'a>) -> Self {
        CallEvaluator { interner, subtype }
    }

    /// Resolve a function call: func(args...) -> result
    ///
    /// This is pure type logic - no AST nodes, just types in and types out.
    pub fn resolve_call(&mut self, func_type: TypeId, arg_types: &[TypeId]) -> CallResult {
        // Look up the function shape
        let key = match self.interner.lookup(func_type) {
            Some(k) => k,
            None => return CallResult::NotCallable { type_id: func_type },
        };

        match key {
            TypeKey::Function(ref f) => self.resolve_function_call(f, arg_types),
            TypeKey::Callable(ref c) => self.resolve_callable_call(c, arg_types),
            _ => CallResult::NotCallable { type_id: func_type },
        }
    }

    /// Resolve a call to a simple function type.
    fn resolve_function_call(&mut self, func: &FunctionShape, arg_types: &[TypeId]) -> CallResult {
        // Check argument count
        let min_args = func.params.iter().filter(|p| !p.optional).count();
        let max_args = if func.params.iter().any(|p| p.rest) {
            None
        } else {
            Some(func.params.len())
        };

        if arg_types.len() < min_args {
            return CallResult::ArgumentCountMismatch {
                expected_min: min_args,
                expected_max: max_args,
                actual: arg_types.len(),
            };
        }

        if let Some(max) = max_args {
            if arg_types.len() > max {
                return CallResult::ArgumentCountMismatch {
                    expected_min: min_args,
                    expected_max: Some(max),
                    actual: arg_types.len(),
                };
            }
        }

        // Check argument types
        for (i, arg_type) in arg_types.iter().enumerate() {
            if i >= func.params.len() {
                // Rest parameter or excess args already handled by count check
                break;
            }

            let param = &func.params[i];
            let param_type = if param.rest {
                // For rest parameters, unwrap the array type
                match self.interner.lookup(param.type_id) {
                    Some(TypeKey::Array(elem)) => elem,
                    _ => param.type_id,
                }
            } else {
                param.type_id
            };

            if !self.subtype.is_assignable_to(*arg_type, param_type) {
                return CallResult::ArgumentTypeMismatch {
                    index: i,
                    expected: param_type,
                    actual: *arg_type,
                };
            }
        }

        CallResult::Success(func.return_type)
    }

    /// Resolve a call to a callable type (with overloads).
    fn resolve_callable_call(&mut self, callable: &CallableShape, arg_types: &[TypeId]) -> CallResult {
        // Try each call signature
        let mut failures = Vec::new();

        for sig in &callable.call_signatures {
            // Convert CallSignature to FunctionShape
            let func = FunctionShape {
                params: sig.params.clone(),
                return_type: sig.return_type,
                type_params: Vec::new(),
                is_constructor: false,
            };

            match self.resolve_function_call(&func, arg_types) {
                CallResult::Success(ret) => return CallResult::Success(ret),
                CallResult::ArgumentTypeMismatch { index: _, expected, actual } => {
                    failures.push(
                        crate::solver::diagnostics::PendingDiagnosticBuilder::argument_not_assignable(
                            actual, expected
                        )
                    );
                }
                CallResult::ArgumentCountMismatch { expected_min, expected_max, actual } => {
                    let expected = expected_max.unwrap_or(expected_min);
                    failures.push(
                        crate::solver::diagnostics::PendingDiagnosticBuilder::argument_count_mismatch(
                            expected, actual
                        )
                    );
                }
                _ => {}
            }
        }

        // If we got here, no signature matched
        CallResult::NoOverloadMatch {
            func_type: self.interner.callable(callable.clone()),
            arg_types: arg_types.to_vec(),
            failures,
        }
    }
}

// =============================================================================
// Property Access Resolution
// =============================================================================

/// Result of attempting to access a property on a type.
#[derive(Clone, Debug)]
pub enum PropertyAccessResult {
    /// Property exists, returns its type
    Success(TypeId),

    /// Property does not exist on this type
    PropertyNotFound {
        type_id: TypeId,
        property_name: String,
    },

    /// Type is possibly null or undefined
    PossiblyNullOrUndefined {
        type_id: TypeId,
    },

    /// Type is unknown
    IsUnknown,
}

/// Evaluates property access.
pub struct PropertyAccessEvaluator<'a> {
    interner: &'a TypeInterner,
}

impl<'a> PropertyAccessEvaluator<'a> {
    pub fn new(interner: &'a TypeInterner) -> Self {
        PropertyAccessEvaluator { interner }
    }

    /// Resolve property access: obj.prop -> type
    pub fn resolve_property_access(
        &self,
        obj_type: TypeId,
        prop_name: &str,
    ) -> PropertyAccessResult {
        // Handle intrinsic types first
        if obj_type == TypeId::UNKNOWN {
            return PropertyAccessResult::IsUnknown;
        }

        if obj_type == TypeId::NULL || obj_type == TypeId::UNDEFINED {
            return PropertyAccessResult::PossiblyNullOrUndefined { type_id: obj_type };
        }

        // Look up the type key
        let key = match self.interner.lookup(obj_type) {
            Some(k) => k,
            None => return PropertyAccessResult::PropertyNotFound {
                type_id: obj_type,
                property_name: prop_name.to_string(),
            },
        };

        match key {
            TypeKey::Object(ref props) => {
                // Search for the property
                for prop in props {
                    if prop.name.as_ref() == prop_name {
                        return PropertyAccessResult::Success(prop.type_id);
                    }
                }
                PropertyAccessResult::PropertyNotFound {
                    type_id: obj_type,
                    property_name: prop_name.to_string(),
                }
            }

            TypeKey::ObjectWithIndex(ref shape) => {
                // Check named properties first
                for prop in &shape.properties {
                    if prop.name.as_ref() == prop_name {
                        return PropertyAccessResult::Success(prop.type_id);
                    }
                }

                // Check string index signature
                if let Some(ref idx) = shape.string_index {
                    return PropertyAccessResult::Success(idx.value_type);
                }

                PropertyAccessResult::PropertyNotFound {
                    type_id: obj_type,
                    property_name: prop_name.to_string(),
                }
            }

            TypeKey::Union(ref members) => {
                // Property access on union: must exist on ALL members
                let mut result_types = Vec::new();

                for &member in members {
                    match self.resolve_property_access(member, prop_name) {
                        PropertyAccessResult::Success(t) => result_types.push(t),
                        _ => return PropertyAccessResult::PropertyNotFound {
                            type_id: obj_type,
                            property_name: prop_name.to_string(),
                        },
                    }
                }

                // Union of all result types
                PropertyAccessResult::Success(self.interner.union(result_types))
            }

            TypeKey::Intersection(ref members) => {
                // Property access on intersection: check each member
                for &member in members {
                    if let PropertyAccessResult::Success(t) = self.resolve_property_access(member, prop_name) {
                        return PropertyAccessResult::Success(t);
                    }
                }

                PropertyAccessResult::PropertyNotFound {
                    type_id: obj_type,
                    property_name: prop_name.to_string(),
                }
            }

            // Built-in properties
            TypeKey::Intrinsic(IntrinsicKind::String) => {
                self.resolve_string_property(prop_name)
            }

            TypeKey::Array(_) => {
                self.resolve_array_property(obj_type, prop_name)
            }

            _ => PropertyAccessResult::PropertyNotFound {
                type_id: obj_type,
                property_name: prop_name.to_string(),
            },
        }
    }

    /// Resolve properties on string type.
    fn resolve_string_property(&self, prop_name: &str) -> PropertyAccessResult {
        match prop_name {
            "length" => PropertyAccessResult::Success(TypeId::NUMBER),
            // Add more string properties as needed
            _ => PropertyAccessResult::PropertyNotFound {
                type_id: TypeId::STRING,
                property_name: prop_name.to_string(),
            },
        }
    }

    /// Resolve properties on array type.
    fn resolve_array_property(&self, array_type: TypeId, prop_name: &str) -> PropertyAccessResult {
        match prop_name {
            "length" => PropertyAccessResult::Success(TypeId::NUMBER),
            // Add more array properties as needed
            _ => PropertyAccessResult::PropertyNotFound {
                type_id: array_type,
                property_name: prop_name.to_string(),
            },
        }
    }
}

// =============================================================================
// Binary Operations
// =============================================================================

/// Result of a binary operation.
#[derive(Clone, Debug)]
pub enum BinaryOpResult {
    /// Operation succeeded
    Success(TypeId),

    /// Type error in operation
    TypeError {
        left: TypeId,
        right: TypeId,
        op: &'static str,
    },
}

/// Evaluates binary operations.
pub struct BinaryOpEvaluator<'a> {
    interner: &'a TypeInterner,
}

impl<'a> BinaryOpEvaluator<'a> {
    pub fn new(interner: &'a TypeInterner) -> Self {
        BinaryOpEvaluator { interner }
    }

    /// Evaluate a binary operation: left op right -> result
    pub fn evaluate(&self, left: TypeId, right: TypeId, op: &'static str) -> BinaryOpResult {
        match op {
            "+" => self.evaluate_plus(left, right),
            "-" | "*" | "/" | "%" => self.evaluate_arithmetic(left, right),
            "==" | "!=" | "===" | "!==" => BinaryOpResult::Success(TypeId::BOOLEAN),
            "<" | ">" | "<=" | ">=" => self.evaluate_comparison(left, right),
            "&&" | "||" => self.evaluate_logical(left, right),
            _ => BinaryOpResult::TypeError { left, right, op },
        }
    }

    fn evaluate_plus(&self, left: TypeId, right: TypeId) -> BinaryOpResult {
        // string + any = string
        if left == TypeId::STRING || right == TypeId::STRING {
            return BinaryOpResult::Success(TypeId::STRING);
        }

        // number + number = number
        if left == TypeId::NUMBER && right == TypeId::NUMBER {
            return BinaryOpResult::Success(TypeId::NUMBER);
        }

        BinaryOpResult::TypeError { left, right, op: "+" }
    }

    fn evaluate_arithmetic(&self, left: TypeId, right: TypeId) -> BinaryOpResult {
        if left == TypeId::NUMBER && right == TypeId::NUMBER {
            BinaryOpResult::Success(TypeId::NUMBER)
        } else {
            BinaryOpResult::TypeError { left, right, op: "arithmetic" }
        }
    }

    fn evaluate_comparison(&self, _left: TypeId, _right: TypeId) -> BinaryOpResult {
        BinaryOpResult::Success(TypeId::BOOLEAN)
    }

    fn evaluate_logical(&self, left: TypeId, right: TypeId) -> BinaryOpResult {
        // For && and ||, TypeScript returns a union of the two types
        BinaryOpResult::Success(self.interner.union(vec![left, right]))
    }
}

#[cfg(test)]
#[path = "operations_tests.rs"]
mod tests;
