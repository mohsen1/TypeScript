//! Type evaluation for meta-types (conditional, mapped, index access).
//!
//! Meta-types are "type-level functions" that compute output types from input types.
//! This module provides evaluation logic for:
//! - Conditional types: T extends U ? X : Y
//! - Distributive conditional types: (A | B) extends U ? X : Y
//! - Index access types: T[K]
//!
//! Key design:
//! - Lazy evaluation: only evaluate when needed for subtype checking
//! - Handles deferred evaluation when type parameters are unknown
//! - Supports distributivity for naked type parameters in unions

use crate::solver::types::*;
use crate::solver::{apparent_primitive_members, TypeDatabase};
use crate::solver::subtype::{SubtypeChecker, TypeResolver, NoopResolver};
use crate::solver::instantiate::{TypeSubstitution, instantiate_type};

#[cfg(test)]
use crate::solver::TypeInterner;

/// Result of conditional type evaluation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConditionalResult {
    /// The condition was resolved to a definite type
    Resolved(TypeId),
    /// The condition could not be resolved (deferred)
    /// This happens when check_type is a type parameter that hasn't been substituted
    Deferred(TypeId),
}

/// Type evaluator for meta-types.
pub struct TypeEvaluator<'a, R: TypeResolver = NoopResolver> {
    interner: &'a dyn TypeDatabase,
    resolver: &'a R,
    no_unchecked_indexed_access: bool,
}

impl<'a> TypeEvaluator<'a, NoopResolver> {
    /// Create a new evaluator without a resolver.
    pub fn new(interner: &'a dyn TypeDatabase) -> TypeEvaluator<'a, NoopResolver> {
        static NOOP: NoopResolver = NoopResolver;
        TypeEvaluator {
            interner,
            resolver: &NOOP,
            no_unchecked_indexed_access: false,
        }
    }
}

impl<'a, R: TypeResolver> TypeEvaluator<'a, R> {
    /// Create a new evaluator with a custom resolver.
    pub fn with_resolver(interner: &'a dyn TypeDatabase, resolver: &'a R) -> Self {
        TypeEvaluator {
            interner,
            resolver,
            no_unchecked_indexed_access: false,
        }
    }

    pub fn set_no_unchecked_indexed_access(&mut self, enabled: bool) {
        self.no_unchecked_indexed_access = enabled;
    }

    /// Evaluate a type, resolving any meta-types if possible.
    /// Returns the evaluated type (may be the same if no evaluation needed).
    pub fn evaluate(&self, type_id: TypeId) -> TypeId {
        // Fast path for intrinsics
        if type_id.is_intrinsic() {
            return type_id;
        }

        let key = match self.interner.lookup(type_id) {
            Some(k) => k,
            None => return type_id,
        };

        match &key {
            TypeKey::Conditional(cond) => {
                self.evaluate_conditional(cond)
            }
            TypeKey::IndexAccess(obj, idx) => {
                self.evaluate_index_access(*obj, *idx)
            }
            TypeKey::Mapped(mapped) => {
                self.evaluate_mapped(mapped)
            }
            TypeKey::KeyOf(operand) => {
                self.evaluate_keyof(*operand)
            }
            // Other types pass through unchanged
            _ => type_id,
        }
    }

