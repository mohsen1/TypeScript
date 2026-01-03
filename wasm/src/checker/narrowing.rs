//! Type narrowing for the type checker.
//!
//! This module contains type narrowing logic including typeof guards,
//! instanceof guards, truthiness checks, discriminant guards, and flow analysis.

use crate::binder::{SymbolId, FlowNodeArena, flow_flags};
use crate::parser::NodeIndex;
use super::types::{type_flags, Type, TypeId, LiteralValue};
use super::state::{CheckerState, TypeGuard};

impl<'a> CheckerState<'a> {
    // =========================================================================
    // Type Narrowing
    // =========================================================================

    /// Narrow a type based on a typeof guard.
    /// Returns the narrowed type if the guard matches, or the original type.
    ///
    /// For example, if type is `string | number` and typeof_result is "string",
    /// returns `string`.
    pub fn narrow_type_by_typeof(&mut self, type_id: TypeId, typeof_result: &str) -> TypeId {
        let expected_flags = match typeof_result {
            "string" => type_flags::STRING | type_flags::STRING_LITERAL,
            "number" => type_flags::NUMBER | type_flags::NUMBER_LITERAL,
            "boolean" => type_flags::BOOLEAN | type_flags::BOOLEAN_LITERAL,
            "bigint" => type_flags::BIG_INT | type_flags::BIG_INT_LITERAL,
            "symbol" => type_flags::ES_SYMBOL | type_flags::UNIQUE_ES_SYMBOL,
            "undefined" => type_flags::UNDEFINED,
            "function" => type_flags::OBJECT, // Functions are objects with call signatures
            "object" => type_flags::OBJECT | type_flags::NULL, // null returns "object" for typeof
            _ => return type_id, // Unknown typeof result
        };

        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // If it's a union, filter to matching types
        if let Type::Union(u) = typ {
            let matching_types: Vec<TypeId> = u.types.iter()
                .filter(|&&t| {
                    if let Some(inner) = self.types.get(t) {
                        (inner.flags() & expected_flags) != 0
                    } else {
                        false
                    }
                })
                .copied()
                .collect();

            if matching_types.is_empty() {
                return self.types.never_type;
            } else if matching_types.len() == 1 {
                return matching_types[0];
            } else {
                return self.types.create_union_type(matching_types);
            }
        }

        // For non-union types, check if it matches
        let type_flags = typ.flags();
        if (type_flags & expected_flags) != 0 {
            type_id // Type matches, return as-is
        } else {
            self.types.never_type // Type doesn't match, narrow to never
        }
    }

    /// Narrow a type to exclude types matching a typeof guard.
    /// Returns the narrowed type if the guard doesn't match.
    ///
    /// For example, if type is `string | number` and typeof_result is "string",
    /// returns `number`.
    pub fn narrow_type_by_typeof_negation(&mut self, type_id: TypeId, typeof_result: &str) -> TypeId {
        let excluded_flags = match typeof_result {
            "string" => type_flags::STRING | type_flags::STRING_LITERAL,
            "number" => type_flags::NUMBER | type_flags::NUMBER_LITERAL,
            "boolean" => type_flags::BOOLEAN | type_flags::BOOLEAN_LITERAL,
            "bigint" => type_flags::BIG_INT | type_flags::BIG_INT_LITERAL,
            "symbol" => type_flags::ES_SYMBOL | type_flags::UNIQUE_ES_SYMBOL,
            "undefined" => type_flags::UNDEFINED,
            "function" => type_flags::OBJECT,
            "object" => type_flags::OBJECT | type_flags::NULL,
            _ => return type_id,
        };

        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // If it's a union, filter out matching types
        if let Type::Union(u) = typ {
            let remaining_types: Vec<TypeId> = u.types.iter()
                .filter(|&&t| {
                    if let Some(inner) = self.types.get(t) {
                        (inner.flags() & excluded_flags) == 0
                    } else {
                        true
                    }
                })
                .copied()
                .collect();

            if remaining_types.is_empty() {
                return self.types.never_type;
            } else if remaining_types.len() == 1 {
                return remaining_types[0];
            } else {
                return self.types.create_union_type(remaining_types);
            }
        }

        // For non-union types, check if it should be excluded
        let type_flags = typ.flags();
        if (type_flags & excluded_flags) != 0 {
            self.types.never_type // Type matches exclusion
        } else {
            type_id // Type doesn't match, keep as-is
        }
    }

