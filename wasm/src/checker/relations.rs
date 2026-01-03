//! Type relationship checking for the type checker.
//!
//! This module contains the structural type compatibility checking logic,
//! implementing TypeScript's assignability, subtyping, and identity relations.

use super::types::{
    type_flags, object_flags,
    Type, TypeId, Signature,
};
use super::state::{CheckerState, TypeRelation};

impl<'a> CheckerState<'a> {
    /// Check if source type is related to target type under the given relation.
    /// This is the core of TypeScript's structural type system.
    ///
    /// # Arguments
    /// * `source` - The source type (e.g., the type of the value being assigned)
    /// * `target` - The target type (e.g., the declared type of the variable)
    /// * `relation` - The type of relationship to check
    ///
    /// # Returns
    /// `true` if source is related to target, `false` otherwise
    pub fn is_type_related_to(&self, source: TypeId, target: TypeId, relation: TypeRelation) -> bool {
        // Identity check - same type reference is always related
        if source == target {
            return true;
        }

        // Get the actual types
        let source_type = match self.types.get(source) {
            Some(t) => t,
            None => return false,
        };
        let target_type = match self.types.get(target) {
            Some(t) => t,
            None => return false,
        };

        let source_flags = source_type.flags();
        let target_flags = target_type.flags();

        // Quick checks based on flags
        // 1. never is a subtype of everything (bottom type)
        if (source_flags & type_flags::NEVER) != 0 {
            return true;
        }

        // 2. Everything is a subtype of unknown (top type for safe operations)
        if (target_flags & type_flags::UNKNOWN) != 0 {
            return true;
        }

        // 3. any is assignable to/from everything (opt-out of type checking)
        if (source_flags & type_flags::ANY) != 0 || (target_flags & type_flags::ANY) != 0 {
            // For identity, any !== any unless same reference
            if relation == TypeRelation::Identity {
                return source == target;
            }
            return true;
        }

        // 4. undefined is assignable to void
        if (source_flags & type_flags::UNDEFINED) != 0 && (target_flags & type_flags::VOID) != 0 {
            return true;
        }

        // 5. null and undefined are only assignable to themselves, any, unknown, and void
        //    (with strictNullChecks, which we assume is on)
        if (source_flags & type_flags::NULLABLE) != 0 {
            // Already checked any/unknown above
            // Check if target is same nullable type or void
            if (source_flags & target_flags & type_flags::NULLABLE) != 0 {
                return true;
            }
            // Check for union containing null/undefined
            if let Type::Union(u) = target_type {
                // Source is related if it's related to at least one member
                for &member in &u.types {
                    if self.is_type_related_to(source, member, relation) {
                        return true;
                    }
                }
            }
            return false;
        }

        // Handle source being a union type
        // A union is related to target if ALL of its members are related
        if let Type::Union(u) = source_type {
            let source_types = u.types.clone();
            for member in source_types {
                if !self.is_type_related_to(member, target, relation) {
                    return false;
                }
            }
            return true;
        }

        // Handle target being a union type
        // Source is related to union if it's related to at least one member
        if let Type::Union(u) = target_type {
            let target_types = u.types.clone();
            for member in target_types {
                if self.is_type_related_to(source, member, relation) {
                    return true;
                }
            }
            return false;
        }

        // Handle source being an intersection type
        // Intersection is related if ANY of its members is related
        if let Type::Intersection(i) = source_type {
            let source_types = i.types.clone();
            for member in source_types {
                if self.is_type_related_to(member, target, relation) {
                    return true;
                }
            }
            // Fall through to check structural compatibility
        }

        // Handle target being an intersection type
        // Source must be related to ALL members of the intersection
        if let Type::Intersection(i) = target_type {
            let target_types = i.types.clone();
            for member in target_types {
                if !self.is_type_related_to(source, member, relation) {
                    return false;
                }
            }
            return true;
        }

        // Primitive type compatibility
        if self.is_primitive_type_related(source_flags, target_flags, relation) {
            return true;
        }

        // Literal types are related to their widened types
        if self.is_literal_type_related(source, source_type, target, target_type, relation) {
            return true;
        }

        // Object type structural compatibility
        if self.is_object_type_related(source, source_type, target, target_type, relation) {
            return true;
        }

        // Array type compatibility
        if self.is_array_type_related(source, source_type, target, target_type, relation) {
            return true;
        }

        // Function type compatibility
        if self.is_function_type_related(source, source_type, target, target_type, relation) {
            return true;
        }

        // Tuple type compatibility
        if self.is_tuple_type_related(source, source_type, target, target_type, relation) {
            return true;
        }

        false
    }

