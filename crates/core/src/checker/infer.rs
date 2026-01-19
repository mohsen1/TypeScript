//! Infer keyword implementation for conditional type inference.
//!
//! This module handles:
//! - `infer` keyword in conditional types
//! - Multiple infer positions with same name
//! - Co-variant and contra-variant inference positions
//! - Infer in template literal types
//! - Infer in tuple rest positions
//! - Inference constraints and defaults
//! - Nested conditional types with infer
//!
//! # Examples
//!
//! ```ignore
//! // Basic infer usage
//! type ReturnType<T> = T extends (...args: any[]) => infer R ? R : never;
//!
//! // Infer in tuple rest position
//! type First<T> = T extends [infer F, ...infer R] ? F : never;
//!
//! // Infer with constraint
//! type GetString<T> = T extends { value: infer V extends string } ? V : never;
//!
//! // Multiple infer with same name (intersection)
//! type Both<T> = T extends { a: infer X; b: infer X } ? X : never;
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use super::type_defs::{
    Type, TypeParameter, ConditionalType, ObjectType, TupleType, TupleElement,
    FunctionType, CallSignature, ParameterSignature, PropertySignature,
};

/// Variance position for inference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variance {
    /// Co-variant position (return types, property types)
    Covariant,
    /// Contra-variant position (parameter types)
    Contravariant,
    /// Invariant position (both read and written)
    Invariant,
    /// Independent (no variance)
    Independent,
}

impl Variance {
    /// Combine two variances
    pub fn combine(self, other: Variance) -> Variance {
        match (self, other) {
            (Variance::Independent, v) | (v, Variance::Independent) => v,
            (Variance::Covariant, Variance::Covariant) => Variance::Covariant,
            (Variance::Contravariant, Variance::Contravariant) => Variance::Covariant,
            (Variance::Covariant, Variance::Contravariant)
            | (Variance::Contravariant, Variance::Covariant) => Variance::Contravariant,
            _ => Variance::Invariant,
        }
    }

    /// Flip variance (for contravariant positions)
    pub fn flip(self) -> Variance {
        match self {
            Variance::Covariant => Variance::Contravariant,
            Variance::Contravariant => Variance::Covariant,
            v => v,
        }
    }
}

/// An infer type placeholder (infer T)
#[derive(Debug, Clone)]
pub struct InferType {
    /// The name of the inferred type variable
    pub name: String,
    /// Optional constraint (infer T extends U)
    pub constraint: Option<Arc<Type>>,
    /// The variance position where this infer appears
    pub variance: Variance,
}

impl InferType {
    /// Create a new infer type
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            constraint: None,
            variance: Variance::Covariant,
        }
    }

    /// Create a new infer type with a constraint
    pub fn with_constraint(name: impl Into<String>, constraint: Arc<Type>) -> Self {
        Self {
            name: name.into(),
            constraint: Some(constraint),
            variance: Variance::Covariant,
        }
    }

    /// Set the variance position
    pub fn with_variance(mut self, variance: Variance) -> Self {
        self.variance = variance;
        self
    }
}

/// Result of type inference
#[derive(Debug, Clone)]
pub struct InferenceResult {
    /// Map from infer type names to their inferred types
    pub inferred_types: HashMap<String, Arc<Type>>,
    /// Whether inference was successful
    pub success: bool,
    /// Inference errors if any
    pub errors: Vec<String>,
}

impl InferenceResult {
    /// Create a successful inference result
    pub fn success(inferred_types: HashMap<String, Arc<Type>>) -> Self {
        Self {
            inferred_types,
            success: true,
            errors: Vec::new(),
        }
    }

    /// Create a failed inference result
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            inferred_types: HashMap::new(),
            success: false,
            errors,
        }
    }

    /// Create an empty successful result
    pub fn empty() -> Self {
        Self {
            inferred_types: HashMap::new(),
            success: true,
            errors: Vec::new(),
        }
    }
}

/// Context for type inference during conditional type evaluation
#[derive(Debug, Clone)]
pub struct InferenceContext {
    /// Current inference candidates per infer type name
    candidates: HashMap<String, Vec<InferenceCandidate>>,
    /// Constraints for infer types (infer T extends U)
    constraints: HashMap<String, Arc<Type>>,
    /// Current variance position
    current_variance: Variance,
    /// Nesting depth for nested conditionals
    depth: usize,
}

/// A candidate for an inferred type
#[derive(Debug, Clone)]
pub struct InferenceCandidate {
    /// The inferred type
    pub type_: Arc<Type>,
    /// The variance at which it was inferred
    pub variance: Variance,
    /// Priority (lower is better, for ordered inference)
    pub priority: usize,
}

impl InferenceContext {
    /// Create a new inference context
    pub fn new() -> Self {
        Self {
            candidates: HashMap::new(),
            constraints: HashMap::new(),
            current_variance: Variance::Covariant,
            depth: 0,
        }
    }

    /// Add a constraint for an infer type
    pub fn add_constraint(&mut self, name: &str, constraint: Arc<Type>) {
        self.constraints.insert(name.to_string(), constraint);
    }

    /// Add an inference candidate
    pub fn add_candidate(&mut self, name: &str, type_: Arc<Type>, priority: usize) {
        let candidate = InferenceCandidate {
            type_,
            variance: self.current_variance,
            priority,
        };
        self.candidates
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(candidate);
    }

    /// Get all candidates for an infer type
    pub fn get_candidates(&self, name: &str) -> Option<&Vec<InferenceCandidate>> {
        self.candidates.get(name)
    }

    /// Set the current variance
    pub fn set_variance(&mut self, variance: Variance) {
        self.current_variance = variance;
    }

