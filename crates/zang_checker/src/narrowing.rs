//! Control Flow Type Narrowing
//!
//! Narrows types based on control flow analysis (type guards, if statements, etc.)

use std::collections::HashMap;
use zang_core::{InternedString, Span};
use zang_parser::{
    Expression, Statement, BinaryOperator, UnaryOperator, Identifier,
};
use crate::types::{ResolvedType, LiteralType, ObjectType};

/// Represents the narrowed type of a variable at a specific point in control flow
#[derive(Debug, Clone)]
pub struct NarrowedType {
    /// The original (declared) type
    pub original: ResolvedType,
    /// The narrowed type
    pub narrowed: ResolvedType,
    /// The span where the narrowing occurred
    pub narrowed_at: Span,
}

/// Control flow graph node for type narrowing
#[derive(Debug, Clone)]
pub struct FlowNode {
    /// Node ID
    pub id: u32,
    /// Parent nodes (for merging)
    pub antecedents: Vec<u32>,
    /// Type narrowings at this node
    pub narrowings: HashMap<InternedString, NarrowedType>,
    /// Node kind
    pub kind: FlowNodeKind,
}

/// Kind of control flow node
#[derive(Debug, Clone)]
pub enum FlowNodeKind {
    /// Start of function/block
    Start,
    /// Assignment
    Assignment {
        target: InternedString,
        assigned_type: ResolvedType,
    },
    /// Condition (if, while, etc.)
    Condition {
        /// True branch node ID
        true_branch: u32,
        /// False branch node ID
        false_branch: u32,
    },
    /// Branch merge point
    BranchMerge,
    /// End of function/block
    End,
    /// Call expression (might throw or have side effects)
    Call,
    /// Return statement
    Return,
    /// Narrowing from type guard
    TypeGuard {
        target: InternedString,
        narrowed_type: ResolvedType,
    },
}

/// Type narrowing context
pub struct NarrowingContext {
    /// Current type narrowings by variable name
    narrowings: HashMap<InternedString, ResolvedType>,
    /// Stack of saved states for branch handling
    state_stack: Vec<HashMap<InternedString, ResolvedType>>,
    /// Next flow node ID
    next_node_id: u32,
}

impl NarrowingContext {
    /// Creates a new narrowing context
    pub fn new() -> Self {
        Self {
            narrowings: HashMap::new(),
            state_stack: Vec::new(),
            next_node_id: 1,
        }
    }

    /// Gets the narrowed type for a variable, or None if not narrowed
    pub fn get_narrowed_type(&self, name: &InternedString) -> Option<&ResolvedType> {
        self.narrowings.get(name)
    }

    /// Sets the narrowed type for a variable
    pub fn set_narrowed_type(&mut self, name: InternedString, ty: ResolvedType) {
        self.narrowings.insert(name, ty);
    }

    /// Removes a narrowing (e.g., after assignment)
    pub fn remove_narrowing(&mut self, name: &InternedString) {
        self.narrowings.remove(name);
    }

    /// Saves the current state for branching
    pub fn save_state(&mut self) {
        self.state_stack.push(self.narrowings.clone());
    }

    /// Restores the previous state
    pub fn restore_state(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.narrowings = state;
        }
    }

    /// Merges two branch states
    pub fn merge_states(&mut self, other: &HashMap<InternedString, ResolvedType>) {
        // For each narrowing in the current state, check if it exists in other
        let mut to_remove = Vec::new();
        let mut to_widen = Vec::new();

        for (name, ty) in &self.narrowings {
            if let Some(other_ty) = other.get(name) {
                // Both branches have a narrowing - create union
                if !types_equal(ty, other_ty) {
                    to_widen.push((name.clone(), ty.clone(), other_ty.clone()));
                }
            } else {
                // Only one branch has narrowing - remove it
                to_remove.push(name.clone());
            }
        }

        for name in to_remove {
            self.narrowings.remove(&name);
        }

        for (name, ty1, ty2) in to_widen {
            self.narrowings.insert(name, create_union(ty1, ty2));
        }
    }

    /// Allocates a new flow node ID
    fn alloc_node_id(&mut self) -> u32 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }
}

impl Default for NarrowingContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Type narrower for control flow analysis
pub struct TypeNarrower<'a> {
    /// The narrowing context
    context: &'a mut NarrowingContext,
    /// Type map for looking up variable types
    type_map: &'a HashMap<InternedString, ResolvedType>,
}

