//! Conditional type inference utilities.
//!
//! This module provides additional utilities for handling complex conditional type
//! inference scenarios including:
//! - Distributive conditional types
//! - Deferred conditional type resolution
//! - Inference priority and ordering
//! - Recursive conditional type handling

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[allow(unused_imports)]
use super::type_defs::{
    Type, TypeParameter, ConditionalType, ObjectType, TupleType, TupleElement,
    FunctionType, CallSignature, ParameterSignature,
};
#[allow(unused_imports)]
use super::infer::{
    InferenceContext, InferenceResult, ConditionalTypeInferrer, Variance, InferTypeBuilder,
};

/// Represents a distributive conditional type evaluation
///
/// When the check type is a naked type parameter, the conditional
/// distributes over union types:
/// `T extends U ? X : Y` where T = A | B becomes
/// `(A extends U ? X : Y) | (B extends U ? X : Y)`
#[derive(Debug, Clone)]
pub struct DistributiveConditional {
    /// The conditional type being distributed
    pub conditional: ConditionalType,
    /// Whether distribution is enabled
    pub distribute: bool,
}

impl DistributiveConditional {
    /// Create a new distributive conditional
    pub fn new(conditional: ConditionalType) -> Self {
        let distribute = Self::should_distribute(&conditional.check_type);
        Self {
            conditional,
            distribute,
        }
    }

    /// Check if a type is a naked type parameter (should distribute)
    fn should_distribute(type_: &Type) -> bool {
        matches!(type_, Type::TypeParameter(_))
    }

    /// Evaluate the conditional type with distribution
    pub fn evaluate(&self, substituted_check: &Type) -> Arc<Type> {
        if self.distribute {
            if let Type::Union(members) = substituted_check {
                // Distribute over union members
                let results: Vec<Arc<Type>> = members
                    .iter()
                    .map(|member| {
                        let sub_conditional = ConditionalType {
                            check_type: member.clone(),
                            extends_type: self.conditional.extends_type.clone(),
                            true_type: self.conditional.true_type.clone(),
                            false_type: self.conditional.false_type.clone(),
                        };
                        let mut ctx = InferenceContext::new();
                        ConditionalTypeInferrer::evaluate(&sub_conditional, &mut ctx)
                    })
                    .collect();

                // Simplify union if possible
                return Self::simplify_union(results);
            }
        }

        // Non-distributive evaluation
        let mut ctx = InferenceContext::new();
        let eval_conditional = ConditionalType {
            check_type: Arc::new(substituted_check.clone()),
            extends_type: self.conditional.extends_type.clone(),
            true_type: self.conditional.true_type.clone(),
            false_type: self.conditional.false_type.clone(),
        };
        ConditionalTypeInferrer::evaluate(&eval_conditional, &mut ctx)
    }

    /// Simplify a union type by removing duplicates and never types
    fn simplify_union(types: Vec<Arc<Type>>) -> Arc<Type> {
        let mut filtered: Vec<Arc<Type>> = types
            .into_iter()
            .filter(|t| !matches!(t.as_ref(), Type::Never))
            .collect();

        // Remove exact duplicates (simple comparison)
        filtered.dedup_by(|a, b| {
            match (a.as_ref(), b.as_ref()) {
                (Type::String, Type::String) => true,
                (Type::Number, Type::Number) => true,
                (Type::Boolean, Type::Boolean) => true,
                (Type::StringLiteral(x), Type::StringLiteral(y)) => x == y,
                (Type::NumberLiteral(x), Type::NumberLiteral(y)) => x == y,
                (Type::BooleanLiteral(x), Type::BooleanLiteral(y)) => x == y,
                _ => false,
            }
        });

        match filtered.len() {
            0 => Arc::new(Type::Never),
            1 => filtered.remove(0),
            _ => Arc::new(Type::Union(filtered)),
        }
    }
}

/// Deferred conditional type that hasn't been resolved yet
#[derive(Debug, Clone)]
pub struct DeferredConditional {
    /// The original conditional type
    pub conditional: ConditionalType,
    /// Unresolved type parameters
    pub unresolved_params: HashSet<String>,
    /// Partial inference results
    pub partial_inference: HashMap<String, Arc<Type>>,
}