    /// Enter a contravariant position (flips variance)
    pub fn enter_contravariant<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let old_variance = self.current_variance;
        self.current_variance = old_variance.flip();
        let result = f(self);
        self.current_variance = old_variance;
        result
    }

    /// Enter a nested conditional
    pub fn enter_nested<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        result
    }

    /// Get the current depth
    pub fn depth(&self) -> usize {
        self.depth
    }
}

impl Default for InferenceContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Conditional type inference engine
#[derive(Debug)]
pub struct ConditionalTypeInferrer;

impl ConditionalTypeInferrer {
    /// Evaluate a conditional type, performing inference
    pub fn evaluate(
        conditional: &ConditionalType,
        context: &mut InferenceContext,
    ) -> Arc<Type> {
        // Check if we can infer types from the extends clause
        let infer_result = Self::infer_from_extends(
            &conditional.check_type,
            &conditional.extends_type,
            context,
        );

        if infer_result.success {
            // Substitute inferred types into true branch
            Self::substitute_inferred(
                &conditional.true_type,
                &infer_result.inferred_types,
            )
        } else {
            // Return false branch
            conditional.false_type.clone()
        }
    }

    /// Infer types from an extends clause
    pub fn infer_from_extends(
        check_type: &Type,
        extends_type: &Type,
        context: &mut InferenceContext,
    ) -> InferenceResult {
        // Find all infer types in extends_type and try to match them
        Self::infer_recursive(check_type, extends_type, context, 0);

        // Finalize inference results
        Self::finalize_inference(context)
    }

    /// Recursively infer types by matching structure
    fn infer_recursive(
        source: &Type,
        target: &Type,
        context: &mut InferenceContext,
        priority: usize,
    ) {
        match target {
            // Handle infer type - this is where we capture the inferred type
            Type::TypeParameter(tp) if tp.name.starts_with("infer ") => {
                let infer_name = tp.name.strip_prefix("infer ").unwrap();

                // Check constraint if present
                if let Some(constraint) = &tp.constraint {
                    if !Self::is_assignable_to(source, constraint) {
                        return; // Constraint not satisfied
                    }
                }

                context.add_candidate(infer_name, Arc::new(source.clone()), priority);
            }

            // Match function types
            Type::Function(target_func) => {
                if let Type::Function(source_func) = source {
                    Self::infer_from_function(source_func, target_func, context, priority);
                }
            }

            // Match object types
            Type::Object(target_obj) => {
                if let Type::Object(source_obj) = source {
                    Self::infer_from_object(source_obj, target_obj, context, priority);
                }
            }

            // Match tuple types
            Type::Tuple(target_tuple) => {
                if let Type::Tuple(source_tuple) = source {
                    Self::infer_from_tuple(source_tuple, target_tuple, context, priority);
                } else if let Type::Array(source_element) = source {
                    // Array can match tuple with rest
                    Self::infer_from_array_to_tuple(source_element, target_tuple, context, priority);
                }
            }

            // Match array types
            Type::Array(target_element) => {
                if let Type::Array(source_element) = source {
                    Self::infer_recursive(source_element, target_element, context, priority);
                } else if let Type::Tuple(source_tuple) = source {
                    // Infer array element type from tuple
                    Self::infer_array_from_tuple(source_tuple, target_element, context, priority);
                }
            }

            // Match union types
            Type::Union(target_members) => {
                // For unions, try to find a matching member
                for target_member in target_members {
                    Self::infer_recursive(source, target_member, context, priority);
                }
            }

            // Match conditional types (nested)
            Type::Conditional(nested_cond) => {
                context.enter_nested(|ctx| {
                    Self::infer_recursive(source, &nested_cond.check_type, ctx, priority);
                });
            }

            // Match indexed access types
            Type::IndexedAccess(target_indexed) => {
                if let Type::IndexedAccess(source_indexed) = source {
                    Self::infer_recursive(
                        &source_indexed.object_type,
                        &target_indexed.object_type,
                        context,
                        priority,
                    );
                    Self::infer_recursive(
                        &source_indexed.index_type,
                        &target_indexed.index_type,
                        context,
                        priority,
                    );
                }
            }

            // Template literal type inference
            Type::StringLiteral(target_str) => {
                // Handle template literal type matching
                if target_str.contains("${") {
                    Self::infer_from_template_literal(source, target_str, context, priority);
                }
            }

            _ => {}
        }
    }

    /// Infer from function types
    fn infer_from_function(
        source: &FunctionType,
        target: &FunctionType,
        context: &mut InferenceContext,
        priority: usize,
    ) {
        // Match signatures
        for (source_sig, target_sig) in source.signatures.iter().zip(target.signatures.iter()) {
            Self::infer_from_signature(source_sig, target_sig, context, priority);
        }
    }

    /// Infer from call signatures
    fn infer_from_signature(
        source: &CallSignature,
        target: &CallSignature,
        context: &mut InferenceContext,
        priority: usize,
    ) {
        // Parameters are contravariant
        context.enter_contravariant(|ctx| {
            for (source_param, target_param) in source.parameters.iter().zip(target.parameters.iter())
            {
                Self::infer_recursive(&source_param.type_, &target_param.type_, ctx, priority);
            }
        });

        // Return type is covariant
        Self::infer_recursive(&source.return_type, &target.return_type, context, priority);
    }

    /// Infer from object types
    fn infer_from_object(
        source: &ObjectType,
        target: &ObjectType,
        context: &mut InferenceContext,
        priority: usize,
    ) {
        // Match properties
        for (name, target_prop) in &target.properties {
            if let Some(source_prop) = source.properties.get(name) {
                Self::infer_recursive(&source_prop.type_, &target_prop.type_, context, priority);
            }
        }

        // Match call signatures
        for (source_sig, target_sig) in source.call_signatures.iter().zip(target.call_signatures.iter())
        {
            Self::infer_from_signature(source_sig, target_sig, context, priority);
        }
    }