impl<'a> TypeNarrower<'a> {
    /// Creates a new type narrower
    pub fn new(
        context: &'a mut NarrowingContext,
        type_map: &'a HashMap<InternedString, ResolvedType>,
    ) -> Self {
        Self { context, type_map }
    }

    /// Narrows types based on a condition expression being true
    pub fn narrow_by_condition(&mut self, condition: &Expression, assume_true: bool) {
        match condition {
            Expression::Binary(bin) => {
                self.narrow_by_binary_expression(bin, assume_true);
            }
            Expression::Unary(unary) => {
                self.narrow_by_unary_expression(unary, assume_true);
            }
            Expression::Identifier(ident) => {
                // Variable used as condition - narrow to truthy type
                self.narrow_by_truthiness(ident, assume_true);
            }
            Expression::Call(call) => {
                // Could be a type predicate function
                self.narrow_by_call(call, assume_true);
            }
            Expression::Parenthesized(inner) => {
                self.narrow_by_condition(inner, assume_true);
            }
            _ => {}
        }
    }

    /// Narrows types based on a binary expression
    fn narrow_by_binary_expression(&mut self, bin: &zang_parser::BinaryExpression, assume_true: bool) {
        match bin.operator {
            // Equality checks
            BinaryOperator::StrictEquals | BinaryOperator::Equals => {
                if assume_true {
                    self.narrow_by_equality(&bin.left, &bin.right, true);
                }
            }
            BinaryOperator::StrictNotEquals | BinaryOperator::NotEquals => {
                if assume_true {
                    self.narrow_by_equality(&bin.left, &bin.right, false);
                }
            }

            // typeof checks
            BinaryOperator::LogicalAnd => {
                if assume_true {
                    // Both sides must be true
                    self.narrow_by_condition(&bin.left, true);
                    self.narrow_by_condition(&bin.right, true);
                } else {
                    // At least one side is false - we can't narrow safely
                }
            }
            BinaryOperator::LogicalOr => {
                if !assume_true {
                    // Both sides must be false
                    self.narrow_by_condition(&bin.left, false);
                    self.narrow_by_condition(&bin.right, false);
                } else {
                    // At least one side is true - we can't narrow safely without branching
                }
            }

            // instanceof
            BinaryOperator::InstanceOf => {
                self.narrow_by_instanceof(&bin.left, &bin.right, assume_true);
            }

            // in operator
            BinaryOperator::In => {
                self.narrow_by_in_operator(&bin.left, &bin.right, assume_true);
            }

            _ => {}
        }
    }

    /// Narrows types based on a unary expression
    fn narrow_by_unary_expression(&mut self, unary: &zang_parser::UnaryExpression, assume_true: bool) {
        match unary.operator {
            UnaryOperator::LogicalNot => {
                // !x being true means x is false
                self.narrow_by_condition(&unary.operand, !assume_true);
            }
            UnaryOperator::TypeOf => {
                // typeof alone doesn't narrow
            }
            _ => {}
        }
    }

    /// Narrows types based on equality comparison
    fn narrow_by_equality(&mut self, left: &Expression, right: &Expression, is_equal: bool) {
        // typeof x === "string"
        if let Expression::Unary(unary) = left {
            if matches!(unary.operator, UnaryOperator::TypeOf) {
                if let Expression::Identifier(ident) = &*unary.operand {
                    if let Expression::StringLiteral(lit) = right {
                        self.narrow_by_typeof(ident, &lit.value, is_equal);
                        return;
                    }
                }
            }
        }

        // "string" === typeof x (reversed)
        if let Expression::Unary(unary) = right {
            if matches!(unary.operator, UnaryOperator::TypeOf) {
                if let Expression::Identifier(ident) = &*unary.operand {
                    if let Expression::StringLiteral(lit) = left {
                        self.narrow_by_typeof(ident, &lit.value, is_equal);
                        return;
                    }
                }
            }
        }

        // x === null
        if let Expression::Identifier(ident) = left {
            if let Expression::NullLiteral(_) = right {
                self.narrow_to_null_or_not_null(ident, is_equal);
                return;
            }
        }

        // null === x
        if let Expression::NullLiteral(_) = left {
            if let Expression::Identifier(ident) = right {
                self.narrow_to_null_or_not_null(ident, is_equal);
                return;
            }
        }

        // x === undefined (via identifier named "undefined")
        if let Expression::Identifier(left_ident) = left {
            if let Expression::Identifier(right_ident) = right {
                // Check if right is "undefined"
                if let Some(ty) = self.type_map.get(&right_ident.name) {
                    if matches!(ty, ResolvedType::Undefined) {
                        self.narrow_to_undefined_or_not(left_ident, is_equal);
                    }
                }
            }
        }
    }