impl DeferredConditional {
    /// Create a new deferred conditional
    pub fn new(conditional: ConditionalType) -> Self {
        let unresolved = Self::find_unresolved_params(&conditional);
        Self {
            conditional,
            unresolved_params: unresolved,
            partial_inference: HashMap::new(),
        }
    }

    /// Find all unresolved type parameters in a conditional
    fn find_unresolved_params(conditional: &ConditionalType) -> HashSet<String> {
        let mut params = HashSet::new();
        Self::collect_type_params(&conditional.check_type, &mut params);
        Self::collect_type_params(&conditional.extends_type, &mut params);
        Self::collect_type_params(&conditional.true_type, &mut params);
        Self::collect_type_params(&conditional.false_type, &mut params);
        params
    }

    /// Recursively collect type parameter names
    fn collect_type_params(type_: &Type, params: &mut HashSet<String>) {
        match type_ {
            Type::TypeParameter(tp) => {
                if !tp.name.starts_with("infer ") {
                    params.insert(tp.name.clone());
                }
            }
            Type::Union(members) | Type::Intersection(members) => {
                for member in members {
                    Self::collect_type_params(member, params);
                }
            }
            Type::Array(element) => {
                Self::collect_type_params(element, params);
            }
            Type::Tuple(tuple) => {
                for elem in &tuple.element_types {
                    Self::collect_type_params(&elem.type_, params);
                }
            }
            Type::Object(obj) => {
                for prop in obj.properties.values() {
                    Self::collect_type_params(&prop.type_, params);
                }
            }
            Type::Function(func) => {
                for sig in &func.signatures {
                    for param in &sig.parameters {
                        Self::collect_type_params(&param.type_, params);
                    }
                    Self::collect_type_params(&sig.return_type, params);
                }
            }
            Type::Conditional(cond) => {
                Self::collect_type_params(&cond.check_type, params);
                Self::collect_type_params(&cond.extends_type, params);
                Self::collect_type_params(&cond.true_type, params);
                Self::collect_type_params(&cond.false_type, params);
            }
            Type::IndexedAccess(idx) => {
                Self::collect_type_params(&idx.object_type, params);
                Self::collect_type_params(&idx.index_type, params);
            }
            Type::Index(target) => {
                Self::collect_type_params(target, params);
            }
            _ => {}
        }
    }

    /// Check if the conditional can be resolved
    pub fn can_resolve(&self) -> bool {
        self.unresolved_params.is_empty()
    }

    /// Add a type parameter resolution
    pub fn resolve_param(&mut self, name: &str, type_: Arc<Type>) {
        self.unresolved_params.remove(name);
        self.partial_inference.insert(name.to_string(), type_);
    }

    /// Try to resolve the conditional type
    pub fn try_resolve(&self) -> Option<Arc<Type>> {
        if !self.can_resolve() {
            return None;
        }

        // Substitute all resolved parameters
        let substituted = ConditionalType {
            check_type: ConditionalTypeInferrer::substitute_inferred(
                &self.conditional.check_type,
                &self.partial_inference,
            ),
            extends_type: ConditionalTypeInferrer::substitute_inferred(
                &self.conditional.extends_type,
                &self.partial_inference,
            ),
            true_type: ConditionalTypeInferrer::substitute_inferred(
                &self.conditional.true_type,
                &self.partial_inference,
            ),
            false_type: ConditionalTypeInferrer::substitute_inferred(
                &self.conditional.false_type,
                &self.partial_inference,
            ),
        };

        let mut ctx = InferenceContext::new();
        Some(ConditionalTypeInferrer::evaluate(&substituted, &mut ctx))
    }
}

/// Inference priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InferencePriority {
    /// Highest priority - direct match
    Exact = 0,
    /// Literal type match
    Literal = 1,
    /// Subtype match
    Subtype = 2,
    /// Widened type match
    Widened = 3,
    /// Lowest priority - any/unknown fallback
    Fallback = 4,
}

/// Extended inference context with priority tracking
#[derive(Debug)]
pub struct PriorityInferenceContext {
    /// Base inference context
    base: InferenceContext,
    /// Priority assignments for each candidate
    priorities: HashMap<String, Vec<InferencePriority>>,
}