    /// Evaluate a conditional type: T extends U ? X : Y
    ///
    /// Algorithm:
    /// 1. If check_type is a union and the conditional is distributive, distribute
    /// 2. Otherwise, check if check_type <: extends_type
    /// 3. If true -> return true_type
    /// 4. If false (disjoint) -> return false_type
    /// 5. If ambiguous (unresolved type param) -> return deferred conditional
    pub fn evaluate_conditional(&self, cond: &ConditionalType) -> TypeId {
        let check_type = cond.check_type;
        let extends_type = cond.extends_type;

        if cond.is_distributive && check_type == TypeId::NEVER {
            return TypeId::NEVER;
        }

        if check_type == TypeId::ANY {
            let true_eval = self.evaluate(cond.true_type);
            let false_eval = self.evaluate(cond.false_type);
            return self.interner.union(vec![true_eval, false_eval]);
        }

        // Step 1: Check for distributivity
        // Only distribute for naked type parameters (recorded at lowering time).
        if cond.is_distributive {
            if let Some(TypeKey::Union(members)) = self.interner.lookup(check_type) {
                return self.distribute_conditional(
                    &members,
                    extends_type,
                    cond.true_type,
                    cond.false_type,
                );
            }
        }

        // Step 2: Check for naked type parameter (defer)
        if let Some(TypeKey::TypeParameter(_)) = self.interner.lookup(check_type) {
            // Type parameter hasn't been substituted - defer evaluation
            return self.interner.intern(TypeKey::Conditional(Box::new(cond.clone())));
        }

        // Step 3: Perform subtype check
        let mut checker = SubtypeChecker::with_resolver(self.interner, self.resolver);

        if checker.is_subtype_of(check_type, extends_type) {
            // T <: U -> true branch
            self.evaluate(cond.true_type)
        } else {
            // Check if types are definitely disjoint
            // For now, we use a simple heuristic: if not subtype, assume disjoint
            // More sophisticated: check if intersection is never
            self.evaluate(cond.false_type)
        }
    }

    /// Distribute a conditional type over a union.
    /// (A | B) extends U ? X : Y -> (A extends U ? X : Y) | (B extends U ? X : Y)
    fn distribute_conditional(
        &self,
        members: &[TypeId],
        extends_type: TypeId,
        true_type: TypeId,
        false_type: TypeId,
    ) -> TypeId {
        let mut results: Vec<TypeId> = Vec::with_capacity(members.len());

        for &member in members {
            // Create conditional for this union member
            let member_cond = ConditionalType {
                check_type: member,
                extends_type,
                true_type,
                false_type,
                is_distributive: false,
            };

            // Recursively evaluate (handles nested unions, type parameters)
            let result = self.evaluate_conditional(&member_cond);
            results.push(result);
        }

        // Combine results into a union
        self.interner.union(results)
    }

    /// Evaluate an index access type: T[K]
    ///
    /// This resolves property access on object types.
    pub fn evaluate_index_access(&self, object_type: TypeId, index_type: TypeId) -> TypeId {
        // Get the object structure
        let obj_key = match self.interner.lookup(object_type) {
            Some(k) => k,
            None => return TypeId::ERROR,
        };

        match obj_key {
            TypeKey::ReadonlyType(inner) => {
                self.evaluate_index_access(inner, index_type)
            }
            TypeKey::Ref(sym) => {
                if let Some(resolved) = self.resolver.resolve_ref(sym, self.interner) {
                    if resolved == object_type {
                        self.interner.intern(TypeKey::IndexAccess(object_type, index_type))
                    } else {
                        self.evaluate_index_access(resolved, index_type)
                    }
                } else {
                    self.interner.intern(TypeKey::IndexAccess(object_type, index_type))
                }
            }
            TypeKey::TypeParameter(param) | TypeKey::Infer(param) => {
                if let Some(constraint) = param.constraint {
                    if constraint == object_type {
                        self.interner.intern(TypeKey::IndexAccess(object_type, index_type))
                    } else {
                        self.evaluate_index_access(constraint, index_type)
                    }
                } else {
                    self.interner.intern(TypeKey::IndexAccess(object_type, index_type))
                }
            }
            TypeKey::Object(props) => {
                self.evaluate_object_index(&props, index_type)
            }
            TypeKey::ObjectWithIndex(shape) => {
                self.evaluate_object_with_index(&shape, index_type)
            }
            TypeKey::Union(members) => {
                let mut results = Vec::new();
                for &member in &members {
                    let result = self.evaluate_index_access(member, index_type);
                    if result != TypeId::UNDEFINED || self.no_unchecked_indexed_access {
                        results.push(result);
                    }
                }
                if results.is_empty() {
                    return TypeId::UNDEFINED;
                }
                self.interner.union(results)
            }
            TypeKey::Array(elem) => {
                // Array[number] -> element type
                if self.is_number_like(index_type) {
                    self.add_undefined_if_unchecked(elem)
                } else {
                    // Could be string key for length etc, but for now return element
                    elem
                }
            }
            TypeKey::Tuple(elements) => {
                self.evaluate_tuple_index(&elements, index_type)
            }
            // For other types, keep as IndexAccess (deferred)
            _ => self.interner.intern(TypeKey::IndexAccess(object_type, index_type)),
        }
    }

