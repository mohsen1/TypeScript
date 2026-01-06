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
use crate::solver::infer::InferenceContext;
use crate::solver::instantiate::{TypeSubstitution, instantiate_type};
use std::collections::HashMap;
use std::sync::Arc;

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
        // Handle generic functions
        if !func.type_params.is_empty() {
            return self.resolve_generic_call(func, arg_types);
        }

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

    /// Resolve a call to a generic function by inferring type arguments.
    fn resolve_generic_call(&mut self, func: &FunctionShape, arg_types: &[TypeId]) -> CallResult {
        let mut infer_ctx = InferenceContext::new(self.interner);
        let mut substitution = TypeSubstitution::new();
        let mut var_map: HashMap<TypeId, crate::solver::infer::InferenceVar> = HashMap::new();

        // 1. Create inference variables and placeholders for each type parameter
        for tp in &func.type_params {
            let var = infer_ctx.fresh_type_param(tp.name.clone());

            // Create a unique placeholder type for this inference variable
            // We use a TypeParameter with a special name to track it during constraint collection
            let placeholder_key = TypeKey::TypeParameter(TypeParamInfo {
                name: Arc::from(format!("__infer_{}", var.0)),
                constraint: tp.constraint,
                default: None,
            });
            let placeholder_id = self.interner.intern(placeholder_key);

            substitution.insert(tp.name.clone(), placeholder_id);
            var_map.insert(placeholder_id, var);
        }

        // 2. Instantiate parameters with placeholders
        let instantiated_params: Vec<ParamInfo> = func.params.iter().map(|p| {
            ParamInfo {
                name: p.name.clone(),
                type_id: instantiate_type(self.interner, p.type_id, &substitution),
                optional: p.optional,
                rest: p.rest,
            }
        }).collect();

        // 3. Collect constraints from arguments
        for (i, &arg_type) in arg_types.iter().enumerate() {
            if i >= instantiated_params.len() && !instantiated_params.last().map_or(false, |p| p.rest) {
                break;
            }

            let param_idx = if i >= instantiated_params.len() { instantiated_params.len() - 1 } else { i };
            let param = &instantiated_params[param_idx];

            let target_type = if param.rest {
                match self.interner.lookup(param.type_id) {
                    Some(TypeKey::Array(elem)) => elem,
                    _ => param.type_id,
                }
            } else {
                param.type_id
            };

            // arg_type <: target_type
            self.constrain_types(&mut infer_ctx, &var_map, arg_type, target_type);
        }

        // 4. Resolve inference variables
        match infer_ctx.resolve_all_with_constraints() {
            Ok(resolved_params) => {
                // Build final substitution
                let mut final_subst = TypeSubstitution::new();
                for (name, ty) in resolved_params {
                    final_subst.insert(name, ty);
                }

                // Instantiate return type
                let return_type = instantiate_type(self.interner, func.return_type, &final_subst);
                CallResult::Success(return_type)
            },
            Err(_) => {
                // Inference failed - return any (could be more specific error)
                CallResult::Success(TypeId::ANY)
            }
        }
    }

    /// Structural walker to collect constraints: source <: target
    fn constrain_types(
        &self,
        ctx: &mut InferenceContext,
        var_map: &HashMap<TypeId, crate::solver::infer::InferenceVar>,
        source: TypeId,
        target: TypeId
    ) {
        if source == target { return; }

        // If target is an inference placeholder, add lower bound: source <: var
        if let Some(&var) = var_map.get(&target) {
            ctx.add_lower_bound(var, source);
            return;
        }

        // If source is an inference placeholder, add upper bound: var <: target
        if let Some(&var) = var_map.get(&source) {
            ctx.add_upper_bound(var, target);
            return;
        }

        // Recurse structurally
        let source_key = self.interner.lookup(source);
        let target_key = self.interner.lookup(target);

        match (source_key, target_key) {
            (Some(TypeKey::Array(s_elem)), Some(TypeKey::Array(t_elem))) => {
                self.constrain_types(ctx, var_map, s_elem, t_elem);
            }
            (Some(TypeKey::Function(ref s_fn)), Some(TypeKey::Function(ref t_fn))) => {
                // Contravariant parameters: target_param <: source_param
                for (s_p, t_p) in s_fn.params.iter().zip(t_fn.params.iter()) {
                    self.constrain_types(ctx, var_map, t_p.type_id, s_p.type_id);
                }
                // Covariant return: source_return <: target_return
                self.constrain_types(ctx, var_map, s_fn.return_type, t_fn.return_type);
            }
            // TODO: Add support for Objects, Unions, Promises, etc.
            _ => {}
        }
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

    /// Type is possibly null or undefined.
    /// Contains the type of the property from non-nullable members (if any),
    /// and the specific nullable type causing the error.
    PossiblyNullOrUndefined {
        /// Type from valid non-nullable members (for recovery/optional chaining)
        property_type: Option<TypeId>,
        /// The nullable type causing the issue: NULL, UNDEFINED, or union of both
        cause: TypeId,
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
            return PropertyAccessResult::PossiblyNullOrUndefined {
                property_type: None,
                cause: obj_type,
            };
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
                // Property access on union: partition into nullable and non-nullable members
                let mut valid_results = Vec::new();
                let mut nullable_causes = Vec::new();

                for &member in members {
                    // Check for null/undefined directly
                    if member == TypeId::NULL || member == TypeId::UNDEFINED {
                        nullable_causes.push(member);
                        continue;
                    }

                    match self.resolve_property_access(member, prop_name) {
                        PropertyAccessResult::Success(t) => valid_results.push(t),
                        PropertyAccessResult::PossiblyNullOrUndefined { property_type, cause } => {
                            if let Some(t) = property_type {
                                valid_results.push(t);
                            }
                            nullable_causes.push(cause);
                        }
                        // If any non-nullable member is missing the property, it's a PropertyNotFound error
                        _ => return PropertyAccessResult::PropertyNotFound {
                            type_id: obj_type,
                            property_name: prop_name.to_string(),
                        },
                    }
                }

                // If there are nullable causes, return PossiblyNullOrUndefined
                if !nullable_causes.is_empty() {
                    let cause = if nullable_causes.len() == 1 {
                        nullable_causes[0]
                    } else {
                        self.interner.union(nullable_causes)
                    };

                    let property_type = if valid_results.is_empty() {
                        None
                    } else if valid_results.len() == 1 {
                        Some(valid_results[0])
                    } else {
                        Some(self.interner.union(valid_results))
                    };

                    return PropertyAccessResult::PossiblyNullOrUndefined {
                        property_type,
                        cause,
                    };
                }

                // Union of all result types
                PropertyAccessResult::Success(self.interner.union(valid_results))
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
            // Array properties
            "length" => PropertyAccessResult::Success(TypeId::NUMBER),

            // Array methods that return arrays
            "concat" | "filter" | "flat" | "flatMap" | "map" | "reverse" |
            "slice" | "sort" | "splice" | "toReversed" | "toSorted" |
            "toSpliced" | "with" => {
                // These return array-related types; for now, return ANY as placeholder
                // Full type inference would require understanding the callback return type
                PropertyAccessResult::Success(TypeId::ANY)
            }

            // Array methods that return specific types
            "at" | "find" | "findLast" | "pop" | "shift" => {
                // Returns element type or undefined; use ANY as placeholder
                PropertyAccessResult::Success(TypeId::ANY)
            }

            "every" | "includes" | "some" => {
                // Returns boolean
                PropertyAccessResult::Success(TypeId::BOOLEAN)
            }

            "findIndex" | "findLastIndex" | "indexOf" | "lastIndexOf" | "push" | "unshift" => {
                // Returns number
                PropertyAccessResult::Success(TypeId::NUMBER)
            }

            "forEach" | "copyWithin" | "fill" => {
                // forEach returns undefined, copyWithin/fill return this
                PropertyAccessResult::Success(TypeId::UNDEFINED)
            }

            "join" | "toLocaleString" | "toString" => {
                // Returns string
                PropertyAccessResult::Success(TypeId::STRING)
            }

            "entries" | "keys" | "values" => {
                // Returns iterator; use ANY as placeholder
                PropertyAccessResult::Success(TypeId::ANY)
            }

            "reduce" | "reduceRight" => {
                // Returns the accumulator type; use ANY as placeholder
                PropertyAccessResult::Success(TypeId::ANY)
            }

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