    /// Narrow a union type to exclude null and undefined.
    pub fn get_type_with_facts(&mut self, type_id: TypeId, include_null: bool, include_undefined: bool) -> TypeId {
        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // If it's a union, filter appropriately
        if let Type::Union(u) = typ {
            let filtered_types: Vec<TypeId> = u.types.iter()
                .filter(|&&t| {
                    if let Some(inner) = self.types.get(t) {
                        let flags = inner.flags();
                        let is_null = (flags & type_flags::NULL) != 0;
                        let is_undefined = (flags & type_flags::UNDEFINED) != 0;

                        if is_null && !include_null {
                            return false;
                        }
                        if is_undefined && !include_undefined {
                            return false;
                        }
                        true
                    } else {
                        true
                    }
                })
                .copied()
                .collect();

            if filtered_types.is_empty() {
                return self.types.never_type;
            } else if filtered_types.len() == 1 {
                return filtered_types[0];
            } else {
                return self.types.create_union_type(filtered_types);
            }
        }

        // For non-union types, check if they should be excluded
        let type_flags = typ.flags();
        if (type_flags & type_flags::NULL) != 0 && !include_null {
            return self.types.never_type;
        }
        if (type_flags & type_flags::UNDEFINED) != 0 && !include_undefined {
            return self.types.never_type;
        }

        type_id
    }

    /// Get a non-nullable version of a type (exclude null and undefined).
    pub fn get_non_nullable_type(&mut self, type_id: TypeId) -> TypeId {
        self.get_type_with_facts(type_id, false, false)
    }

    /// Make a type readonly by setting the readonly flag on arrays and tuples.
    /// For other types, returns the type unchanged.
    pub fn make_type_readonly(&mut self, type_id: TypeId) -> TypeId {
        let Some(ty) = self.types.get(type_id) else {
            return type_id;
        };

        match ty {
            Type::Array(arr) => {
                // If already readonly, return as-is
                if arr.is_readonly {
                    return type_id;
                }
                // Create a new readonly array type
                let element_type = arr.element_type;
                self.types.create_array_type(element_type, true)
            }
            Type::Tuple(tup) => {
                // If already readonly, return as-is
                if tup.is_readonly {
                    return type_id;
                }
                // Create a new readonly tuple type
                let element_types = tup.element_types.clone();
                let has_optional = tup.has_optional_elements;
                let has_rest = tup.has_rest_element;
                self.types.create_tuple_type(element_types, has_optional, has_rest, true)
            }
            _ => {
                // For other types, readonly doesn't change anything structurally
                type_id
            }
        }
    }

    /// Narrow a type based on an instanceof guard.
    /// When `x instanceof Foo` is true, narrows x to Foo (or intersection with Foo).
    pub fn narrow_type_by_instanceof(&mut self, type_id: TypeId, target_type: TypeId) -> TypeId {
        // If the source type is any, unknown, or object, narrow to target
        let Some(source) = self.types.get(type_id) else {
            return target_type;
        };

        let source_flags = source.flags();

        // any or unknown narrows directly to the target
        if (source_flags & (type_flags::ANY | type_flags::UNKNOWN)) != 0 {
            return target_type;
        }

        // If it's a union type, filter to types that could be instanceof the target
        if let Type::Union(u) = source {
            let types = u.types.clone();
            let filtered_types: Vec<TypeId> = types.iter()
                .filter(|&&t| self.could_be_instanceof(t, target_type))
                .copied()
                .collect();

            if filtered_types.is_empty() {
                // No types could match, but instanceof succeeded, so result is target
                return target_type;
            } else if filtered_types.len() == 1 {
                return filtered_types[0];
            } else {
                return self.types.create_union_type(filtered_types);
            }
        }

        // For object types, check if they're related to target
        if (source_flags & type_flags::OBJECT) != 0 {
            // If source could be the target type, return the target
            if self.could_be_instanceof(type_id, target_type) {
                return target_type;
            }
        }

        // Default: return the target type for simplicity
        target_type
    }

