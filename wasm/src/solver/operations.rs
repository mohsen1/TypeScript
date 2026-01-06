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
use crate::solver::TypeDatabase;
use crate::solver::subtype::SubtypeChecker;
use crate::solver::diagnostics::PendingDiagnostic;
use crate::solver::infer::InferenceContext;
use crate::solver::instantiate::{TypeSubstitution, instantiate_type};
use std::collections::HashMap;

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
    interner: &'a dyn TypeDatabase,
    subtype: &'a mut SubtypeChecker<'a>,
}

impl<'a> CallEvaluator<'a> {
    pub fn new(interner: &'a dyn TypeDatabase, subtype: &'a mut SubtypeChecker<'a>) -> Self {
        CallEvaluator { interner, subtype }
    }

    pub fn infer_call_signature(&mut self, sig: &CallSignature, arg_types: &[TypeId]) -> TypeId {
        let func = FunctionShape {
            params: sig.params.clone(),
            return_type: sig.return_type,
            type_params: sig.type_params.clone(),
            is_constructor: false,
        };
        match self.resolve_function_call(&func, arg_types) {
            CallResult::Success(ret) => ret,
            _ => TypeId::ANY,
        }
    }

    pub fn infer_generic_function(&mut self, func: &FunctionShape, arg_types: &[TypeId]) -> TypeId {
        match self.resolve_function_call(func, arg_types) {
            CallResult::Success(ret) => ret,
            _ => TypeId::ANY,
        }
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
            // Resolve Atom to String for inference context (still uses Arc<str>)
            let tp_name_str = self.interner.resolve_atom(tp.name);
            let var = infer_ctx.fresh_type_param(std::sync::Arc::from(tp_name_str.as_str()));

            // Create a unique placeholder type for this inference variable
            // We use a TypeParameter with a special name to track it during constraint collection
            let placeholder_name = format!("__infer_{}", var.0);
            let placeholder_key = TypeKey::TypeParameter(TypeParamInfo {
                name: self.interner.intern_string(&placeholder_name),
                constraint: tp.constraint,
                default: None,
            });
            let placeholder_id = self.interner.intern(placeholder_key);

            // TypeSubstitution still uses Arc<str>, so resolve the atom
            substitution.insert(std::sync::Arc::from(self.interner.resolve_atom(tp.name).as_str()), placeholder_id);
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

        let is_nullish = |ty: TypeId| matches!(ty, TypeId::NULL | TypeId::UNDEFINED | TypeId::VOID);

        match (source_key, target_key) {
            (Some(TypeKey::Union(ref s_members)), _) => {
                for &member in s_members {
                    self.constrain_types(ctx, var_map, member, target);
                }
            }
            (_, Some(TypeKey::Intersection(ref t_members))) => {
                for &member in t_members {
                    self.constrain_types(ctx, var_map, source, member);
                }
            }
            (_, Some(TypeKey::Union(ref t_members))) => {
                let mut non_nullable = Vec::new();
                for &member in t_members {
                    if !is_nullish(member) {
                        non_nullable.push(member);
                    }
                }
                if non_nullable.len() == 1 {
                    self.constrain_types(ctx, var_map, source, non_nullable[0]);
                }
            }
            (Some(TypeKey::Array(s_elem)), Some(TypeKey::Array(t_elem))) => {
                self.constrain_types(ctx, var_map, s_elem, t_elem);
            }
            (Some(TypeKey::Tuple(ref s_elems)), Some(TypeKey::Tuple(ref t_elems))) => {
                for (s_elem, t_elem) in s_elems.iter().zip(t_elems.iter()) {
                    self.constrain_types(ctx, var_map, s_elem.type_id, t_elem.type_id);
                }
            }
            (Some(TypeKey::Function(ref s_fn)), Some(TypeKey::Function(ref t_fn))) => {
                // Contravariant parameters: target_param <: source_param
                for (s_p, t_p) in s_fn.params.iter().zip(t_fn.params.iter()) {
                    self.constrain_types(ctx, var_map, t_p.type_id, s_p.type_id);
                }
                // Covariant return: source_return <: target_return
                self.constrain_types(ctx, var_map, s_fn.return_type, t_fn.return_type);
            }
            (Some(TypeKey::Object(ref s_props)), Some(TypeKey::Object(ref t_props))) => {
                self.constrain_properties(ctx, var_map, s_props, t_props);
            }
            (Some(TypeKey::ObjectWithIndex(ref s_shape)), Some(TypeKey::ObjectWithIndex(ref t_shape))) => {
                self.constrain_properties(ctx, var_map, &s_shape.properties, &t_shape.properties);
                if let (Some(s_idx), Some(t_idx)) = (&s_shape.string_index, &t_shape.string_index) {
                    self.constrain_types(ctx, var_map, s_idx.value_type, t_idx.value_type);
                }
                if let (Some(s_idx), Some(t_idx)) = (&s_shape.number_index, &t_shape.number_index) {
                    self.constrain_types(ctx, var_map, s_idx.value_type, t_idx.value_type);
                }
            }
            (Some(TypeKey::Object(ref s_props)), Some(TypeKey::ObjectWithIndex(ref t_shape))) => {
                self.constrain_properties(ctx, var_map, s_props, &t_shape.properties);
            }
            (Some(TypeKey::ObjectWithIndex(ref s_shape)), Some(TypeKey::Object(ref t_props))) => {
                self.constrain_properties(ctx, var_map, &s_shape.properties, t_props);
            }
            (Some(TypeKey::Application(ref s_app)), Some(TypeKey::Application(ref t_app))) => {
                if s_app.base == t_app.base && s_app.args.len() == t_app.args.len() {
                    for (s_arg, t_arg) in s_app.args.iter().zip(t_app.args.iter()) {
                        self.constrain_types(ctx, var_map, *s_arg, *t_arg);
                    }
                }
            }
            _ => {}
        }
    }