    /// Narrows type based on typeof check
    fn narrow_by_typeof(&mut self, ident: &Identifier, type_string: &InternedString, is_equal: bool) {
        // Get the current type for the identifier
        let current_type = self.context
            .get_narrowed_type(&ident.name)
            .cloned()
            .or_else(|| self.type_map.get(&ident.name).cloned())
            .unwrap_or(ResolvedType::Any);

        // Determine the narrowed type based on the typeof string
        let lookup = self.type_map.get(type_string);
        let narrowed = match lookup {
            Some(ResolvedType::Literal(LiteralType::String(s))) => {
                match s.as_str() {
                    "string" => Some(ResolvedType::String),
                    "number" => Some(ResolvedType::Number),
                    "boolean" => Some(ResolvedType::Boolean),
                    "undefined" => Some(ResolvedType::Undefined),
                    "object" => Some(ResolvedType::Object(ObjectType {
                        properties: Vec::new(),
                        call_signatures: Vec::new(),
                        construct_signatures: Vec::new(),
                        index_signatures: Vec::new(),
                    })),
                    "function" => Some(ResolvedType::Function(crate::types::FunctionType {
                        type_parameters: Vec::new(),
                        parameters: Vec::new(),
                        return_type: Box::new(ResolvedType::Any),
                    })),
                    "bigint" => Some(ResolvedType::BigInt),
                    "symbol" => Some(ResolvedType::Symbol),
                    _ => None,
                }
            }
            _ => None,
        };

        if let Some(narrowed_type) = narrowed {
            if is_equal {
                // typeof x === "string" - narrow to string
                let result = narrow_type_to(&current_type, &narrowed_type);
                self.context.set_narrowed_type(ident.name, result);
            } else {
                // typeof x !== "string" - remove string from union
                let result = narrow_type_away_from(&current_type, &narrowed_type);
                self.context.set_narrowed_type(ident.name, result);
            }
        }
    }

    /// Narrows based on truthiness (variable used as condition)
    fn narrow_by_truthiness(&mut self, ident: &Identifier, assume_truthy: bool) {
        let current_type = self.context
            .get_narrowed_type(&ident.name)
            .cloned()
            .or_else(|| self.type_map.get(&ident.name).cloned())
            .unwrap_or(ResolvedType::Any);

        let narrowed = if assume_truthy {
            // Remove falsy types (null, undefined, false, 0, "", NaN)
            remove_falsy_types(&current_type)
        } else {
            // We can't narrow to just falsy types easily
            current_type
        };

        self.context.set_narrowed_type(ident.name, narrowed);
    }

    /// Narrows to null or non-null
    fn narrow_to_null_or_not_null(&mut self, ident: &Identifier, is_null: bool) {
        let current_type = self.context
            .get_narrowed_type(&ident.name)
            .cloned()
            .or_else(|| self.type_map.get(&ident.name).cloned())
            .unwrap_or(ResolvedType::Any);

        let narrowed = if is_null {
            ResolvedType::Null
        } else {
            remove_null(&current_type)
        };

        self.context.set_narrowed_type(ident.name, narrowed);
    }

    /// Narrows to undefined or non-undefined
    fn narrow_to_undefined_or_not(&mut self, ident: &Identifier, is_undefined: bool) {
        let current_type = self.context
            .get_narrowed_type(&ident.name)
            .cloned()
            .or_else(|| self.type_map.get(&ident.name).cloned())
            .unwrap_or(ResolvedType::Any);

        let narrowed = if is_undefined {
            ResolvedType::Undefined
        } else {
            remove_undefined(&current_type)
        };

        self.context.set_narrowed_type(ident.name, narrowed);
    }

    /// Narrows based on instanceof
    fn narrow_by_instanceof(&mut self, left: &Expression, _right: &Expression, assume_true: bool) {
        if let Expression::Identifier(ident) = left {
            // TODO: Look up the constructor type and narrow to its instance type
            // For now, we just keep the current type
            if assume_true {
                // Would narrow to instance type
            }
        }
    }