    /// Narrow a type by excluding types that match instanceof.
    /// When `x instanceof Foo` is false, narrows x to exclude Foo.
    pub fn narrow_type_by_instanceof_negation(&mut self, type_id: TypeId, target_type: TypeId) -> TypeId {
        let Some(source) = self.types.get(type_id) else {
            return type_id;
        };

        // If it's a union type, filter out the target type and its subtypes
        if let Type::Union(u) = source {
            let types = u.types.clone();
            let remaining_types: Vec<TypeId> = types.iter()
                .filter(|&&t| !self.is_definitely_instanceof(t, target_type))
                .copied()
                .collect();

            if remaining_types.is_empty() {
                return self.types.never_type;
            } else if remaining_types.len() == 1 {
                return remaining_types[0];
            } else {
                return self.types.create_union_type(remaining_types);
            }
        }

        // For single types, if it's definitely the target, narrow to never
        if self.is_definitely_instanceof(type_id, target_type) {
            return self.types.never_type;
        }

        type_id
    }

    /// Check if a type could potentially be an instance of a target type.
    fn could_be_instanceof(&self, type_id: TypeId, target_type: TypeId) -> bool {
        // Same type always matches
        if type_id == target_type {
            return true;
        }

        let Some(source) = self.types.get(type_id) else {
            return false;
        };

        let source_flags = source.flags();

        // Any/unknown could be anything
        if (source_flags & (type_flags::ANY | type_flags::UNKNOWN)) != 0 {
            return true;
        }

        // Primitives can't be instanceof
        if (source_flags & (type_flags::STRING | type_flags::NUMBER | type_flags::BOOLEAN |
                           type_flags::UNDEFINED | type_flags::NULL | type_flags::VOID |
                           type_flags::NEVER)) != 0 {
            return false;
        }

        // Objects could potentially match
        if (source_flags & type_flags::OBJECT) != 0 {
            return true;
        }

        false
    }

    /// Check if a type is definitely an instance of a target type.
    /// For class types, we use nominal (identity-based) checking, not structural.
    fn is_definitely_instanceof(&self, type_id: TypeId, target_type: TypeId) -> bool {
        // Same type always matches
        if type_id == target_type {
            return true;
        }

        // For class/object types, use nominal comparison (TypeId identity)
        let Some(source_type) = self.types.get(type_id) else {
            return false;
        };
        let Some(target) = self.types.get(target_type) else {
            return false;
        };

        // If both are class/object types, they must have the same TypeId
        if matches!(source_type, Type::Object(_)) && matches!(target, Type::Object(_)) {
            return false;
        }

        // For non-class types, use assignability
        self.is_type_assignable_to(type_id, target_type)
    }

    /// Narrow a type by discriminant property value (x.kind === "circle").
    pub fn narrow_type_by_discriminant(&mut self, type_id: TypeId, property_name: &str, discriminant_value: &str) -> TypeId {
        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // Handle union types - filter to members that have the matching discriminant
        if let Type::Union(u) = typ {
            let types = u.types.clone();
            let matching_types: Vec<TypeId> = types.iter()
                .filter(|&&t| self.type_has_discriminant_value(t, property_name, discriminant_value))
                .copied()
                .collect();

            if matching_types.is_empty() {
                return self.types.never_type;
            } else if matching_types.len() == 1 {
                return matching_types[0];
            } else {
                return self.types.create_union_type(matching_types);
            }
        }

        // For single types, check if it has the matching discriminant
        if self.type_has_discriminant_value(type_id, property_name, discriminant_value) {
            return type_id;
        }

        self.types.never_type
    }

    /// Narrow a type by excluding discriminant property value (x.kind !== "circle").
    pub fn narrow_type_by_discriminant_negation(&mut self, type_id: TypeId, property_name: &str, discriminant_value: &str) -> TypeId {
        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // Handle union types - filter to members that DON'T have the matching discriminant
        if let Type::Union(u) = typ {
            let types = u.types.clone();
            let remaining_types: Vec<TypeId> = types.iter()
                .filter(|&&t| !self.type_has_discriminant_value(t, property_name, discriminant_value))
                .copied()
                .collect();

            if remaining_types.is_empty() {
                return self.types.never_type;
            } else if remaining_types.len() == 1 {
                return remaining_types[0];
            } else {
                return self.types.create_union_type(remaining_types);
            }
        }

        // For single types, if it has the discriminant, narrow to never
        if self.type_has_discriminant_value(type_id, property_name, discriminant_value) {
            return self.types.never_type;
        }

        type_id
    }