impl PriorityInferenceContext {
    /// Create a new priority inference context
    pub fn new() -> Self {
        Self {
            base: InferenceContext::new(),
            priorities: HashMap::new(),
        }
    }

    /// Add a candidate with priority
    pub fn add_candidate_with_priority(
        &mut self,
        name: &str,
        type_: Arc<Type>,
        priority: InferencePriority,
    ) {
        self.base.add_candidate(name, type_, priority as usize);
        self.priorities
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(priority);
    }

    /// Get the base context
    pub fn base_mut(&mut self) -> &mut InferenceContext {
        &mut self.base
    }

    /// Determine the priority of a type match
    pub fn determine_priority(source: &Type, target: &Type) -> InferencePriority {
        match (source, target) {
            // Exact match
            (Type::String, Type::String)
            | (Type::Number, Type::Number)
            | (Type::Boolean, Type::Boolean) => InferencePriority::Exact,

            // Literal match
            (Type::StringLiteral(_), Type::StringLiteral(_))
            | (Type::NumberLiteral(_), Type::NumberLiteral(_))
            | (Type::BooleanLiteral(_), Type::BooleanLiteral(_)) => InferencePriority::Literal,

            // Literal to base type
            (Type::StringLiteral(_), Type::String)
            | (Type::NumberLiteral(_), Type::Number)
            | (Type::BooleanLiteral(_), Type::Boolean) => InferencePriority::Subtype,

            // Any/unknown
            (Type::Any, _) | (_, Type::Any) | (_, Type::Unknown) => InferencePriority::Fallback,

            // Default widened match
            _ => InferencePriority::Widened,
        }
    }
}

impl Default for PriorityInferenceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Recursive conditional type handler
///
/// Handles deeply nested conditional types by tracking recursion depth
/// and preventing infinite loops
#[derive(Debug)]
pub struct RecursiveConditionalHandler {
    /// Maximum recursion depth
    max_depth: usize,
    /// Current recursion stack (type signatures)
    recursion_stack: Vec<String>,
    /// Cache for resolved conditionals
    cache: HashMap<String, Arc<Type>>,
}

impl RecursiveConditionalHandler {
    /// Create a new handler with default max depth
    pub fn new() -> Self {
        Self {
            max_depth: 50,
            recursion_stack: Vec::new(),
            cache: HashMap::new(),
        }
    }

    /// Create with custom max depth
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            max_depth,
            recursion_stack: Vec::new(),
            cache: HashMap::new(),
        }
    }

    /// Get a signature for a conditional type (for recursion detection)
    fn get_signature(conditional: &ConditionalType) -> String {
        format!(
            "({:?} extends {:?} ? {:?} : {:?})",
            conditional.check_type,
            conditional.extends_type,
            conditional.true_type,
            conditional.false_type
        )
    }

    /// Check if we're in a recursive loop
    pub fn is_recursive(&self, conditional: &ConditionalType) -> bool {
        let sig = Self::get_signature(conditional);
        self.recursion_stack.contains(&sig)
    }

    /// Check if we've exceeded max depth
    pub fn is_too_deep(&self) -> bool {
        self.recursion_stack.len() >= self.max_depth
    }

    /// Enter a conditional evaluation
    pub fn enter(&mut self, conditional: &ConditionalType) -> bool {
        if self.is_too_deep() || self.is_recursive(conditional) {
            return false;
        }

        let sig = Self::get_signature(conditional);
        self.recursion_stack.push(sig);
        true
    }

    /// Exit a conditional evaluation
    pub fn exit(&mut self) {
        self.recursion_stack.pop();
    }

    /// Evaluate a conditional type with recursion handling
    pub fn evaluate(&mut self, conditional: &ConditionalType) -> Arc<Type> {
        let sig = Self::get_signature(conditional);

        // Check cache
        if let Some(cached) = self.cache.get(&sig) {
            return cached.clone();
        }

        // Check recursion
        if !self.enter(conditional) {
            // Return the conditional as-is (deferred) if recursive
            return Arc::new(Type::Conditional(conditional.clone()));
        }

        // Evaluate
        let mut ctx = InferenceContext::new();
        let result = ConditionalTypeInferrer::evaluate(conditional, &mut ctx);

        self.exit();

        // Cache result
        self.cache.insert(sig, result.clone());

        result
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get current depth
    pub fn current_depth(&self) -> usize {
        self.recursion_stack.len()
    }
}