    fn constrain_properties(
        &self,
        ctx: &mut InferenceContext,
        var_map: &HashMap<TypeId, crate::solver::infer::InferenceVar>,
        source_props: &[PropertyInfo],
        target_props: &[PropertyInfo],
    ) {
        let mut source_idx = 0;
        let mut target_idx = 0;

        while source_idx < source_props.len() && target_idx < target_props.len() {
            let source = &source_props[source_idx];
            let target = &target_props[target_idx];

            match source.name.cmp(&target.name) {
                std::cmp::Ordering::Equal => {
                    self.constrain_types(ctx, var_map, source.type_id, target.type_id);
                    source_idx += 1;
                    target_idx += 1;
                }
                std::cmp::Ordering::Less => {
                    source_idx += 1;
                }
                std::cmp::Ordering::Greater => {
                    target_idx += 1;
                }
            }
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

pub fn infer_call_signature<'a>(
    interner: &'a dyn TypeDatabase,
    subtype: &'a mut SubtypeChecker<'a>,
    sig: &CallSignature,
    arg_types: &[TypeId],
) -> TypeId {
    let mut evaluator = CallEvaluator::new(interner, subtype);
    evaluator.infer_call_signature(sig, arg_types)
}

pub fn infer_generic_function<'a>(
    interner: &'a dyn TypeDatabase,
    subtype: &'a mut SubtypeChecker<'a>,
    func: &FunctionShape,
    arg_types: &[TypeId],
) -> TypeId {
    let mut evaluator = CallEvaluator::new(interner, subtype);
    evaluator.infer_generic_function(func, arg_types)
}

// =============================================================================
// Property Access Resolution
// =============================================================================

/// Result of attempting to access a property on a type.
#[derive(Clone, Debug)]
pub enum PropertyAccessResult {
    /// Property exists, returns its type
    Success {
        type_id: TypeId,
        /// True if this property was resolved via an index signature
        /// (not an explicit property declaration). Used for error 4111.
        from_index_signature: bool,
    },

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
    interner: &'a dyn TypeDatabase,
}

impl<'a> PropertyAccessEvaluator<'a> {
    pub fn new(interner: &'a dyn TypeDatabase) -> Self {
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

        // Handle Symbol primitive properties
        if obj_type == TypeId::SYMBOL {
            return self.resolve_symbol_primitive_property(prop_name);
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
                    if self.interner.resolve_atom(prop.name) == prop_name {
                        return PropertyAccessResult::Success {
                            type_id: prop.type_id,
                            from_index_signature: false,
                        };
                    }
                }
                PropertyAccessResult::PropertyNotFound {
                    type_id: obj_type,
                    property_name: prop_name.to_string(),
                }
            }