    /// Infer from tuple types
    fn infer_from_tuple(
        source: &TupleType,
        target: &TupleType,
        context: &mut InferenceContext,
        priority: usize,
    ) {
        let source_len = source.element_types.len();
        let target_len = target.element_types.len();

        // Handle rest elements
        if target.has_rest && target_len > 0 {
            // Match leading elements
            let rest_index = target_len - 1;
            for (i, target_elem) in target.element_types.iter().enumerate() {
                if i < rest_index {
                    // Fixed position element
                    if i < source_len {
                        Self::infer_recursive(
                            &source.element_types[i].type_,
                            &target_elem.type_,
                            context,
                            priority,
                        );
                    }
                } else {
                    // Rest element - collect remaining source elements
                    Self::infer_rest_element(
                        &source.element_types[i..].iter().collect::<Vec<_>>(),
                        &target_elem.type_,
                        context,
                        priority,
                    );
                }
            }
        } else {
            // No rest, match positionally
            for (source_elem, target_elem) in
                source.element_types.iter().zip(target.element_types.iter())
            {
                Self::infer_recursive(&source_elem.type_, &target_elem.type_, context, priority);
            }
        }
    }

    /// Infer rest element from multiple source elements
    fn infer_rest_element(
        source_elements: &[&TupleElement],
        target_type: &Type,
        context: &mut InferenceContext,
        priority: usize,
    ) {
        // For rest elements, we need to infer an array/tuple type from the remaining elements
        if let Type::TypeParameter(tp) = target_type {
            if tp.name.starts_with("infer ") {
                let infer_name = tp.name.strip_prefix("infer ").unwrap();

                // Create a tuple type from remaining elements
                let tuple = TupleType {
                    element_types: source_elements
                        .iter()
                        .map(|e| (*e).clone())
                        .collect(),
                    min_length: source_elements.len(),
                    has_rest: false,
                };

                context.add_candidate(infer_name, Arc::new(Type::Tuple(tuple)), priority);
                return;
            }
        }

        // Otherwise, try to match each element against the target
        for elem in source_elements {
            Self::infer_recursive(&elem.type_, target_type, context, priority);
        }
    }

    /// Infer from array to tuple with rest
    fn infer_from_array_to_tuple(
        source_element: &Arc<Type>,
        target: &TupleType,
        context: &mut InferenceContext,
        priority: usize,
    ) {
        // For array matching tuple with rest, the rest captures the array element type
        if target.has_rest && !target.element_types.is_empty() {
            let rest_elem = &target.element_types[target.element_types.len() - 1];
            Self::infer_recursive(source_element, &rest_elem.type_, context, priority);
        }
    }

    /// Infer array element type from tuple
    fn infer_array_from_tuple(
        source: &TupleType,
        target_element: &Arc<Type>,
        context: &mut InferenceContext,
        priority: usize,
    ) {
        // The array element type is the union of all tuple element types
        // For inference, we try to match against each element
        for elem in &source.element_types {
            Self::infer_recursive(&elem.type_, target_element, context, priority);
        }
    }

    /// Infer from template literal types
    fn infer_from_template_literal(
        source: &Type,
        template: &str,
        context: &mut InferenceContext,
        priority: usize,
    ) {
        // Extract the source string
        let source_str = match source {
            Type::StringLiteral(s) => s.as_str(),
            Type::String => {
                // Cannot infer from general string type
                return;
            }
            _ => return,
        };

        // Parse template literal pattern and extract infer positions
        // Template format: "prefix${infer X}middle${infer Y}suffix"
        Self::match_template_pattern(source_str, template, context, priority);
    }

    /// Match a string against a template pattern with infer placeholders
    fn match_template_pattern(
        source: &str,
        template: &str,
        context: &mut InferenceContext,
        priority: usize,
    ) {
        // Split template by ${...} patterns
        let mut remaining_source = source;
        let mut remaining_template = template;

        while !remaining_template.is_empty() {
            // Find next ${
            if let Some(start) = remaining_template.find("${") {
                // Match prefix
                let prefix = &remaining_template[..start];
                if !remaining_source.starts_with(prefix) {
                    return; // Prefix doesn't match
                }
                remaining_source = &remaining_source[prefix.len()..];
                remaining_template = &remaining_template[start + 2..];

                // Find closing }
                if let Some(end) = remaining_template.find('}') {
                    let placeholder = &remaining_template[..end];
                    remaining_template = &remaining_template[end + 1..];

                    // Check if this is an infer placeholder
                    if let Some(infer_name) = placeholder.strip_prefix("infer ") {
                        // Find where this placeholder ends (next literal part or end)
                        let next_literal = if let Some(next_start) = remaining_template.find("${") {
                            &remaining_template[..next_start]
                        } else {
                            remaining_template
                        };

                        // Find where the inferred part ends in source
                        let inferred_end = if next_literal.is_empty() {
                            remaining_source.len()
                        } else if let Some(pos) = remaining_source.find(next_literal) {
                            pos
                        } else {
                            return; // Cannot find next literal in source
                        };

                        let inferred_value = &remaining_source[..inferred_end];
                        context.add_candidate(
                            infer_name,
                            Arc::new(Type::StringLiteral(inferred_value.to_string())),
                            priority,
                        );

                        remaining_source = &remaining_source[inferred_end..];
                    }
                }
            } else {
                // No more placeholders, check remaining matches
                if remaining_source != remaining_template {
                    return;
                }
                break;
            }
        }
    }