    /// Evaluate property access on an object type
    fn evaluate_object_index(&self, props: &[PropertyInfo], index_type: TypeId) -> TypeId {
        // If index is a literal string, look up the property directly
        if let Some(TypeKey::Literal(LiteralValue::String(name))) = self.interner.lookup(index_type) {
            for prop in props {
                if prop.name == name {
                    return prop.type_id;
                }
            }
            // Property not found
            return TypeId::UNDEFINED;
        }

        // If index is a union of literals, return union of property types
        if let Some(TypeKey::Union(members)) = self.interner.lookup(index_type) {
            let mut results = Vec::new();
            for &member in &members {
                let result = self.evaluate_object_index(props, member);
                if result != TypeId::UNDEFINED || self.no_unchecked_indexed_access {
                    results.push(result);
                }
            }
            if results.is_empty() {
                return TypeId::UNDEFINED;
            }
            return self.interner.union(results);
        }

        // If index is string, return union of all property types (index signature behavior)
        if index_type == TypeId::STRING {
            let all_types: Vec<TypeId> = props.iter().map(|p| p.type_id).collect();
            if all_types.is_empty() {
                return TypeId::UNDEFINED;
            }
            let union = self.interner.union(all_types);
            return self.add_undefined_if_unchecked(union);
        }

        TypeId::UNDEFINED
    }

    /// Evaluate property access on an object type with index signatures.
    fn evaluate_object_with_index(&self, shape: &ObjectShape, index_type: TypeId) -> TypeId {
        // If index is a union, evaluate each member
        if let Some(TypeKey::Union(members)) = self.interner.lookup(index_type) {
            let mut results = Vec::new();
            for &member in &members {
                let result = self.evaluate_object_with_index(shape, member);
                if result != TypeId::UNDEFINED || self.no_unchecked_indexed_access {
                    results.push(result);
                }
            }
            if results.is_empty() {
                return TypeId::UNDEFINED;
            }
            return self.interner.union(results);
        }

        // If index is a literal string, look up the property first, then fallback to string index.
        if let Some(TypeKey::Literal(LiteralValue::String(name))) = self.interner.lookup(index_type) {
            for prop in &shape.properties {
                if prop.name == name {
                    return prop.type_id;
                }
            }
            if let Some(string_index) = shape.string_index.as_ref() {
                return self.add_undefined_if_unchecked(string_index.value_type);
            }
            return TypeId::UNDEFINED;
        }

        // If index is a literal number, prefer number index, then string index.
        if let Some(TypeKey::Literal(LiteralValue::Number(_))) = self.interner.lookup(index_type) {
            if let Some(number_index) = shape.number_index.as_ref() {
                return self.add_undefined_if_unchecked(number_index.value_type);
            }
            if let Some(string_index) = shape.string_index.as_ref() {
                return self.add_undefined_if_unchecked(string_index.value_type);
            }
            return TypeId::UNDEFINED;
        }

        if index_type == TypeId::STRING {
            let result = if let Some(string_index) = shape.string_index.as_ref() {
                string_index.value_type
            } else {
                self.union_property_types(&shape.properties)
            };
            return self.add_undefined_if_unchecked(result);
        }

        if index_type == TypeId::NUMBER {
            let result = if let Some(number_index) = shape.number_index.as_ref() {
                number_index.value_type
            } else if let Some(string_index) = shape.string_index.as_ref() {
                string_index.value_type
            } else {
                self.union_property_types(&shape.properties)
            };
            return self.add_undefined_if_unchecked(result);
        }

        TypeId::UNDEFINED
    }