impl Default for RecursiveConditionalHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for working with infer types
pub struct InferUtils;

impl InferUtils {
    /// Extract all infer type names from a type
    pub fn find_infer_types(type_: &Type) -> Vec<String> {
        let mut names = Vec::new();
        Self::find_infer_types_recursive(type_, &mut names);
        names
    }

    fn find_infer_types_recursive(type_: &Type, names: &mut Vec<String>) {
        match type_ {
            Type::TypeParameter(tp) => {
                if let Some(name) = tp.name.strip_prefix("infer ") {
                    names.push(name.to_string());
                }
            }
            Type::Union(members) | Type::Intersection(members) => {
                for member in members {
                    Self::find_infer_types_recursive(member, names);
                }
            }
            Type::Array(element) => {
                Self::find_infer_types_recursive(element, names);
            }
            Type::Tuple(tuple) => {
                for elem in &tuple.element_types {
                    Self::find_infer_types_recursive(&elem.type_, names);
                }
            }
            Type::Object(obj) => {
                for prop in obj.properties.values() {
                    Self::find_infer_types_recursive(&prop.type_, names);
                }
                for sig in &obj.call_signatures {
                    for param in &sig.parameters {
                        Self::find_infer_types_recursive(&param.type_, names);
                    }
                    Self::find_infer_types_recursive(&sig.return_type, names);
                }
            }
            Type::Function(func) => {
                for sig in &func.signatures {
                    for param in &sig.parameters {
                        Self::find_infer_types_recursive(&param.type_, names);
                    }
                    Self::find_infer_types_recursive(&sig.return_type, names);
                }
            }
            Type::Conditional(cond) => {
                Self::find_infer_types_recursive(&cond.check_type, names);
                Self::find_infer_types_recursive(&cond.extends_type, names);
                Self::find_infer_types_recursive(&cond.true_type, names);
                Self::find_infer_types_recursive(&cond.false_type, names);
            }
            Type::IndexedAccess(idx) => {
                Self::find_infer_types_recursive(&idx.object_type, names);
                Self::find_infer_types_recursive(&idx.index_type, names);
            }
            Type::Index(target) => {
                Self::find_infer_types_recursive(target, names);
            }
            _ => {}
        }
    }

    /// Check if a type contains any infer types
    pub fn has_infer_type(type_: &Type) -> bool {
        !Self::find_infer_types(type_).is_empty()
    }

    /// Create a ReturnType<T> style inference
    pub fn infer_return_type(func_type: &Type) -> Option<Arc<Type>> {
        if let Type::Function(func) = func_type {
            if let Some(sig) = func.signatures.first() {
                return Some(sig.return_type.clone());
            }
        }
        None
    }

    /// Create a Parameters<T> style inference (returns tuple of parameter types)
    pub fn infer_parameters(func_type: &Type) -> Option<Arc<Type>> {
        if let Type::Function(func) = func_type {
            if let Some(sig) = func.signatures.first() {
                let param_types: Vec<TupleElement> = sig
                    .parameters
                    .iter()
                    .map(|p| TupleElement {
                        type_: p.type_.clone(),
                        optional: p.optional,
                        label: Some(p.name.clone()),
                    })
                    .collect();

                return Some(Arc::new(Type::Tuple(TupleType {
                    element_types: param_types,
                    min_length: sig.parameters.iter().filter(|p| !p.optional).count(),
                    has_rest: sig.parameters.iter().any(|p| p.rest),
                })));
            }
        }
        None
    }

    /// Create ConstructorParameters<T> style inference
    pub fn infer_constructor_parameters(class_type: &Type) -> Option<Arc<Type>> {
        if let Type::Class(_class) = class_type {
            // Look for construct signature in members
            // This is a simplified version
            return None;
        }
        if let Type::Object(obj) = class_type {
            if let Some(sig) = obj.construct_signatures.first() {
                let param_types: Vec<TupleElement> = sig
                    .parameters
                    .iter()
                    .map(|p| TupleElement {
                        type_: p.type_.clone(),
                        optional: p.optional,
                        label: Some(p.name.clone()),
                    })
                    .collect();

                return Some(Arc::new(Type::Tuple(TupleType {
                    element_types: param_types,
                    min_length: sig.parameters.iter().filter(|p| !p.optional).count(),
                    has_rest: sig.parameters.iter().any(|p| p.rest),
                })));
            }
        }
        None
    }