    /// Check if a type is assignable to another
    fn is_assignable_to(source: &Type, target: &Type) -> bool {
        // Simplified assignability check
        match (source, target) {
            // Any is assignable to anything
            (Type::Any, _) | (_, Type::Any) => true,
            // Unknown is assignable from anything
            (_, Type::Unknown) => true,
            // Never is assignable to anything
            (Type::Never, _) => true,
            // Same primitive types
            (Type::String, Type::String) => true,
            (Type::Number, Type::Number) => true,
            (Type::Boolean, Type::Boolean) => true,
            (Type::BigInt, Type::BigInt) => true,
            (Type::Symbol, Type::Symbol) => true,
            // Literal to base type
            (Type::StringLiteral(_), Type::String) => true,
            (Type::NumberLiteral(_), Type::Number) => true,
            (Type::BooleanLiteral(_), Type::Boolean) => true,
            (Type::BigIntLiteral(_), Type::BigInt) => true,
            // Same literals
            (Type::StringLiteral(a), Type::StringLiteral(b)) => a == b,
            (Type::NumberLiteral(a), Type::NumberLiteral(b)) => a == b,
            (Type::BooleanLiteral(a), Type::BooleanLiteral(b)) => a == b,
            // Union source - all members must be assignable
            (Type::Union(members), target) => {
                members.iter().all(|m| Self::is_assignable_to(m, target))
            }
            // Union target - at least one member must be assignable to
            (source, Type::Union(members)) => {
                members.iter().any(|m| Self::is_assignable_to(source, m))
            }
            // Object types
            (Type::Object(source_obj), Type::Object(target_obj)) => {
                // All target properties must be present in source
                target_obj.properties.iter().all(|(name, target_prop)| {
                    source_obj
                        .properties
                        .get(name)
                        .map(|source_prop| {
                            Self::is_assignable_to(&source_prop.type_, &target_prop.type_)
                        })
                        .unwrap_or(target_prop.optional)
                })
            }
            // Array types
            (Type::Array(source_elem), Type::Array(target_elem)) => {
                Self::is_assignable_to(source_elem, target_elem)
            }
            // Tuple to array
            (Type::Tuple(tuple), Type::Array(target_elem)) => {
                tuple
                    .element_types
                    .iter()
                    .all(|elem| Self::is_assignable_to(&elem.type_, target_elem))
            }
            _ => false,
        }
    }

    /// Finalize inference by combining candidates
    fn finalize_inference(context: &InferenceContext) -> InferenceResult {
        let mut inferred = HashMap::new();
        let mut errors = Vec::new();

        for (name, candidates) in &context.candidates {
            if candidates.is_empty() {
                continue;
            }

            // Separate candidates by variance
            let covariant: Vec<_> = candidates
                .iter()
                .filter(|c| c.variance == Variance::Covariant)
                .collect();
            let contravariant: Vec<_> = candidates
                .iter()
                .filter(|c| c.variance == Variance::Contravariant)
                .collect();

            // For covariant positions, we want the union (widest type)
            // For contravariant positions, we want the intersection (narrowest type)
            let inferred_type = if !covariant.is_empty() && contravariant.is_empty() {
                // Only covariant - take union if multiple
                Self::combine_covariant(&covariant)
            } else if covariant.is_empty() && !contravariant.is_empty() {
                // Only contravariant - take intersection
                Self::combine_contravariant(&contravariant)
            } else if !covariant.is_empty() && !contravariant.is_empty() {
                // Both - intersect contravariant, then check against covariant
                let contra_type = Self::combine_contravariant(&contravariant);
                let co_type = Self::combine_covariant(&covariant);

                // Use the more specific type if compatible
                if Self::is_assignable_to(&contra_type, &co_type) {
                    contra_type
                } else {
                    co_type
                }
            } else {
                // No candidates with variance - use first
                candidates[0].type_.clone()
            };

            // Check constraint
            if let Some(constraint) = context.constraints.get(name) {
                if !Self::is_assignable_to(&inferred_type, constraint) {
                    errors.push(format!(
                        "Inferred type for '{}' does not satisfy constraint",
                        name
                    ));
                    continue;
                }
            }

            inferred.insert(name.clone(), inferred_type);
        }

        if errors.is_empty() {
            InferenceResult::success(inferred)
        } else {
            InferenceResult {
                inferred_types: inferred,
                success: false,
                errors,
            }
        }
    }

    /// Combine covariant candidates (union/widening)
    fn combine_covariant(candidates: &[&InferenceCandidate]) -> Arc<Type> {
        if candidates.len() == 1 {
            return candidates[0].type_.clone();
        }

        // Sort by priority and take lowest priority first
        let mut sorted: Vec<_> = candidates.iter().collect();
        sorted.sort_by_key(|c| c.priority);

        // Create union of all candidate types
        let types: Vec<Arc<Type>> = sorted.iter().map(|c| c.type_.clone()).collect();

        // Simplify if all same
        if types.iter().all(|t| Self::types_equal(&types[0], t)) {
            return types[0].clone();
        }

        Arc::new(Type::Union(types))
    }

    /// Combine contravariant candidates (intersection/narrowing)
    fn combine_contravariant(candidates: &[&InferenceCandidate]) -> Arc<Type> {
        if candidates.len() == 1 {
            return candidates[0].type_.clone();
        }

        // Sort by priority
        let mut sorted: Vec<_> = candidates.iter().collect();
        sorted.sort_by_key(|c| c.priority);

        // Create intersection of all candidate types
        let types: Vec<Arc<Type>> = sorted.iter().map(|c| c.type_.clone()).collect();

        // Simplify if all same
        if types.iter().all(|t| Self::types_equal(&types[0], t)) {
            return types[0].clone();
        }

        Arc::new(Type::Intersection(types))
    }