    fn union_property_types(&self, props: &[PropertyInfo]) -> TypeId {
        let all_types: Vec<TypeId> = props.iter().map(|p| p.type_id).collect();
        if all_types.is_empty() {
            TypeId::UNDEFINED
        } else {
            self.interner.union(all_types)
        }
    }

    fn add_undefined_if_unchecked(&self, type_id: TypeId) -> TypeId {
        if !self.no_unchecked_indexed_access || type_id == TypeId::UNDEFINED {
            return type_id;
        }
        self.interner.union(vec![type_id, TypeId::UNDEFINED])
    }

    /// Evaluate index access on a tuple type
    fn evaluate_tuple_index(&self, elements: &[TupleElement], index_type: TypeId) -> TypeId {
        // If index is a literal number, return the specific element
        if let Some(TypeKey::Literal(LiteralValue::Number(n))) = self.interner.lookup(index_type) {
            let idx = n.0 as usize;
            if idx < elements.len() {
                return elements[idx].type_id;
            }
            // Check for rest element
            if let Some(last) = elements.last() {
                if last.rest {
                    return last.type_id;
                }
            }
            return TypeId::UNDEFINED;
        }

        // If index is number, return union of all element types
        if index_type == TypeId::NUMBER {
            let all_types: Vec<TypeId> = elements.iter().map(|e| e.type_id).collect();
            if all_types.is_empty() {
                return TypeId::NEVER;
            }
            let union = self.interner.union(all_types);
            return self.add_undefined_if_unchecked(union);
        }

        TypeId::UNDEFINED
    }

    /// Check if a type is number-like (number or numeric literal)
    fn is_number_like(&self, type_id: TypeId) -> bool {
        if type_id == TypeId::NUMBER {
            return true;
        }
        if let Some(TypeKey::Literal(LiteralValue::Number(_))) = self.interner.lookup(type_id) {
            return true;
        }
        false
    }

    /// Evaluate a mapped type: { [K in Keys]: Template }
    ///
    /// Algorithm:
    /// 1. Extract the constraint (Keys) - this defines what keys to iterate over
    /// 2. For each key K in the constraint:
    ///    - Substitute K into the template type
    ///    - Apply readonly/optional modifiers
    /// 3. Construct a new object type with the resulting properties
    pub fn evaluate_mapped(&self, mapped: &MappedType) -> TypeId {
        // Get the constraint - this tells us what keys to iterate over
        let constraint = mapped.constraint;

        // Evaluate the constraint to get concrete keys
        let keys = self.evaluate_keyof_or_constraint(constraint);

        // If we can't determine concrete keys, keep it as a mapped type (deferred)
        let key_literals = match self.extract_string_literals(keys) {
            Some(literals) => literals,
            None => {
                // Can't evaluate - return deferred mapped type
                return self.interner.intern(TypeKey::Mapped(Box::new(mapped.clone())));
            }
        };

        // Build the resulting object properties
        let mut properties = Vec::new();

        for key_name in key_literals {
            // Create substitution: type_param.name -> literal key type
            // First intern the Atom as a literal string type
            let key_literal = self.interner.intern(TypeKey::Literal(LiteralValue::String(key_name)));

            let mut subst = TypeSubstitution::new();
            subst.insert(mapped.type_param.name, key_literal);

            // Substitute into the template
            let property_type = instantiate_type(self.interner, mapped.template, &subst);

            // Apply modifiers
            let optional = match mapped.optional_modifier {
                Some(MappedModifier::Add) => true,
                Some(MappedModifier::Remove) => false,
                None => false, // Default: preserve original (but we don't have original info here)
            };

            let readonly = match mapped.readonly_modifier {
                Some(MappedModifier::Add) => true,
                Some(MappedModifier::Remove) => false,
                None => false,
            };

            properties.push(PropertyInfo {
                name: key_name,
                type_id: property_type,
                optional,
                readonly,
                is_method: false,
            });
        }

        self.interner.object(properties)
    }