    /// Narrows based on in operator
    fn narrow_by_in_operator(&mut self, left: &Expression, right: &Expression, assume_true: bool) {
        if let Expression::Identifier(obj_ident) = right {
            if let Expression::StringLiteral(_prop_name) = left {
                // "prop" in x - could narrow x to include that property
                // TODO: Implement discriminated union narrowing
            }
        }
    }

    /// Narrows based on a call expression (type predicates)
    fn narrow_by_call(&mut self, call: &zang_parser::CallExpression, assume_true: bool) {
        // TODO: Implement type predicate narrowing
        // e.g., if (isString(x)) where isString is (x: any) => x is string
    }

    /// Processes an if statement for narrowing
    pub fn process_if_statement(&mut self, if_stmt: &zang_parser::IfStatement) {
        // Save state before condition
        self.context.save_state();

        // Narrow types based on condition being true
        self.narrow_by_condition(&if_stmt.condition, true);

        // Process then branch (with narrowed types)
        // After then branch, save state
        let then_state = self.context.narrowings.clone();

        // Restore original state and narrow based on condition being false
        self.context.restore_state();

        if if_stmt.else_statement.is_some() {
            self.context.save_state();
            self.narrow_by_condition(&if_stmt.condition, false);

            // After else branch, merge with then branch
            let else_state = self.context.narrowings.clone();
            self.context.restore_state();

            // Merge the two branch states
            self.context.merge_states(&then_state);
            self.context.merge_states(&else_state);
        } else {
            // No else branch - merge then state with original
            self.context.merge_states(&then_state);
        }
    }
}

/// Checks if two types are equal (simple comparison)
fn types_equal(a: &ResolvedType, b: &ResolvedType) -> bool {
    match (a, b) {
        (ResolvedType::Any, ResolvedType::Any) => true,
        (ResolvedType::Unknown, ResolvedType::Unknown) => true,
        (ResolvedType::Never, ResolvedType::Never) => true,
        (ResolvedType::Void, ResolvedType::Void) => true,
        (ResolvedType::Undefined, ResolvedType::Undefined) => true,
        (ResolvedType::Null, ResolvedType::Null) => true,
        (ResolvedType::String, ResolvedType::String) => true,
        (ResolvedType::Number, ResolvedType::Number) => true,
        (ResolvedType::Boolean, ResolvedType::Boolean) => true,
        (ResolvedType::BigInt, ResolvedType::BigInt) => true,
        (ResolvedType::Symbol, ResolvedType::Symbol) => true,
        _ => false,
    }
}

/// Creates a union of two types
fn create_union(a: ResolvedType, b: ResolvedType) -> ResolvedType {
    if types_equal(&a, &b) {
        return a;
    }

    match (&a, &b) {
        (ResolvedType::Union(types_a), ResolvedType::Union(types_b)) => {
            let mut combined = types_a.clone();
            for t in types_b {
                if !combined.iter().any(|existing| types_equal(existing, t)) {
                    combined.push(t.clone());
                }
            }
            ResolvedType::Union(combined)
        }
        (ResolvedType::Union(types), other) | (other, ResolvedType::Union(types)) => {
            let mut result = types.clone();
            if !result.iter().any(|t| types_equal(t, other)) {
                result.push(other.clone());
            }
            ResolvedType::Union(result)
        }
        _ => ResolvedType::Union(vec![a, b]),
    }
}

/// Narrows a type to a specific type (intersection)
fn narrow_type_to(current: &ResolvedType, target: &ResolvedType) -> ResolvedType {
    match current {
        ResolvedType::Union(types) => {
            // Keep only types that are compatible with target
            let filtered: Vec<_> = types
                .iter()
                .filter(|t| is_type_related(t, target))
                .cloned()
                .collect();

            if filtered.is_empty() {
                ResolvedType::Never
            } else if filtered.len() == 1 {
                filtered[0].clone()
            } else {
                ResolvedType::Union(filtered)
            }
        }
        ResolvedType::Any | ResolvedType::Unknown => target.clone(),
        _ => {
            if is_type_related(current, target) {
                target.clone()
            } else {
                ResolvedType::Never
            }
        }
    }
}