    /// Create InstanceType<T> style inference
    pub fn infer_instance_type(constructor_type: &Type) -> Option<Arc<Type>> {
        if let Type::Object(obj) = constructor_type {
            if let Some(sig) = obj.construct_signatures.first() {
                return Some(sig.return_type.clone());
            }
        }
        if let Type::Class(_class) = constructor_type {
            // Return the class as the instance type
            return Some(Arc::new(constructor_type.clone()));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distributive_conditional() {
        // Test: type ToArray<T> = T extends any ? T[] : never
        // When T = string | number, should distribute to string[] | number[]

        let conditional = ConditionalType {
            check_type: Arc::new(Type::TypeParameter(TypeParameter {
                name: "T".to_string(),
                constraint: None,
                default: None,
            })),
            extends_type: Arc::new(Type::Any),
            true_type: Arc::new(Type::Array(Arc::new(Type::TypeParameter(TypeParameter {
                name: "T".to_string(),
                constraint: None,
                default: None,
            })))),
            false_type: Arc::new(Type::Never),
        };

        let dist = DistributiveConditional::new(conditional);
        assert!(dist.distribute);

        // Evaluate with union
        let union = Type::Union(vec![Arc::new(Type::String), Arc::new(Type::Number)]);
        let result = dist.evaluate(&union);

        // Should get union of arrays
        if let Type::Union(members) = result.as_ref() {
            assert_eq!(members.len(), 2);
        } else {
            panic!("Expected Union type");
        }
    }

    #[test]
    fn test_deferred_conditional() {
        let conditional = ConditionalType {
            check_type: Arc::new(Type::TypeParameter(TypeParameter {
                name: "T".to_string(),
                constraint: None,
                default: None,
            })),
            extends_type: Arc::new(Type::String),
            true_type: Arc::new(Type::BooleanLiteral(true)),
            false_type: Arc::new(Type::BooleanLiteral(false)),
        };

        let mut deferred = DeferredConditional::new(conditional);

        // Should have unresolved param
        assert!(!deferred.can_resolve());
        assert!(deferred.unresolved_params.contains("T"));

        // Resolve T
        deferred.resolve_param("T", Arc::new(Type::String));
        assert!(deferred.can_resolve());

        // Should resolve to true branch
        let result = deferred.try_resolve().unwrap();
        if let Type::BooleanLiteral(true) = result.as_ref() {
            // Success
        } else {
            panic!("Expected true literal");
        }
    }

    #[test]
    fn test_priority_inference() {
        let mut ctx = PriorityInferenceContext::new();

        // Test priority determination
        assert_eq!(
            PriorityInferenceContext::determine_priority(&Type::String, &Type::String),
            InferencePriority::Exact
        );
        assert_eq!(
            PriorityInferenceContext::determine_priority(
                &Type::StringLiteral("hello".to_string()),
                &Type::String
            ),
            InferencePriority::Subtype
        );
        assert_eq!(
            PriorityInferenceContext::determine_priority(&Type::Any, &Type::String),
            InferencePriority::Fallback
        );
    }

    #[test]
    fn test_recursive_conditional_handler() {
        let mut handler = RecursiveConditionalHandler::new();

        let conditional = ConditionalType {
            check_type: Arc::new(Type::String),
            extends_type: Arc::new(Type::String),
            true_type: Arc::new(Type::BooleanLiteral(true)),
            false_type: Arc::new(Type::BooleanLiteral(false)),
        };

        // Should not be recursive initially
        assert!(!handler.is_recursive(&conditional));

        // Evaluate
        let result = handler.evaluate(&conditional);

        // Should cache result
        assert_eq!(handler.cache.len(), 1);

        // Clear cache
        handler.clear_cache();
        assert_eq!(handler.cache.len(), 0);
    }

    #[test]
    fn test_recursive_depth_limit() {
        let mut handler = RecursiveConditionalHandler::with_max_depth(3);

        let conditional = ConditionalType {
            check_type: Arc::new(Type::String),
            extends_type: Arc::new(Type::String),
            true_type: Arc::new(Type::String),
            false_type: Arc::new(Type::Never),
        };

        // Manually fill recursion stack to test depth limit
        for i in 0..3 {
            handler.recursion_stack.push(format!("level_{}", i));
        }

        assert!(handler.is_too_deep());
        assert!(!handler.enter(&conditional));
    }

    #[test]
    fn test_find_infer_types() {
        let infer_r = Type::TypeParameter(TypeParameter {
            name: "infer R".to_string(),
            constraint: None,
            default: None,
        });

        let infer_p = Type::TypeParameter(TypeParameter {
            name: "infer P".to_string(),
            constraint: None,
            default: None,
        });

        let func = Type::Function(FunctionType {
            signatures: vec![CallSignature {
                type_parameters: vec![],
                parameters: vec![ParameterSignature {
                    name: "x".to_string(),
                    type_: Arc::new(infer_p),
                    optional: false,
                    rest: false,
                }],
                return_type: Arc::new(infer_r),
            }],
        });

        let names = InferUtils::find_infer_types(&func);
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"P".to_string()));
        assert!(names.contains(&"R".to_string()));

        assert!(InferUtils::has_infer_type(&func));
        assert!(!InferUtils::has_infer_type(&Type::String));
    }