    /// Evaluate keyof T - extract the keys of an object type
    pub fn evaluate_keyof(&self, operand: TypeId) -> TypeId {
        // First evaluate the operand in case it's a meta-type
        let evaluated_operand = self.evaluate(operand);

        let key = match self.interner.lookup(evaluated_operand) {
            Some(k) => k,
            None => return TypeId::NEVER,
        };

        match key {
            TypeKey::ReadonlyType(inner) => {
                self.evaluate_keyof(inner)
            }
            TypeKey::Ref(sym) => {
                if let Some(resolved) = self.resolver.resolve_ref(sym, self.interner) {
                    if resolved == evaluated_operand {
                        self.interner.intern(TypeKey::KeyOf(operand))
                    } else {
                        self.evaluate_keyof(resolved)
                    }
                } else {
                    self.interner.intern(TypeKey::KeyOf(operand))
                }
            }
            TypeKey::Object(props) => {
                // keyof { x: T, y: U } = "x" | "y"
                if props.is_empty() {
                    return TypeId::NEVER;
                }
                let key_types: Vec<TypeId> = props.iter()
                    .map(|p| self.interner.intern(TypeKey::Literal(LiteralValue::String(p.name))))
                    .collect();
                self.interner.union(key_types)
            }
            TypeKey::ObjectWithIndex(shape) => {
                let mut key_types: Vec<TypeId> = shape
                    .properties
                    .iter()
                    .map(|p| self.interner.intern(TypeKey::Literal(LiteralValue::String(p.name))))
                    .collect();

                if shape.string_index.is_some() {
                    key_types.push(TypeId::STRING);
                    key_types.push(TypeId::NUMBER);
                } else if shape.number_index.is_some() {
                    key_types.push(TypeId::NUMBER);
                }

                if key_types.is_empty() {
                    TypeId::NEVER
                } else {
                    self.interner.union(key_types)
                }
            }
            TypeKey::Array(_) => {
                // keyof T[] = number | array methods
                // Simplified: just return number for array indices
                TypeId::NUMBER
            }
            TypeKey::Tuple(elements) => {
                // keyof [A, B, C] = "0" | "1" | "2" | number methods
                let key_types: Vec<TypeId> = (0..elements.len())
                    .map(|i| self.interner.literal_string(&i.to_string()))
                    .collect();
                if key_types.is_empty() {
                    return TypeId::NEVER;
                }
                self.interner.union(key_types)
            }
            TypeKey::Intrinsic(kind) => match kind {
                IntrinsicKind::Any => {
                    // keyof any = string | number | symbol
                    self.interner.union(vec![TypeId::STRING, TypeId::NUMBER, TypeId::SYMBOL])
                }
                IntrinsicKind::Unknown => {
                    // keyof unknown = never
                    TypeId::NEVER
                }
                IntrinsicKind::String
                | IntrinsicKind::Number
                | IntrinsicKind::Boolean
                | IntrinsicKind::Bigint
                | IntrinsicKind::Symbol => self.apparent_primitive_keyof(kind),
                _ => self.interner.intern(TypeKey::KeyOf(operand)),
            },
            TypeKey::Literal(literal) => {
                if let Some(kind) = self.apparent_literal_kind(&literal) {
                    self.apparent_primitive_keyof(kind)
                } else {
                    self.interner.intern(TypeKey::KeyOf(operand))
                }
            }
            TypeKey::Union(members) => {
                // keyof (A | B) = keyof A & keyof B
                let key_sets: Vec<TypeId> = members.iter()
                    .map(|&m| self.evaluate_keyof(m))
                    .collect();
                // Intersection of all key sets
                self.interner.intersection(key_sets)
            }
            TypeKey::Intersection(members) => {
                // keyof (A & B) = keyof A | keyof B
                let key_sets: Vec<TypeId> = members.iter()
                    .map(|&m| self.evaluate_keyof(m))
                    .collect();
                self.interner.union(key_sets)
            }
            // For other types (type parameters, etc.), keep as KeyOf (deferred)
            _ => self.interner.intern(TypeKey::KeyOf(operand)),
        }
    }