    /// Check if a type has a property with a specific string literal value.
    fn type_has_discriminant_value(&self, type_id: TypeId, property_name: &str, discriminant_value: &str) -> bool {
        let Some(typ) = self.types.get(type_id) else {
            return false;
        };

        if let Type::Object(obj) = typ {
            // Look up the property
            if let Some(symbol_id) = obj.members.get(property_name) {
                if let Some(&prop_type) = self.symbol_types.get(&symbol_id) {
                    // Check if the property type is a string literal matching the discriminant
                    if let Some(Type::Literal(lit)) = self.types.get(prop_type) {
                        if let LiteralValue::String(s) = &lit.value {
                            return s == discriminant_value;
                        }
                    }
                }
            }
        }

        false
    }

    /// Narrow a type by "prop" in x check.
    pub fn narrow_type_by_in(&mut self, type_id: TypeId, property_name: &str) -> TypeId {
        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // Handle union types - filter to members that have the property
        if let Type::Union(u) = typ {
            let types = u.types.clone();
            let matching_types: Vec<TypeId> = types.iter()
                .filter(|&&t| self.type_has_property(t, property_name))
                .copied()
                .collect();

            if matching_types.is_empty() {
                return self.types.never_type;
            } else if matching_types.len() == 1 {
                return matching_types[0];
            } else {
                return self.types.create_union_type(matching_types);
            }
        }

        // For single types, return as-is if it has the property
        if self.type_has_property(type_id, property_name) {
            return type_id;
        }

        self.types.never_type
    }

    /// Narrow a type by "prop" NOT in x check.
    pub fn narrow_type_by_in_negation(&mut self, type_id: TypeId, property_name: &str) -> TypeId {
        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // Handle union types - filter to members that DON'T have the property
        if let Type::Union(u) = typ {
            let types = u.types.clone();
            let remaining_types: Vec<TypeId> = types.iter()
                .filter(|&&t| !self.type_has_property(t, property_name))
                .copied()
                .collect();

            if remaining_types.is_empty() {
                return self.types.never_type;
            } else if remaining_types.len() == 1 {
                return remaining_types[0];
            } else {
                return self.types.create_union_type(remaining_types);
            }
        }

        // For single types, if it has the property, narrow to never
        if self.type_has_property(type_id, property_name) {
            return self.types.never_type;
        }

        type_id
    }

    /// Check if a type has a specific property.
    pub fn type_has_property(&self, type_id: TypeId, property_name: &str) -> bool {
        let Some(typ) = self.types.get(type_id) else {
            return false;
        };

        if let Type::Object(obj) = typ {
            return obj.members.has(property_name);
        }

        // For any type, assume it could have the property
        if typ.has_flags(type_flags::ANY) {
            return true;
        }

        false
    }

    // =========================================================================
    // Switch Statement Exhaustiveness Checking
    // =========================================================================

    /// Narrow a union type by a switch case value.
    pub fn narrow_type_by_switch_case(&mut self, type_id: TypeId, case_value: TypeId) -> TypeId {
        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // For union types, filter out members that match the case value
        if let Type::Union(u) = typ {
            let types = u.types.clone();
            let remaining_types: Vec<TypeId> = types.iter()
                .filter(|&&t| !self.types_are_equal(t, case_value))
                .copied()
                .collect();

            if remaining_types.is_empty() {
                return self.types.never_type;
            } else if remaining_types.len() == 1 {
                return remaining_types[0];
            } else {
                return self.types.create_union_type(remaining_types);
            }
        }

        // For non-union types, if it matches the case, narrow to never
        if self.types_are_equal(type_id, case_value) {
            return self.types.never_type;
        }

        type_id
    }

    /// Check if a switch statement over a discriminated union is exhaustive.
    pub fn check_switch_exhaustiveness(
        &mut self,
        discriminant_type: TypeId,
        case_values: &[TypeId],
        has_default: bool
    ) -> TypeId {
        // If there's a default clause, the switch is always exhaustive
        if has_default {
            return self.types.never_type;
        }

        // Start with the full discriminant type and narrow by each case
        let mut remaining_type = discriminant_type;

        for &case_value in case_values {
            remaining_type = self.narrow_type_by_switch_case(remaining_type, case_value);

            // Early exit if already never
            if remaining_type == self.types.never_type {
                break;
            }
        }

        remaining_type
    }