    /// Check if primitive types are related.
    fn is_primitive_type_related(&self, source_flags: u32, target_flags: u32, relation: TypeRelation) -> bool {
        // Same primitive type (checking major flags)
        let primitives = type_flags::STRING | type_flags::NUMBER | type_flags::BOOLEAN
            | type_flags::BIG_INT | type_flags::ES_SYMBOL | type_flags::VOID
            | type_flags::UNDEFINED | type_flags::NULL;

        let source_primitive = source_flags & primitives;
        let target_primitive = target_flags & primitives;

        if source_primitive != 0 && source_primitive == target_primitive {
            return true;
        }

        // For identity, primitives must match exactly
        if relation == TypeRelation::Identity {
            return false;
        }

        // String literal to string
        if (source_flags & type_flags::STRING_LITERAL) != 0 && (target_flags & type_flags::STRING) != 0 {
            return true;
        }

        // Number literal to number
        if (source_flags & type_flags::NUMBER_LITERAL) != 0 && (target_flags & type_flags::NUMBER) != 0 {
            return true;
        }

        // Boolean literal to boolean
        if (source_flags & type_flags::BOOLEAN_LITERAL) != 0 && (target_flags & type_flags::BOOLEAN) != 0 {
            return true;
        }

        // BigInt literal to bigint
        if (source_flags & type_flags::BIG_INT_LITERAL) != 0 && (target_flags & type_flags::BIG_INT) != 0 {
            return true;
        }

        // Enum to number (non-const enums)
        if (source_flags & type_flags::ENUM) != 0 && (target_flags & type_flags::NUMBER) != 0 {
            return true;
        }

        false
    }

    /// Check if literal types are related.
    fn is_literal_type_related(
        &self,
        _source: TypeId,
        source_type: &Type,
        _target: TypeId,
        target_type: &Type,
        _relation: TypeRelation,
    ) -> bool {
        // For identity, literal values must match exactly
        if let (Type::Literal(src_lit), Type::Literal(tgt_lit)) = (source_type, target_type) {
            return src_lit.value == tgt_lit.value;
        }

        // Already handled in is_primitive_type_related for literal->base type
        false
    }