/// Narrows a type away from a specific type (excludes)
fn narrow_type_away_from(current: &ResolvedType, excluded: &ResolvedType) -> ResolvedType {
    match current {
        ResolvedType::Union(types) => {
            let filtered: Vec<_> = types
                .iter()
                .filter(|t| !is_type_related(t, excluded))
                .cloned()
                .collect();

            if filtered.is_empty() {
                ResolvedType::Never
            } else if filtered.len() == 1 {
                filtered[0].clone()
            } else {
                ResolvedType::Union(filtered)
            }
        }
        _ => {
            if is_type_related(current, excluded) {
                ResolvedType::Never
            } else {
                current.clone()
            }
        }
    }
}

/// Removes falsy types from a type
fn remove_falsy_types(ty: &ResolvedType) -> ResolvedType {
    match ty {
        ResolvedType::Union(types) => {
            let filtered: Vec<_> = types
                .iter()
                .filter(|t| !is_falsy_type(t))
                .cloned()
                .collect();

            if filtered.is_empty() {
                ResolvedType::Never
            } else if filtered.len() == 1 {
                filtered[0].clone()
            } else {
                ResolvedType::Union(filtered)
            }
        }
        _ if is_falsy_type(ty) => ResolvedType::Never,
        _ => ty.clone(),
    }
}

/// Checks if a type is falsy
fn is_falsy_type(ty: &ResolvedType) -> bool {
    match ty {
        ResolvedType::Null | ResolvedType::Undefined => true,
        ResolvedType::Literal(LiteralType::Boolean(false)) => true,
        ResolvedType::Literal(LiteralType::Number(n)) => *n == 0.0,
        ResolvedType::Literal(LiteralType::String(s)) => s.is_empty(),
        _ => false,
    }
}

/// Removes null from a type
fn remove_null(ty: &ResolvedType) -> ResolvedType {
    narrow_type_away_from(ty, &ResolvedType::Null)
}

/// Removes undefined from a type
fn remove_undefined(ty: &ResolvedType) -> ResolvedType {
    narrow_type_away_from(ty, &ResolvedType::Undefined)
}

/// Checks if a type is related to another (simple subtype check)
fn is_type_related(sub: &ResolvedType, sup: &ResolvedType) -> bool {
    match (sub, sup) {
        _ if types_equal(sub, sup) => true,
        (_, ResolvedType::Any) => true,
        (_, ResolvedType::Unknown) => true,
        (ResolvedType::Never, _) => true,
        (ResolvedType::Literal(LiteralType::String(_)), ResolvedType::String) => true,
        (ResolvedType::Literal(LiteralType::Number(_)), ResolvedType::Number) => true,
        (ResolvedType::Literal(LiteralType::Boolean(_)), ResolvedType::Boolean) => true,
        (ResolvedType::Literal(LiteralType::BigInt(_)), ResolvedType::BigInt) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_narrowing_context_creation() {
        let ctx = NarrowingContext::new();
        assert!(ctx.narrowings.is_empty());
    }

    #[test]
    fn test_set_and_get_narrowed_type() {
        let mut ctx = NarrowingContext::new();
        let name = InternedString::from_raw(1);

        ctx.set_narrowed_type(name, ResolvedType::String);
        assert!(matches!(ctx.get_narrowed_type(&name), Some(ResolvedType::String)));
    }

    #[test]
    fn test_save_restore_state() {
        let mut ctx = NarrowingContext::new();
        let name = InternedString::from_raw(1);

        ctx.set_narrowed_type(name, ResolvedType::String);
        ctx.save_state();
        ctx.set_narrowed_type(name, ResolvedType::Number);

        assert!(matches!(ctx.get_narrowed_type(&name), Some(ResolvedType::Number)));

        ctx.restore_state();
        assert!(matches!(ctx.get_narrowed_type(&name), Some(ResolvedType::String)));
    }

    #[test]
    fn test_remove_null() {
        let union = ResolvedType::Union(vec![
            ResolvedType::String,
            ResolvedType::Null,
        ]);

        let result = remove_null(&union);
        assert!(matches!(result, ResolvedType::String));
    }

    #[test]
    fn test_narrow_type_to() {
        let union = ResolvedType::Union(vec![
            ResolvedType::String,
            ResolvedType::Number,
        ]);

        let result = narrow_type_to(&union, &ResolvedType::String);
        assert!(matches!(result, ResolvedType::String));
    }

    #[test]
    fn test_is_falsy_type() {
        assert!(is_falsy_type(&ResolvedType::Null));
        assert!(is_falsy_type(&ResolvedType::Undefined));
        assert!(!is_falsy_type(&ResolvedType::String));
        assert!(!is_falsy_type(&ResolvedType::Number));
    }
}