            TypeKey::ObjectWithIndex(ref shape) => {
                // Check named properties first (explicit properties take precedence)
                for prop in &shape.properties {
                    if self.interner.resolve_atom(prop.name) == prop_name {
                        return PropertyAccessResult::Success {
                            type_id: prop.type_id,
                            from_index_signature: false,
                        };
                    }
                }

                // Check string index signature (THIS is the case for error 4111)
                if let Some(ref idx) = shape.string_index {
                    return PropertyAccessResult::Success {
                        type_id: idx.value_type,
                        from_index_signature: true,  // Resolved via index signature!
                    };
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
                let mut any_from_index = false;  // Track if any member used index signature

                for &member in members {
                    // Check for null/undefined directly
                    if member == TypeId::NULL || member == TypeId::UNDEFINED {
                        nullable_causes.push(member);
                        continue;
                    }

                    match self.resolve_property_access(member, prop_name) {
                        PropertyAccessResult::Success { type_id, from_index_signature } => {
                            valid_results.push(type_id);
                            if from_index_signature {
                                any_from_index = true;  // Propagate: if ANY member uses index, flag it
                            }
                        }
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
                PropertyAccessResult::Success {
                    type_id: self.interner.union(valid_results),
                    from_index_signature: any_from_index,  // Contagious across union members
                }
            }

            TypeKey::Intersection(ref members) => {
                // Property access on intersection: check each member
                for &member in members {
                    if let PropertyAccessResult::Success { type_id, from_index_signature } = self.resolve_property_access(member, prop_name) {
                        return PropertyAccessResult::Success { type_id, from_index_signature };
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
            "length" => PropertyAccessResult::Success {
                type_id: TypeId::NUMBER,
                from_index_signature: false,
            },
            // Add more string properties as needed
            _ => PropertyAccessResult::PropertyNotFound {
                type_id: TypeId::STRING,
                property_name: prop_name.to_string(),
            },
        }
    }

    /// Resolve properties on symbol primitive type.
    fn resolve_symbol_primitive_property(&self, prop_name: &str) -> PropertyAccessResult {
        match prop_name {
            // Symbol.prototype.description: string | undefined
            "description" => {
                let union = self.interner.union(vec![TypeId::STRING, TypeId::UNDEFINED]);
                PropertyAccessResult::Success {
                    type_id: union,
                    from_index_signature: false,
                }
            }
            // Symbol.prototype.toString(): string
            // Symbol.prototype.valueOf(): symbol
            // For now, return ANY for methods as full function type synthesis is complex
            "toString" | "valueOf" => PropertyAccessResult::Success {
                type_id: TypeId::ANY,
                from_index_signature: false,
            },
            _ => PropertyAccessResult::PropertyNotFound {
                type_id: TypeId::SYMBOL,
                property_name: prop_name.to_string(),
            },
        }
    }

    /// Resolve properties on array type.
    fn resolve_array_property(&self, array_type: TypeId, prop_name: &str) -> PropertyAccessResult {
        match prop_name {
            // Array properties
            "length" => PropertyAccessResult::Success { type_id: TypeId::NUMBER, from_index_signature: false },

            // Array methods that return arrays
            "concat" | "filter" | "flat" | "flatMap" | "map" | "reverse" |
            "slice" | "sort" | "splice" | "toReversed" | "toSorted" |
            "toSpliced" | "with" => {
                // These return array-related types; for now, return ANY as placeholder
                // Full type inference would require understanding the callback return type
                PropertyAccessResult::Success { type_id: TypeId::ANY, from_index_signature: false }
            }

            // Array methods that return specific types
            "at" | "find" | "findLast" | "pop" | "shift" => {
                // Returns element type or undefined; use ANY as placeholder
                PropertyAccessResult::Success { type_id: TypeId::ANY, from_index_signature: false }
            }

            "every" | "includes" | "some" => {
                // Returns boolean
                PropertyAccessResult::Success { type_id: TypeId::BOOLEAN, from_index_signature: false }
            }

            "findIndex" | "findLastIndex" | "indexOf" | "lastIndexOf" | "push" | "unshift" => {
                // Returns number
                PropertyAccessResult::Success { type_id: TypeId::NUMBER, from_index_signature: false }
            }

            "forEach" | "copyWithin" | "fill" => {
                // forEach returns undefined, copyWithin/fill return this
                PropertyAccessResult::Success { type_id: TypeId::UNDEFINED, from_index_signature: false }
            }

            "join" | "toLocaleString" | "toString" => {
                // Returns string
                PropertyAccessResult::Success { type_id: TypeId::STRING, from_index_signature: false }
            }

            "entries" | "keys" | "values" => {
                // Returns iterator; use ANY as placeholder
                PropertyAccessResult::Success { type_id: TypeId::ANY, from_index_signature: false }
            }

            "reduce" | "reduceRight" => {
                // Returns the accumulator type; use ANY as placeholder
                PropertyAccessResult::Success { type_id: TypeId::ANY, from_index_signature: false }
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
    interner: &'a dyn TypeDatabase,
}

impl<'a> BinaryOpEvaluator<'a> {
    pub fn new(interner: &'a dyn TypeDatabase) -> Self {
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