    /// Helper to evaluate keyof or pass through union constraint
    fn evaluate_keyof_or_constraint(&self, constraint: TypeId) -> TypeId {
        // If constraint is already a union of literals, return it
        if let Some(TypeKey::Union(_)) = self.interner.lookup(constraint) {
            return constraint;
        }

        // If constraint is a literal, return it
        if let Some(TypeKey::Literal(LiteralValue::String(_))) = self.interner.lookup(constraint) {
            return constraint;
        }

        // If constraint is KeyOf, evaluate it
        if let Some(TypeKey::KeyOf(operand)) = self.interner.lookup(constraint) {
            return self.evaluate_keyof(operand);
        }

        // Otherwise return as-is
        constraint
    }

    /// Extract string literals from a type (for mapped type iteration)
    fn extract_string_literals(&self, type_id: TypeId) -> Option<Vec<crate::interner::Atom>> {
        let key = self.interner.lookup(type_id)?;

        match key {
            TypeKey::Literal(LiteralValue::String(s)) => {
                Some(vec![s])
            }
            TypeKey::Union(members) => {
                let mut result = Vec::new();
                for &member in &members {
                    if let Some(TypeKey::Literal(LiteralValue::String(s))) = self.interner.lookup(member) {
                        result.push(s);
                    } else {
                        // Non-literal in union - can't fully evaluate
                        return None;
                    }
                }
                Some(result)
            }
            // Can't extract literals from other types
            _ => None,
        }
    }

    fn apparent_literal_kind(&self, literal: &LiteralValue) -> Option<IntrinsicKind> {
        match literal {
            LiteralValue::String(_) => Some(IntrinsicKind::String),
            LiteralValue::Number(_) => Some(IntrinsicKind::Number),
            LiteralValue::BigInt(_) => Some(IntrinsicKind::Bigint),
            LiteralValue::Boolean(_) => Some(IntrinsicKind::Boolean),
        }
    }

    fn apparent_primitive_keyof(&self, kind: IntrinsicKind) -> TypeId {
        let members = apparent_primitive_members(self.interner, kind);
        let mut key_types = Vec::with_capacity(members.len());
        for member in members {
            key_types.push(self.interner.literal_string(member.name));
        }
        if key_types.is_empty() {
            TypeId::NEVER
        } else {
            self.interner.union(key_types)
        }
    }
}

/// Convenience function for evaluating conditional types
pub fn evaluate_conditional(
    interner: &dyn TypeDatabase,
    cond: &ConditionalType,
) -> TypeId {
    let evaluator = TypeEvaluator::new(interner);
    evaluator.evaluate_conditional(cond)
}

/// Convenience function for evaluating index access types
pub fn evaluate_index_access(
    interner: &dyn TypeDatabase,
    object_type: TypeId,
    index_type: TypeId,
) -> TypeId {
    let evaluator = TypeEvaluator::new(interner);
    evaluator.evaluate_index_access(object_type, index_type)
}

/// Convenience function for full type evaluation
pub fn evaluate_type(interner: &dyn TypeDatabase, type_id: TypeId) -> TypeId {
    let evaluator = TypeEvaluator::new(interner);
    evaluator.evaluate(type_id)
}

/// Convenience function for evaluating mapped types
pub fn evaluate_mapped(interner: &dyn TypeDatabase, mapped: &MappedType) -> TypeId {
    let evaluator = TypeEvaluator::new(interner);
    evaluator.evaluate_mapped(mapped)
}

/// Convenience function for evaluating keyof types
pub fn evaluate_keyof(interner: &dyn TypeDatabase, operand: TypeId) -> TypeId {
    let evaluator = TypeEvaluator::new(interner);
    evaluator.evaluate_keyof(operand)
}

#[cfg(test)]
#[path = "evaluate_tests.rs"]
mod tests;