    /// Check if object types are structurally related.
    fn is_object_type_related(
        &self,
        _source: TypeId,
        source_type: &Type,
        _target: TypeId,
        target_type: &Type,
        relation: TypeRelation,
    ) -> bool {
        let (source_obj, target_obj) = match (source_type, target_type) {
            (Type::Object(s), Type::Object(t)) => (s, t),
            _ => return false,
        };

        // For each property in target, check that source has a compatible property
        for (name, &target_symbol) in target_obj.members.iter() {
            // Look for property in source
            let source_symbol = match source_obj.members.get(name) {
                Some(s) => s,
                None => {
                    // Missing property - check if it's optional in target
                    // For now, assume all properties are required
                    return false;
                }
            };

            // Get types of properties
            let source_prop_type = self.symbol_types.get(&source_symbol).copied()
                .unwrap_or(self.types.any_type);
            let target_prop_type = self.symbol_types.get(&target_symbol).copied()
                .unwrap_or(self.types.any_type);

            // Property types must be related
            if !self.is_type_related_to(source_prop_type, target_prop_type, relation) {
                return false;
            }
        }

        // Excess property check for fresh object literals
        // Only check if source is a fresh object literal (has FRESH_LITERAL flag)
        if source_obj.has_object_flags(object_flags::FRESH_LITERAL) {
            // Check if target has any index signature that would accept excess properties
            let has_index_signature = !target_obj.index_infos.is_empty();

            if !has_index_signature {
                // For each property in source, check that target has it
                for (name, _) in source_obj.members.iter() {
                    if !target_obj.members.has(name) {
                        // Excess property found
                        return false;
                    }
                }
            }
        }

        // Check call signatures compatibility
        if !target_obj.call_signatures.is_empty() {
            if source_obj.call_signatures.is_empty() {
                return false;
            }
            // Each target signature must have a compatible source signature
            // (Simplified: just check if there's at least one compatible signature)
            for target_sig in &target_obj.call_signatures {
                let mut found = false;
                for source_sig in &source_obj.call_signatures {
                    if self.is_signature_related(source_sig, target_sig, relation) {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return false;
                }
            }
        }

        // Check construct signatures compatibility
        if !target_obj.construct_signatures.is_empty() {
            if source_obj.construct_signatures.is_empty() {
                return false;
            }
            for target_sig in &target_obj.construct_signatures {
                let mut found = false;
                for source_sig in &source_obj.construct_signatures {
                    if self.is_signature_related(source_sig, target_sig, relation) {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return false;
                }
            }
        }

        // Check index signatures
        for target_index in &target_obj.index_infos {
            let mut found = false;
            for source_index in &source_obj.index_infos {
                // Key types must match
                if self.is_type_related_to(source_index.key_type, target_index.key_type, TypeRelation::Identity) {
                    // Value type must be related
                    if self.is_type_related_to(source_index.value_type, target_index.value_type, relation) {
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                // Source must also be compatible if it has more specific properties
                // For now, fail if no matching index signature
                return false;
            }
        }

        true
    }

    /// Check if call/construct signatures are related.
    fn is_signature_related(&self, source: &Signature, target: &Signature, relation: TypeRelation) -> bool {
        // Target signature can have more required parameters than source
        // (source with fewer params is callable with more args)

        // Check parameter count compatibility
        if source.min_argument_count > target.min_argument_count {
            // Source requires more params than target - not compatible
            // Actually this is backwards - revisit TypeScript semantics
        }

        // For each parameter position up to min(source, target), check types
        // Note: In TypeScript, function parameters are bivariant for pragmatic reasons
        // but strictFunctionTypes makes them contravariant
        let param_count = std::cmp::min(source.parameters.len(), target.parameters.len());
        for i in 0..param_count {
            let source_param = source.parameters.get(i).copied();
            let target_param = target.parameters.get(i).copied();

            if let (Some(src_sym), Some(tgt_sym)) = (source_param, target_param) {
                let src_type = self.symbol_types.get(&src_sym).copied()
                    .unwrap_or(self.types.any_type);
                let tgt_type = self.symbol_types.get(&tgt_sym).copied()
                    .unwrap_or(self.types.any_type);

                // Parameters are contravariant (target param must be assignable to source param)
                // But we use bivariance for compatibility
                if !self.is_type_related_to(src_type, tgt_type, relation)
                    && !self.is_type_related_to(tgt_type, src_type, relation) {
                    return false;
                }
            }
        }

        // Check return types (covariant)
        if let (Some(src_ret), Some(tgt_ret)) = (source.resolved_return_type, target.resolved_return_type) {
            if !self.is_type_related_to(src_ret, tgt_ret, relation) {
                return false;
            }
        }

        true
    }

    /// Check if array types are related.
    fn is_array_type_related(
        &self,
        _source: TypeId,
        source_type: &Type,
        _target: TypeId,
        target_type: &Type,
        relation: TypeRelation,
    ) -> bool {
        let (source_arr, target_arr) = match (source_type, target_type) {
            (Type::Array(s), Type::Array(t)) => (s, t),
            _ => return false,
        };

        // Element types must be related
        if !self.is_type_related_to(source_arr.element_type, target_arr.element_type, relation) {
            return false;
        }

        // Readonly array can be assigned to readonly array
        // Mutable array can be assigned to readonly array
        // But readonly array cannot be assigned to mutable array
        if source_arr.is_readonly && !target_arr.is_readonly {
            return false;
        }

        true
    }

    /// Check if tuple types are related.
    fn is_tuple_type_related(
        &self,
        _source: TypeId,
        source_type: &Type,
        _target: TypeId,
        target_type: &Type,
        relation: TypeRelation,
    ) -> bool {
        // Handle Tuple -> Array assignment
        // A tuple [T1, T2, ...] is assignable to U[] if all Ti are assignable to U
        if let Type::Tuple(source_tuple) = source_type {
            if let Type::Array(target_arr) = target_type {
                // All tuple element types must be assignable to the array element type
                for &elem_type in &source_tuple.element_types {
                    if !self.is_type_related_to(elem_type, target_arr.element_type, relation) {
                        return false;
                    }
                }
                // Readonly tuple cannot be assigned to mutable array
                if source_tuple.is_readonly && !target_arr.is_readonly {
                    return false;
                }
                return true;
            }
        }

        let (source_tuple, target_tuple) = match (source_type, target_type) {
            (Type::Tuple(s), Type::Tuple(t)) => (s, t),
            _ => return false,
        };

        // For now, simple length check (ignoring optional/rest elements)
        if !target_tuple.has_optional_elements && !target_tuple.has_rest_element {
            if source_tuple.element_types.len() != target_tuple.element_types.len() {
                return false;
            }
        }

        // Each element type must be related
        let len = std::cmp::min(
            source_tuple.element_types.len(),
            target_tuple.element_types.len()
        );

        for i in 0..len {
            if !self.is_type_related_to(
                source_tuple.element_types[i],
                target_tuple.element_types[i],
                relation
            ) {
                return false;
            }
        }

        // Check readonly compatibility
        if source_tuple.is_readonly && !target_tuple.is_readonly {
            return false;
        }

        true
    }

    /// Check if function types are related.
    fn is_function_type_related(
        &self,
        _source: TypeId,
        source_type: &Type,
        _target: TypeId,
        target_type: &Type,
        relation: TypeRelation,
    ) -> bool {
        let (source_fn, target_fn) = match (source_type, target_type) {
            (Type::Function(s), Type::Function(t)) => (s, t),
            _ => return false,
        };

        // Source must have at least as many parameters as target requires
        if source_fn.min_argument_count > target_fn.parameter_types.len() as u32 {
            return false;
        }

        // Check parameter types (bivariant for pragmatic reasons)
        let param_count = std::cmp::min(
            source_fn.parameter_types.len(),
            target_fn.parameter_types.len()
        );

        for i in 0..param_count {
            let src_param = source_fn.parameter_types[i];
            let tgt_param = target_fn.parameter_types[i];

            // Bivariant check: either direction must work
            if !self.is_type_related_to(src_param, tgt_param, relation)
                && !self.is_type_related_to(tgt_param, src_param, relation) {
                return false;
            }
        }

        // Check return type (covariant)
        if !self.is_type_related_to(source_fn.return_type, target_fn.return_type, relation) {
            return false;
        }

        true
    }

    /// Convenience method to check assignability.
    pub fn is_type_assignable_to(&self, source: TypeId, target: TypeId) -> bool {
        self.is_type_related_to(source, target, TypeRelation::Assignable)
    }

    /// Convenience method to check subtype relationship.
    pub fn is_subtype_of(&self, source: TypeId, target: TypeId) -> bool {
        self.is_type_related_to(source, target, TypeRelation::Subtype)
    }

    /// Convenience method to check type identity.
    pub fn is_type_identical_to(&self, source: TypeId, target: TypeId) -> bool {
        self.is_type_related_to(source, target, TypeRelation::Identity)
    }
}