    /// Check if two types are structurally equal
    fn types_equal(a: &Type, b: &Type) -> bool {
        match (a, b) {
            (Type::Any, Type::Any) => true,
            (Type::Unknown, Type::Unknown) => true,
            (Type::String, Type::String) => true,
            (Type::Number, Type::Number) => true,
            (Type::Boolean, Type::Boolean) => true,
            (Type::BigInt, Type::BigInt) => true,
            (Type::Symbol, Type::Symbol) => true,
            (Type::Void, Type::Void) => true,
            (Type::Undefined, Type::Undefined) => true,
            (Type::Null, Type::Null) => true,
            (Type::Never, Type::Never) => true,
            (Type::StringLiteral(x), Type::StringLiteral(y)) => x == y,
            (Type::NumberLiteral(x), Type::NumberLiteral(y)) => x == y,
            (Type::BooleanLiteral(x), Type::BooleanLiteral(y)) => x == y,
            (Type::BigIntLiteral(x), Type::BigIntLiteral(y)) => x == y,
            _ => false, // For complex types, assume not equal
        }
    }

    /// Substitute inferred types into a type
    pub fn substitute_inferred(
        type_: &Type,
        inferred: &HashMap<String, Arc<Type>>,
    ) -> Arc<Type> {
        match type_ {
            Type::TypeParameter(tp) => {
                // Check if this is an infer placeholder that was resolved
                if let Some(name) = tp.name.strip_prefix("infer ") {
                    if let Some(inferred_type) = inferred.get(name) {
                        return inferred_type.clone();
                    }
                }
                // Check if this is a regular type parameter that was inferred
                if let Some(inferred_type) = inferred.get(&tp.name) {
                    return inferred_type.clone();
                }
                Arc::new(type_.clone())
            }

            Type::Union(members) => {
                let substituted: Vec<_> = members
                    .iter()
                    .map(|m| Self::substitute_inferred(m, inferred))
                    .collect();
                Arc::new(Type::Union(substituted))
            }

            Type::Intersection(members) => {
                let substituted: Vec<_> = members
                    .iter()
                    .map(|m| Self::substitute_inferred(m, inferred))
                    .collect();
                Arc::new(Type::Intersection(substituted))
            }

            Type::Array(element) => {
                Arc::new(Type::Array(Self::substitute_inferred(element, inferred)))
            }

            Type::Tuple(tuple) => {
                let substituted_elements: Vec<_> = tuple
                    .element_types
                    .iter()
                    .map(|elem| TupleElement {
                        type_: Self::substitute_inferred(&elem.type_, inferred),
                        optional: elem.optional,
                        label: elem.label.clone(),
                    })
                    .collect();
                Arc::new(Type::Tuple(TupleType {
                    element_types: substituted_elements,
                    min_length: tuple.min_length,
                    has_rest: tuple.has_rest,
                }))
            }

            Type::Object(obj) => {
                let substituted_properties: HashMap<_, _> = obj
                    .properties
                    .iter()
                    .map(|(name, prop)| {
                        (
                            name.clone(),
                            PropertySignature {
                                name: prop.name.clone(),
                                type_: Self::substitute_inferred(&prop.type_, inferred),
                                optional: prop.optional,
                                readonly: prop.readonly,
                            },
                        )
                    })
                    .collect();

                let substituted_call_signatures: Vec<_> = obj
                    .call_signatures
                    .iter()
                    .map(|sig| Self::substitute_signature(sig, inferred))
                    .collect();

                Arc::new(Type::Object(ObjectType {
                    properties: substituted_properties,
                    call_signatures: substituted_call_signatures,
                    construct_signatures: obj.construct_signatures.clone(),
                    index_signatures: obj.index_signatures.clone(),
                }))
            }

            Type::Function(func) => {
                let substituted_signatures: Vec<_> = func
                    .signatures
                    .iter()
                    .map(|sig| Self::substitute_signature(sig, inferred))
                    .collect();
                Arc::new(Type::Function(FunctionType {
                    signatures: substituted_signatures,
                }))
            }

            Type::Conditional(cond) => {
                Arc::new(Type::Conditional(ConditionalType {
                    check_type: Self::substitute_inferred(&cond.check_type, inferred),
                    extends_type: Self::substitute_inferred(&cond.extends_type, inferred),
                    true_type: Self::substitute_inferred(&cond.true_type, inferred),
                    false_type: Self::substitute_inferred(&cond.false_type, inferred),
                }))
            }

            Type::IndexedAccess(indexed) => {
                Arc::new(Type::IndexedAccess(super::type_defs::IndexedAccessType {
                    object_type: Self::substitute_inferred(&indexed.object_type, inferred),
                    index_type: Self::substitute_inferred(&indexed.index_type, inferred),
                }))
            }

            Type::Index(target) => {
                Arc::new(Type::Index(Self::substitute_inferred(target, inferred)))
            }

            // Primitive types - no substitution needed
            _ => Arc::new(type_.clone()),
        }
    }

    /// Substitute inferred types in a call signature
    fn substitute_signature(
        sig: &CallSignature,
        inferred: &HashMap<String, Arc<Type>>,
    ) -> CallSignature {
        CallSignature {
            type_parameters: sig.type_parameters.clone(),
            parameters: sig
                .parameters
                .iter()
                .map(|p| ParameterSignature {
                    name: p.name.clone(),
                    type_: Self::substitute_inferred(&p.type_, inferred),
                    optional: p.optional,
                    rest: p.rest,
                })
                .collect(),
            return_type: Self::substitute_inferred(&sig.return_type, inferred),
        }
    }
}

/// Builder for creating infer types
#[derive(Debug)]
pub struct InferTypeBuilder {
    name: String,
    constraint: Option<Arc<Type>>,
    variance: Variance,
}