    /// Narrow a union type by removing members matching a discriminant value.
    pub fn narrow_by_discriminant_case(
        &mut self,
        type_id: TypeId,
        property_name: &str,
        discriminant_value: &str
    ) -> TypeId {
        let Some(typ) = self.types.get(type_id) else {
            return type_id;
        };

        // For union types, filter out members with matching discriminant
        if let Type::Union(u) = typ {
            let types = u.types.clone();
            let remaining_types: Vec<TypeId> = types.iter()
                .filter(|&&t| !self.type_has_discriminant_value(t, property_name, discriminant_value))
                .copied()
                .collect();

            if remaining_types.is_empty() {
                return self.types.never_type;
            } else if remaining_types.len() == 1 {
                return remaining_types[0];
            } else {
                return self.types.create_union_type(remaining_types);
            }
        }

        // For single type with matching discriminant, narrow to never
        if self.type_has_discriminant_value(type_id, property_name, discriminant_value) {
            return self.types.never_type;
        }

        type_id
    }

    /// Check if two types are considered equal for switch case matching.
    fn types_are_equal(&self, type1: TypeId, type2: TypeId) -> bool {
        if type1 == type2 {
            return true;
        }

        let Some(t1) = self.types.get(type1) else {
            return false;
        };
        let Some(t2) = self.types.get(type2) else {
            return false;
        };

        // Compare literal values
        if let (Type::Literal(lit1), Type::Literal(lit2)) = (t1, t2) {
            return lit1.value == lit2.value;
        }

        false
    }

    // =========================================================================
    // Type Guard Analysis
    // =========================================================================

    /// Analyze a condition expression and extract type guard information.
    pub fn get_type_guard_from_expression(&mut self, expr: NodeIndex) -> Option<TypeGuard> {
        use crate::parser::Node;
        use crate::scanner::SyntaxKind;

        let node = self.node_arena.get(expr)?;

        match node {
            // Binary expressions: typeof x === "string", x instanceof Foo, x !== null
            Node::BinaryExpression(be) => {
                self.get_type_guard_from_binary_expression(be)
            }

            // Prefix unary: !x (falsy check)
            Node::PrefixUnaryExpression(pue) => {
                // !x means x is falsy
                if pue.operator == SyntaxKind::ExclamationToken {
                    // Get the inner guard and negate it
                    if let Some(guard) = self.get_type_guard_from_expression(pue.operand) {
                        return Some(self.negate_type_guard(guard));
                    }
                    // Just !x without a nested guard - treat as truthiness check
                    return Some(TypeGuard::Truthiness {
                        target: pue.operand,
                        is_truthy: false,
                    });
                }
                None
            }

            // Parenthesized expression: (x === "string")
            Node::ParenthesizedExpression(pe) => {
                self.get_type_guard_from_expression(pe.expression)
            }

            // Simple expression - treat as truthiness check
            _ => Some(TypeGuard::Truthiness {
                target: expr,
                is_truthy: true,
            }),
        }
    }