    #[test]
    fn test_infer_return_type() {
        let func = Type::Function(FunctionType {
            signatures: vec![CallSignature {
                type_parameters: vec![],
                parameters: vec![],
                return_type: Arc::new(Type::String),
            }],
        });

        let return_type = InferUtils::infer_return_type(&func);
        assert!(return_type.is_some());

        if let Type::String = return_type.unwrap().as_ref() {
            // Success
        } else {
            panic!("Expected String return type");
        }
    }

    #[test]
    fn test_infer_parameters() {
        let func = Type::Function(FunctionType {
            signatures: vec![CallSignature {
                type_parameters: vec![],
                parameters: vec![
                    ParameterSignature {
                        name: "a".to_string(),
                        type_: Arc::new(Type::String),
                        optional: false,
                        rest: false,
                    },
                    ParameterSignature {
                        name: "b".to_string(),
                        type_: Arc::new(Type::Number),
                        optional: true,
                        rest: false,
                    },
                ],
                return_type: Arc::new(Type::Void),
            }],
        });

        let params = InferUtils::infer_parameters(&func);
        assert!(params.is_some());

        if let Type::Tuple(tuple) = params.unwrap().as_ref() {
            assert_eq!(tuple.element_types.len(), 2);
            assert_eq!(tuple.min_length, 1); // Only first param is required
        } else {
            panic!("Expected Tuple type for parameters");
        }
    }

    #[test]
    fn test_infer_instance_type() {
        let constructor = Type::Object(ObjectType {
            properties: HashMap::new(),
            call_signatures: vec![],
            construct_signatures: vec![CallSignature {
                type_parameters: vec![],
                parameters: vec![],
                return_type: Arc::new(Type::Object(ObjectType::default())),
            }],
            index_signatures: vec![],
        });

        let instance = InferUtils::infer_instance_type(&constructor);
        assert!(instance.is_some());
    }

    #[test]
    fn test_simplify_union() {
        // Test removing never types
        let types = vec![
            Arc::new(Type::String),
            Arc::new(Type::Never),
            Arc::new(Type::Number),
        ];

        let simplified = DistributiveConditional::simplify_union(types);

        if let Type::Union(members) = simplified.as_ref() {
            assert_eq!(members.len(), 2);
            // Should not contain Never
            assert!(!members.iter().any(|m| matches!(m.as_ref(), Type::Never)));
        } else {
            panic!("Expected Union type");
        }

        // Test single element becomes non-union
        let single = vec![Arc::new(Type::String), Arc::new(Type::Never)];
        let simplified_single = DistributiveConditional::simplify_union(single);
        if let Type::String = simplified_single.as_ref() {
            // Success
        } else {
            panic!("Expected String type");
        }

        // Test all never becomes never
        let all_never = vec![Arc::new(Type::Never), Arc::new(Type::Never)];
        let simplified_never = DistributiveConditional::simplify_union(all_never);
        if let Type::Never = simplified_never.as_ref() {
            // Success
        } else {
            panic!("Expected Never type");
        }
    }
}