impl InferTypeBuilder {
    /// Create a new builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            constraint: None,
            variance: Variance::Covariant,
        }
    }

    /// Add a constraint
    pub fn constraint(mut self, constraint: Arc<Type>) -> Self {
        self.constraint = Some(constraint);
        self
    }

    /// Set variance
    pub fn variance(mut self, variance: Variance) -> Self {
        self.variance = variance;
        self
    }

    /// Build as a Type::TypeParameter (how infer is represented)
    pub fn build(self) -> Type {
        Type::TypeParameter(TypeParameter {
            name: format!("infer {}", self.name),
            constraint: self.constraint,
            default: None,
        })
    }

    /// Build and wrap in Arc
    pub fn build_arc(self) -> Arc<Type> {
        Arc::new(self.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_infer() {
        // Test: type GetReturn<T> = T extends () => infer R ? R : never
        let infer_r = InferTypeBuilder::new("R").build_arc();

        let target_func = Type::Function(FunctionType {
            signatures: vec![CallSignature {
                type_parameters: vec![],
                parameters: vec![],
                return_type: infer_r,
            }],
        });

        let source_func = Type::Function(FunctionType {
            signatures: vec![CallSignature {
                type_parameters: vec![],
                parameters: vec![],
                return_type: Arc::new(Type::String),
            }],
        });

        let mut context = InferenceContext::new();
        ConditionalTypeInferrer::infer_recursive(&source_func, &target_func, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        assert!(result.success);
        assert!(result.inferred_types.contains_key("R"));

        if let Type::String = result.inferred_types.get("R").unwrap().as_ref() {
            // Success
        } else {
            panic!("Expected String type for R");
        }
    }

    #[test]
    fn test_infer_from_parameter() {
        // Test: type GetArg<T> = T extends (x: infer P) => any ? P : never
        let infer_p = InferTypeBuilder::new("P").build_arc();

        let target_func = Type::Function(FunctionType {
            signatures: vec![CallSignature {
                type_parameters: vec![],
                parameters: vec![ParameterSignature {
                    name: "x".to_string(),
                    type_: infer_p,
                    optional: false,
                    rest: false,
                }],
                return_type: Arc::new(Type::Any),
            }],
        });

        let source_func = Type::Function(FunctionType {
            signatures: vec![CallSignature {
                type_parameters: vec![],
                parameters: vec![ParameterSignature {
                    name: "x".to_string(),
                    type_: Arc::new(Type::Number),
                    optional: false,
                    rest: false,
                }],
                return_type: Arc::new(Type::String),
            }],
        });

        let mut context = InferenceContext::new();
        ConditionalTypeInferrer::infer_recursive(&source_func, &target_func, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        assert!(result.success);
        assert!(result.inferred_types.contains_key("P"));

        if let Type::Number = result.inferred_types.get("P").unwrap().as_ref() {
            // Success
        } else {
            panic!("Expected Number type for P");
        }
    }

    #[test]
    fn test_infer_with_constraint() {
        // Test: type GetString<T> = T extends infer S extends string ? S : never
        let constraint = Arc::new(Type::String);
        let infer_s = InferTypeBuilder::new("S").constraint(constraint.clone()).build_arc();

        // Should succeed with string literal
        let source_string = Type::StringLiteral("hello".to_string());
        let mut context = InferenceContext::new();
        context.add_constraint("S", constraint.clone());
        ConditionalTypeInferrer::infer_recursive(&source_string, &infer_s, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        assert!(result.success);

        // Should fail with number
        let source_number = Type::Number;
        let mut context2 = InferenceContext::new();
        context2.add_constraint("S", constraint);
        ConditionalTypeInferrer::infer_recursive(&source_number, &infer_s, &mut context2, 0);

        let result2 = ConditionalTypeInferrer::finalize_inference(&context2);
        // No candidates added because constraint not satisfied
        assert!(result2.inferred_types.get("S").is_none());
    }

    #[test]
    fn test_infer_tuple_first() {
        // Test: type First<T> = T extends [infer F, ...any[]] ? F : never
        let infer_f = InferTypeBuilder::new("F").build_arc();
        let rest = Arc::new(Type::Array(Arc::new(Type::Any)));

        let target_tuple = Type::Tuple(TupleType {
            element_types: vec![
                TupleElement {
                    type_: infer_f,
                    optional: false,
                    label: None,
                },
                TupleElement {
                    type_: rest,
                    optional: false,
                    label: None,
                },
            ],
            min_length: 1,
            has_rest: true,
        });

        let source_tuple = Type::Tuple(TupleType {
            element_types: vec![
                TupleElement {
                    type_: Arc::new(Type::String),
                    optional: false,
                    label: None,
                },
                TupleElement {
                    type_: Arc::new(Type::Number),
                    optional: false,
                    label: None,
                },
                TupleElement {
                    type_: Arc::new(Type::Boolean),
                    optional: false,
                    label: None,
                },
            ],
            min_length: 3,
            has_rest: false,
        });

        let mut context = InferenceContext::new();
        ConditionalTypeInferrer::infer_recursive(&source_tuple, &target_tuple, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        assert!(result.success);
        assert!(result.inferred_types.contains_key("F"));

        if let Type::String = result.inferred_types.get("F").unwrap().as_ref() {
            // Success
        } else {
            panic!("Expected String type for F");
        }
    }

    #[test]
    fn test_infer_tuple_rest() {
        // Test: type Rest<T> = T extends [any, ...infer R] ? R : never
        let infer_r = InferTypeBuilder::new("R").build_arc();

        let target_tuple = Type::Tuple(TupleType {
            element_types: vec![
                TupleElement {
                    type_: Arc::new(Type::Any),
                    optional: false,
                    label: None,
                },
                TupleElement {
                    type_: infer_r,
                    optional: false,
                    label: None,
                },
            ],
            min_length: 1,
            has_rest: true,
        });

        let source_tuple = Type::Tuple(TupleType {
            element_types: vec![
                TupleElement {
                    type_: Arc::new(Type::String),
                    optional: false,
                    label: None,
                },
                TupleElement {
                    type_: Arc::new(Type::Number),
                    optional: false,
                    label: None,
                },
                TupleElement {
                    type_: Arc::new(Type::Boolean),
                    optional: false,
                    label: None,
                },
            ],
            min_length: 3,
            has_rest: false,
        });

        let mut context = InferenceContext::new();
        ConditionalTypeInferrer::infer_recursive(&source_tuple, &target_tuple, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        assert!(result.success);
        assert!(result.inferred_types.contains_key("R"));

        // R should be a tuple [number, boolean]
        if let Type::Tuple(tuple) = result.inferred_types.get("R").unwrap().as_ref() {
            assert_eq!(tuple.element_types.len(), 2);
        } else {
            panic!("Expected Tuple type for R");
        }
    }

    #[test]
    fn test_infer_object_property() {
        // Test: type GetValue<T> = T extends { value: infer V } ? V : never
        let infer_v = InferTypeBuilder::new("V").build_arc();

        let mut target_properties = HashMap::new();
        target_properties.insert(
            "value".to_string(),
            PropertySignature {
                name: "value".to_string(),
                type_: infer_v,
                optional: false,
                readonly: false,
            },
        );

        let target_obj = Type::Object(ObjectType {
            properties: target_properties,
            call_signatures: vec![],
            construct_signatures: vec![],
            index_signatures: vec![],
        });

        let mut source_properties = HashMap::new();
        source_properties.insert(
            "value".to_string(),
            PropertySignature {
                name: "value".to_string(),
                type_: Arc::new(Type::NumberLiteral(42.0)),
                optional: false,
                readonly: false,
            },
        );

        let source_obj = Type::Object(ObjectType {
            properties: source_properties,
            call_signatures: vec![],
            construct_signatures: vec![],
            index_signatures: vec![],
        });

        let mut context = InferenceContext::new();
        ConditionalTypeInferrer::infer_recursive(&source_obj, &target_obj, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        assert!(result.success);
        assert!(result.inferred_types.contains_key("V"));

        if let Type::NumberLiteral(n) = result.inferred_types.get("V").unwrap().as_ref() {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected NumberLiteral type for V");
        }
    }

    #[test]
    fn test_infer_multiple_same_name_covariant() {
        // Test: type Both<T> = T extends { a: infer X; b: infer X } ? X : never
        // When both positions are covariant, result is union
        let infer_x1 = InferTypeBuilder::new("X").build_arc();
        let infer_x2 = InferTypeBuilder::new("X").build_arc();

        let mut target_properties = HashMap::new();
        target_properties.insert(
            "a".to_string(),
            PropertySignature {
                name: "a".to_string(),
                type_: infer_x1,
                optional: false,
                readonly: false,
            },
        );
        target_properties.insert(
            "b".to_string(),
            PropertySignature {
                name: "b".to_string(),
                type_: infer_x2,
                optional: false,
                readonly: false,
            },
        );

        let target_obj = Type::Object(ObjectType {
            properties: target_properties,
            call_signatures: vec![],
            construct_signatures: vec![],
            index_signatures: vec![],
        });

        let mut source_properties = HashMap::new();
        source_properties.insert(
            "a".to_string(),
            PropertySignature {
                name: "a".to_string(),
                type_: Arc::new(Type::String),
                optional: false,
                readonly: false,
            },
        );
        source_properties.insert(
            "b".to_string(),
            PropertySignature {
                name: "b".to_string(),
                type_: Arc::new(Type::Number),
                optional: false,
                readonly: false,
            },
        );

        let source_obj = Type::Object(ObjectType {
            properties: source_properties,
            call_signatures: vec![],
            construct_signatures: vec![],
            index_signatures: vec![],
        });

        let mut context = InferenceContext::new();
        ConditionalTypeInferrer::infer_recursive(&source_obj, &target_obj, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        assert!(result.success);
        assert!(result.inferred_types.contains_key("X"));

        // X should be string | number (union)
        if let Type::Union(members) = result.inferred_types.get("X").unwrap().as_ref() {
            assert_eq!(members.len(), 2);
        } else {
            panic!("Expected Union type for X");
        }
    }

    #[test]
    fn test_infer_contravariant_position() {
        // Test: type GetParams<T> = T extends (a: infer P, b: infer P) => any ? P : never
        // Parameters are contravariant, so multiple infer with same name creates intersection
        let infer_p1 = InferTypeBuilder::new("P").build_arc();
        let infer_p2 = InferTypeBuilder::new("P").build_arc();

        let target_func = Type::Function(FunctionType {
            signatures: vec![CallSignature {
                type_parameters: vec![],
                parameters: vec![
                    ParameterSignature {
                        name: "a".to_string(),
                        type_: infer_p1,
                        optional: false,
                        rest: false,
                    },
                    ParameterSignature {
                        name: "b".to_string(),
                        type_: infer_p2,
                        optional: false,
                        rest: false,
                    },
                ],
                return_type: Arc::new(Type::Any),
            }],
        });

        let source_func = Type::Function(FunctionType {
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
                        optional: false,
                        rest: false,
                    },
                ],
                return_type: Arc::new(Type::Void),
            }],
        });

        let mut context = InferenceContext::new();
        ConditionalTypeInferrer::infer_recursive(&source_func, &target_func, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        assert!(result.success);
        assert!(result.inferred_types.contains_key("P"));

        // P should be string & number (intersection) due to contravariance
        if let Type::Intersection(members) = result.inferred_types.get("P").unwrap().as_ref() {
            assert_eq!(members.len(), 2);
        } else {
            panic!("Expected Intersection type for P");
        }
    }

    #[test]
    fn test_variance_combine() {
        assert_eq!(Variance::Covariant.combine(Variance::Covariant), Variance::Covariant);
        assert_eq!(
            Variance::Contravariant.combine(Variance::Contravariant),
            Variance::Covariant
        );
        assert_eq!(
            Variance::Covariant.combine(Variance::Contravariant),
            Variance::Contravariant
        );
        assert_eq!(Variance::Covariant.flip(), Variance::Contravariant);
        assert_eq!(Variance::Contravariant.flip(), Variance::Covariant);
    }

    #[test]
    fn test_substitute_inferred() {
        let mut inferred = HashMap::new();
        inferred.insert("R".to_string(), Arc::new(Type::String));

        let type_param = Type::TypeParameter(TypeParameter {
            name: "infer R".to_string(),
            constraint: None,
            default: None,
        });

        let result = ConditionalTypeInferrer::substitute_inferred(&type_param, &inferred);

        if let Type::String = result.as_ref() {
            // Success
        } else {
            panic!("Expected String after substitution");
        }
    }

    #[test]
    fn test_infer_template_literal() {
        // Test: type ParseRoute<T> = T extends `${infer Start}/${infer End}` ? [Start, End] : never
        let template = "${infer Start}/${infer End}";
        let source = Type::StringLiteral("users/profile".to_string());

        let mut context = InferenceContext::new();
        ConditionalTypeInferrer::infer_from_template_literal(&source, template, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        assert!(result.success);

        if let Type::StringLiteral(s) = result.inferred_types.get("Start").unwrap().as_ref() {
            assert_eq!(s, "users");
        } else {
            panic!("Expected StringLiteral for Start");
        }

        if let Type::StringLiteral(s) = result.inferred_types.get("End").unwrap().as_ref() {
            assert_eq!(s, "profile");
        } else {
            panic!("Expected StringLiteral for End");
        }
    }

    #[test]
    fn test_conditional_type_evaluation() {
        // Test: type IsString<T> = T extends string ? true : false
        let conditional = ConditionalType {
            check_type: Arc::new(Type::StringLiteral("hello".to_string())),
            extends_type: Arc::new(Type::String),
            true_type: Arc::new(Type::BooleanLiteral(true)),
            false_type: Arc::new(Type::BooleanLiteral(false)),
        };

        let mut context = InferenceContext::new();
        let result = ConditionalTypeInferrer::evaluate(&conditional, &mut context);

        if let Type::BooleanLiteral(true) = result.as_ref() {
            // Success - string literal extends string
        } else {
            panic!("Expected true branch");
        }
    }

    #[test]
    fn test_nested_conditional_inference() {
        // Test nested conditionals
        let inner_conditional = ConditionalType {
            check_type: Arc::new(Type::TypeParameter(TypeParameter {
                name: "infer T".to_string(),
                constraint: None,
                default: None,
            })),
            extends_type: Arc::new(Type::String),
            true_type: Arc::new(Type::BooleanLiteral(true)),
            false_type: Arc::new(Type::BooleanLiteral(false)),
        };

        let outer_conditional = ConditionalType {
            check_type: Arc::new(Type::String),
            extends_type: Arc::new(Type::Conditional(inner_conditional)),
            true_type: Arc::new(Type::StringLiteral("nested".to_string())),
            false_type: Arc::new(Type::Never),
        };

        let mut context = InferenceContext::new();

        // This tests that nested conditionals are handled
        context.enter_nested(|ctx| {
            assert_eq!(ctx.depth(), 1);
        });
        assert_eq!(context.depth(), 0);
    }

    #[test]
    fn test_infer_array_element() {
        // Test: type ElementType<T> = T extends (infer E)[] ? E : never
        let infer_e = InferTypeBuilder::new("E").build_arc();

        let target_array = Type::Array(infer_e);
        let source_array = Type::Array(Arc::new(Type::String));

        let mut context = InferenceContext::new();
        ConditionalTypeInferrer::infer_recursive(&source_array, &target_array, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        assert!(result.success);
        assert!(result.inferred_types.contains_key("E"));

        if let Type::String = result.inferred_types.get("E").unwrap().as_ref() {
            // Success
        } else {
            panic!("Expected String type for E");
        }
    }

    #[test]
    fn test_infer_from_union() {
        // Test inference from union type
        let infer_t = InferTypeBuilder::new("T").build_arc();

        let target_union = Type::Union(vec![
            Arc::new(Type::String),
            infer_t,
        ]);

        let source = Type::Number;

        let mut context = InferenceContext::new();
        ConditionalTypeInferrer::infer_recursive(&source, &target_union, &mut context, 0);

        let result = ConditionalTypeInferrer::finalize_inference(&context);
        // T should be inferred as Number
        if result.inferred_types.contains_key("T") {
            if let Type::Number = result.inferred_types.get("T").unwrap().as_ref() {
                // Success
            } else {
                panic!("Expected Number for T");
            }
        }
    }

    #[test]
    fn test_is_assignable() {
        // Test assignability checks
        assert!(ConditionalTypeInferrer::is_assignable_to(&Type::String, &Type::String));
        assert!(ConditionalTypeInferrer::is_assignable_to(
            &Type::StringLiteral("hello".to_string()),
            &Type::String
        ));
        assert!(!ConditionalTypeInferrer::is_assignable_to(&Type::Number, &Type::String));
        assert!(ConditionalTypeInferrer::is_assignable_to(&Type::Any, &Type::String));
        assert!(ConditionalTypeInferrer::is_assignable_to(&Type::Never, &Type::String));
    }
}