    /// Extract type guard from a binary expression.
    fn get_type_guard_from_binary_expression(&mut self, be: &crate::parser::BinaryExpression) -> Option<TypeGuard> {
        use crate::parser::Node;
        use crate::scanner::SyntaxKind;

        // Check for typeof guard: typeof x === "string"
        if let Some(Node::PrefixUnaryExpression(pue)) = self.node_arena.get(be.left) {
            if pue.operator == SyntaxKind::TypeOfKeyword {
                // Check if right side is a string literal
                if let Some(Node::StringLiteral(sl)) = self.node_arena.get(be.right) {
                    let is_equality = matches!(be.operator_token,
                        SyntaxKind::EqualsEqualsToken | SyntaxKind::EqualsEqualsEqualsToken);
                    let is_inequality = matches!(be.operator_token,
                        SyntaxKind::ExclamationEqualsToken | SyntaxKind::ExclamationEqualsEqualsToken);

                    if is_equality || is_inequality {
                        return Some(TypeGuard::Typeof {
                            target: pue.operand,
                            typeof_result: sl.text.clone(),
                            is_equality,
                        });
                    }
                }
            }
        }

        // Check for instanceof guard: x instanceof Foo
        if be.operator_token == SyntaxKind::InstanceOfKeyword {
            let constructor_type = self.get_type_of_node(be.right);
            return Some(TypeGuard::Instanceof {
                target: be.left,
                constructor_type,
                is_positive: true,
            });
        }

        // Check for "in" guard: "prop" in x
        if be.operator_token == SyntaxKind::InKeyword {
            if let Some(Node::StringLiteral(sl)) = self.node_arena.get(be.left) {
                return Some(TypeGuard::In {
                    target: be.right,
                    property_name: sl.text.clone(),
                    is_positive: true,
                });
            }
        }

        // Check for discriminant guard and null/undefined guards
        let is_equality = matches!(be.operator_token,
            SyntaxKind::EqualsEqualsToken | SyntaxKind::EqualsEqualsEqualsToken);
        let is_inequality = matches!(be.operator_token,
            SyntaxKind::ExclamationEqualsToken | SyntaxKind::ExclamationEqualsEqualsToken);

        if is_equality || is_inequality {
            // Check for property access on left side: x.kind === "value" (discriminant)
            if let Some(Node::PropertyAccessExpression(pa)) = self.node_arena.get(be.left) {
                if let Some(Node::Identifier(prop_name)) = self.node_arena.get(pa.name) {
                    if let Some(Node::StringLiteral(sl)) = self.node_arena.get(be.right) {
                        return Some(TypeGuard::Discriminant {
                            target: pa.expression, // The base object (x)
                            property_name: prop_name.escaped_text.clone(),
                            discriminant_value: sl.text.clone(),
                            is_equality,
                        });
                    }
                }
            }

            // Check if right side is null or undefined
            if let Some(Node::Token(base)) = self.node_arena.get(be.right) {
                if base.kind == SyntaxKind::NullKeyword as u16 {
                    return Some(TypeGuard::Truthiness {
                        target: be.left,
                        is_truthy: is_inequality, // x !== null means x is truthy (non-null)
                    });
                }
            }
            if let Some(Node::Identifier(id)) = self.node_arena.get(be.right) {
                if id.escaped_text == "undefined" {
                    return Some(TypeGuard::Truthiness {
                        target: be.left,
                        is_truthy: is_inequality,
                    });
                }
            }
        }

        None
    }

    /// Negate a type guard (for use with ! or else branches).
    pub fn negate_type_guard(&self, guard: TypeGuard) -> TypeGuard {
        match guard {
            TypeGuard::Typeof { target, typeof_result, is_equality } => {
                TypeGuard::Typeof { target, typeof_result, is_equality: !is_equality }
            }
            TypeGuard::Instanceof { target, constructor_type, is_positive } => {
                TypeGuard::Instanceof { target, constructor_type, is_positive: !is_positive }
            }
            TypeGuard::Truthiness { target, is_truthy } => {
                TypeGuard::Truthiness { target, is_truthy: !is_truthy }
            }
            TypeGuard::Discriminant { target, property_name, discriminant_value, is_equality } => {
                TypeGuard::Discriminant { target, property_name, discriminant_value, is_equality: !is_equality }
            }
            TypeGuard::In { target, property_name, is_positive } => {
                TypeGuard::In { target, property_name, is_positive: !is_positive }
            }
        }
    }

    /// Apply a type guard to narrow a type.
    pub fn apply_type_guard(&mut self, type_id: TypeId, guard: &TypeGuard) -> TypeId {
        match guard {
            TypeGuard::Typeof { typeof_result, is_equality, .. } => {
                if *is_equality {
                    self.narrow_type_by_typeof(type_id, typeof_result)
                } else {
                    self.narrow_type_by_typeof_negation(type_id, typeof_result)
                }
            }
            TypeGuard::Instanceof { constructor_type, is_positive, .. } => {
                if *is_positive {
                    self.narrow_type_by_instanceof(type_id, *constructor_type)
                } else {
                    self.narrow_type_by_instanceof_negation(type_id, *constructor_type)
                }
            }
            TypeGuard::Truthiness { is_truthy, .. } => {
                if *is_truthy {
                    // Remove null and undefined for truthy check
                    self.get_non_nullable_type(type_id)
                } else {
                    // For falsy, keep only null/undefined (if present in union)
                    type_id // TODO: Implement proper falsy narrowing
                }
            }
            TypeGuard::Discriminant { property_name, discriminant_value, is_equality, .. } => {
                if *is_equality {
                    self.narrow_type_by_discriminant(type_id, property_name, discriminant_value)
                } else {
                    self.narrow_type_by_discriminant_negation(type_id, property_name, discriminant_value)
                }
            }
            TypeGuard::In { property_name, is_positive, .. } => {
                if *is_positive {
                    self.narrow_type_by_in(type_id, property_name)
                } else {
                    self.narrow_type_by_in_negation(type_id, property_name)
                }
            }
        }
    }

    /// Get the narrowed type of a symbol reference given its flow node.
    /// Uses a worklist algorithm to handle control flow merges correctly.
    pub fn get_narrowed_type_at_flow(
        &mut self,
        symbol_id: SymbolId,
        base_type: TypeId,
        flow_node_id: crate::binder::FlowNodeId,
        flow_arena: &FlowNodeArena,
    ) -> TypeId {
        use std::collections::HashSet;

        // Worklist of (flow_node, current_type) pairs
        let mut worklist: Vec<(crate::binder::FlowNodeId, TypeId)> = vec![(flow_node_id, base_type)];
        let mut visited: HashSet<crate::binder::FlowNodeId> = HashSet::new();
        let mut result_types: Vec<TypeId> = Vec::new();

        while let Some((current_flow, current_type)) = worklist.pop() {
            if current_flow.is_none() || visited.contains(&current_flow) {
                // Reached end of path or already visited - this type contributes to result
                if current_type != base_type || result_types.is_empty() {
                    result_types.push(current_type);
                }
                continue;
            }
            visited.insert(current_flow);

            let Some(flow) = flow_arena.get(current_flow) else {
                result_types.push(current_type);
                continue;
            };

            // Apply narrowing if this is a condition node
            let narrowed_type = if flow.has_any_flags(flow_flags::TRUE_CONDITION | flow_flags::FALSE_CONDITION) {
                if !flow.node.is_none() {
                    if let Some(guard) = self.get_type_guard_from_expression(flow.node) {
                        if self.guard_applies_to_symbol(&guard, symbol_id) {
                            let effective_guard = if flow.has_flags(flow_flags::TRUE_CONDITION) {
                                guard
                            } else {
                                self.negate_type_guard(guard)
                            };
                            self.apply_type_guard(current_type, &effective_guard)
                        } else {
                            current_type
                        }
                    } else {
                        current_type
                    }
                } else {
                    current_type
                }
            } else {
                current_type
            };

            // Add ALL antecedents to worklist (not just first!)
            if flow.antecedent.is_empty() {
                result_types.push(narrowed_type);
            } else {
                for &antecedent in &flow.antecedent {
                    worklist.push((antecedent, narrowed_type));
                }
            }
        }

        // If no results, return base type
        if result_types.is_empty() {
            return base_type;
        }

        // If only one result type, return it
        if result_types.len() == 1 {
            return result_types[0];
        }

        // Multiple paths converged - create union of all narrowed types
        // Remove duplicates first
        let unique_types: Vec<TypeId> = result_types
            .into_iter()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        if unique_types.len() == 1 {
            unique_types[0]
        } else {
            self.types.create_union_type(unique_types)
        }
    }

    /// Check if a type guard applies to a specific symbol.
    fn guard_applies_to_symbol(&self, guard: &TypeGuard, symbol_id: SymbolId) -> bool {
        use crate::parser::Node;

        let target = match guard {
            TypeGuard::Typeof { target, .. } => *target,
            TypeGuard::Instanceof { target, .. } => *target,
            TypeGuard::Truthiness { target, .. } => *target,
            TypeGuard::Discriminant { target, .. } => *target,
            TypeGuard::In { target, .. } => *target,
        };

        // Check if the target expression refers to this symbol
        if let Some(Node::Identifier(id)) = self.node_arena.get(target) {
            if let Some(target_symbol) = self.file_locals.get(&id.escaped_text) {
                return target_symbol == symbol_id;
            }
        }
        // Handle property access expressions (x.kind === "foo")
        if let Some(Node::PropertyAccessExpression(pa)) = self.node_arena.get(target) {
            if let Some(Node::Identifier(id)) = self.node_arena.get(pa.expression) {
                if let Some(target_symbol) = self.file_locals.get(&id.escaped_text) {
                    return target_symbol == symbol_id;
                }
            }
        }

        false
    }
}
