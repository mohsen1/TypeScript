//! Control Flow Analysis for type narrowing.
//!
//! This module provides flow-sensitive type analysis that walks the control flow
//! graph backwards from identifier usages to determine narrowed types.
//!
//! Example:
//! ```typescript
//! function foo(x: string | number) {
//!     if (typeof x === "string") {
//!         // FlowAnalyzer walks back and sees TRUE_CONDITION (typeof x === "string")
//!         // Returns: string (narrowed from string | number)
//!         console.log(x.length);
//!     } else {
//!         // FlowAnalyzer sees FALSE_CONDITION
//!         // Returns: number
//!         console.log(x.toFixed(2));
//!     }
//! }
//! ```

use crate::binder::{FlowNode, FlowNodeId, flow_flags};
use crate::parser::thin_node::ThinNodeArena;
use crate::parser::{NodeIndex, syntax_kind_ext};
use crate::scanner::SyntaxKind;
use crate::solver::{LiteralValue, TypeId, TypeInterner, TypeKey, NarrowingContext};
use crate::thin_binder::ThinBinderState;

/// Flow analyzer for control flow-based type narrowing.
///
/// Walks the control flow graph backwards from a reference point to determine
/// what type narrowing applies at that location.
pub struct FlowAnalyzer<'a> {
    arena: &'a ThinNodeArena,
    binder: &'a ThinBinderState,
    interner: &'a TypeInterner,
}

impl<'a> FlowAnalyzer<'a> {
    /// Create a new FlowAnalyzer.
    pub fn new(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        interner: &'a TypeInterner,
    ) -> Self {
        Self { arena, binder, interner }
    }

    /// Get the narrowed type of a symbol at a specific flow node.
    ///
    /// This walks backwards through the flow graph, applying narrowing operations
    /// when it encounters condition nodes.
    pub fn get_flow_type(
        &self,
        reference: NodeIndex,
        initial_type: TypeId,
        flow_node: FlowNodeId,
    ) -> TypeId {
        if flow_node.is_none() {
            return initial_type;
        }

        self.check_flow(reference, initial_type, flow_node, &mut Vec::new())
    }

    /// Recursive flow graph traversal with cycle detection.
    fn check_flow(
        &self,
        reference: NodeIndex,
        type_id: TypeId,
        flow_id: FlowNodeId,
        visited: &mut Vec<FlowNodeId>,
    ) -> TypeId {
        // Cycle detection
        if visited.contains(&flow_id) {
            return type_id;
        }
        visited.push(flow_id);

        let Some(flow) = self.binder.flow_nodes.get(flow_id) else {
            return type_id;
        };

        // Handle different flow node types
        if flow.has_any_flags(flow_flags::BRANCH_LABEL) {
            return self.handle_branch_label(reference, type_id, flow, visited);
        }

        if flow.has_any_flags(flow_flags::LOOP_LABEL) {
            return self.handle_loop_label(reference, type_id, flow, visited);
        }

        if flow.has_any_flags(flow_flags::CONDITION) {
            return self.handle_condition(reference, type_id, flow, visited);
        }

        if flow.has_any_flags(flow_flags::START) {
            // Reached start of flow - return initial type
            return type_id;
        }

        // Default: continue to antecedent
        if let Some(&ant) = flow.antecedent.first() {
            self.check_flow(reference, type_id, ant, visited)
        } else {
            type_id
        }
    }

    /// Handle branch label (merge point) - union of types from all branches.
    fn handle_branch_label(
        &self,
        reference: NodeIndex,
        type_id: TypeId,
        flow: &FlowNode,
        visited: &mut Vec<FlowNodeId>,
    ) -> TypeId {
        if flow.antecedent.is_empty() {
            return type_id;
        }

        // Get types from all incoming branches
        let branch_types: Vec<TypeId> = flow.antecedent.iter()
            .map(|&ant| self.check_flow(reference, type_id, ant, &mut visited.clone()))
            .collect();

        // Union the types from different branches
        if branch_types.is_empty() {
            type_id
        } else if branch_types.len() == 1 {
            branch_types[0]
        } else {
            self.interner.union(branch_types)
        }
    }

    /// Handle loop label - for now, just take the type from entry.
    fn handle_loop_label(
        &self,
        reference: NodeIndex,
        type_id: TypeId,
        flow: &FlowNode,
        visited: &mut Vec<FlowNodeId>,
    ) -> TypeId {
        // For loops, we ideally compute a fixed point.
        // For basic narrowing, we can just take the type from the entry antecedent.
        if let Some(&ant) = flow.antecedent.first() {
            self.check_flow(reference, type_id, ant, visited)
        } else {
            type_id
        }
    }

    /// Handle condition node (TRUE_CONDITION or FALSE_CONDITION).
    fn handle_condition(
        &self,
        reference: NodeIndex,
        type_id: TypeId,
        flow: &FlowNode,
        visited: &mut Vec<FlowNodeId>,
    ) -> TypeId {
        // First get the type before this condition
        let pre_type = if let Some(&ant) = flow.antecedent.first() {
            self.check_flow(reference, type_id, ant, visited)
        } else {
            type_id
        };

        // Check if this condition narrows the reference we're interested in
        let is_true_branch = flow.has_any_flags(flow_flags::TRUE_CONDITION);

        // Apply narrowing based on the condition
        self.narrow_type_by_condition(pre_type, flow.node, reference, is_true_branch)
    }

    /// Apply type narrowing based on a condition expression.
    fn narrow_type_by_condition(
        &self,
        type_id: TypeId,
        condition_idx: NodeIndex,
        target: NodeIndex,
        is_true_branch: bool,
    ) -> TypeId {
        let condition_idx = self.skip_parenthesized(condition_idx);
        let Some(cond_node) = self.arena.get(condition_idx) else {
            return type_id;
        };

        let narrowing = NarrowingContext::new(self.interner);

        match cond_node.kind {
            // typeof x === "string"
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                if let Some(bin) = self.arena.get_binary_expr(cond_node) {
                    return self.narrow_by_binary_expr(type_id, bin, target, is_true_branch, &narrowing);
                }
            }

            // Prefix unary: !x
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                if let Some(unary) = self.arena.get_unary_expr(cond_node) {
                    // !x inverts the narrowing
                    if unary.operator == SyntaxKind::ExclamationToken as u16 {
                        return self.narrow_type_by_condition(type_id, unary.operand, target, !is_true_branch);
                    }
                }
            }

            // Truthiness check: if (x)
            _ => {
                if self.is_matching_reference(condition_idx, target) {
                    if is_true_branch {
                        // Remove null/undefined (truthy narrowing)
                        let narrowed = narrowing.narrow_excluding_type(type_id, TypeId::NULL);
                        return narrowing.narrow_excluding_type(narrowed, TypeId::UNDEFINED);
                    }
                    // False branch - keep only falsy types
                    return self.narrow_to_falsy(type_id);
                }
            }
        }

        type_id
    }

    /// Narrow type based on a binary expression (===, !==, typeof checks, etc.)
    fn narrow_by_binary_expr(
        &self,
        type_id: TypeId,
        bin: &crate::parser::thin_node::BinaryExprData,
        target: NodeIndex,
        is_true_branch: bool,
        narrowing: &NarrowingContext,
    ) -> TypeId {
        let operator = bin.operator_token;

        // Handle typeof x === "string"
        if operator == SyntaxKind::EqualsEqualsEqualsToken as u16 ||
           operator == SyntaxKind::ExclamationEqualsEqualsToken as u16 {
            let is_equals = operator == SyntaxKind::EqualsEqualsEqualsToken as u16;
            let effective_truth = if is_equals { is_true_branch } else { !is_true_branch };

            // Check typeof on left side
            if let Some(type_name) = self.get_typeof_check(bin.left, target) {
                if effective_truth {
                    return narrowing.narrow_by_typeof(type_id, &type_name);
                }
                return self.narrow_by_typeof_negation(type_id, &type_name, narrowing);
            }

            // Check typeof on right side
            if let Some(type_name) = self.get_typeof_check(bin.right, target) {
                if effective_truth {
                    return narrowing.narrow_by_typeof(type_id, &type_name);
                }
                return self.narrow_by_typeof_negation(type_id, &type_name, narrowing);
            }

            // Check for null/undefined comparison
            if self.is_matching_reference(bin.left, target) {
                if self.is_null_or_undefined(bin.right) {
                    if effective_truth {
                        // x === null/undefined -> narrow to null/undefined
                        return if self.is_null_keyword(bin.right) {
                            TypeId::NULL
                        } else {
                            TypeId::UNDEFINED
                        };
                    } else {
                        // x !== null/undefined -> exclude null/undefined
                        return if self.is_null_keyword(bin.right) {
                            narrowing.narrow_excluding_type(type_id, TypeId::NULL)
                        } else {
                            narrowing.narrow_excluding_type(type_id, TypeId::UNDEFINED)
                        };
                    }
                }
            }

            if self.is_matching_reference(bin.right, target) {
                if self.is_null_or_undefined(bin.left) {
                    if effective_truth {
                        return if self.is_null_keyword(bin.left) {
                            TypeId::NULL
                        } else {
                            TypeId::UNDEFINED
                        };
                    } else {
                        return if self.is_null_keyword(bin.left) {
                            narrowing.narrow_excluding_type(type_id, TypeId::NULL)
                        } else {
                            narrowing.narrow_excluding_type(type_id, TypeId::UNDEFINED)
                        };
                    }
                }
            }
        }

        type_id
    }

    fn skip_parenthesized(&self, mut idx: NodeIndex) -> NodeIndex {
        loop {
            let Some(node) = self.arena.get(idx) else {
                return idx;
            };
            if node.kind == syntax_kind_ext::PARENTHESIZED_EXPRESSION {
                if let Some(paren) = self.arena.get_parenthesized(node) {
                    idx = paren.expression;
                    continue;
                }
            }
            return idx;
        }
    }

    fn narrow_by_typeof_negation(
        &self,
        type_id: TypeId,
        typeof_result: &str,
        narrowing: &NarrowingContext,
    ) -> TypeId {
        match typeof_result {
            "string" => narrowing.narrow_excluding_type(type_id, TypeId::STRING),
            "number" => narrowing.narrow_excluding_type(type_id, TypeId::NUMBER),
            "boolean" => narrowing.narrow_excluding_type(type_id, TypeId::BOOLEAN),
            "bigint" => narrowing.narrow_excluding_type(type_id, TypeId::BIGINT),
            "symbol" => narrowing.narrow_excluding_type(type_id, TypeId::SYMBOL),
            "undefined" => narrowing.narrow_excluding_type(type_id, TypeId::UNDEFINED),
            "object" => narrowing.narrow_excluding_type(type_id, TypeId::OBJECT),
            "function" => narrowing.narrow_excluding_function(type_id),
            _ => type_id,
        }
    }

    fn narrow_to_falsy(&self, type_id: TypeId) -> TypeId {
        if type_id == TypeId::ANY || type_id == TypeId::UNKNOWN {
            return type_id;
        }

        match self.falsy_component(type_id) {
            Some(falsy) => falsy,
            None => TypeId::NEVER,
        }
    }

    fn falsy_component(&self, type_id: TypeId) -> Option<TypeId> {
        if type_id == TypeId::NULL || type_id == TypeId::UNDEFINED {
            return Some(type_id);
        }
        if type_id == TypeId::BOOLEAN {
            return Some(self.interner.literal_boolean(false));
        }
        if type_id == TypeId::STRING {
            return Some(self.interner.literal_string(""));
        }
        if type_id == TypeId::NUMBER {
            return Some(self.interner.literal_number(0.0));
        }
        if type_id == TypeId::BIGINT {
            return Some(self.interner.literal_bigint("0"));
        }

        let key = self.interner.lookup(type_id)?;
        match key {
            TypeKey::Literal(literal) => {
                if self.literal_is_falsy(&literal) {
                    Some(type_id)
                } else {
                    None
                }
            }
            TypeKey::Union(members) => {
                let members = self.interner.type_list(members);
                let mut falsy_members = Vec::new();
                for &member in members.iter() {
                    if let Some(falsy) = self.falsy_component(member) {
                        falsy_members.push(falsy);
                    }
                }
                match falsy_members.len() {
                    0 => None,
                    1 => Some(falsy_members[0]),
                    _ => Some(self.interner.union(falsy_members)),
                }
            }
            TypeKey::TypeParameter(_) | TypeKey::Infer(_) => Some(type_id),
            _ => None,
        }
    }

    fn literal_is_falsy(&self, literal: &LiteralValue) -> bool {
        match literal {
            LiteralValue::Boolean(false) => true,
            LiteralValue::Number(value) => value.0 == 0.0,
            LiteralValue::String(atom) => self.interner.resolve_atom(*atom).is_empty(),
            LiteralValue::BigInt(atom) => self.interner.resolve_atom(*atom) == "0",
            _ => false,
        }
    }

    /// Check if an expression is `typeof target` and return the compared type name.
    fn get_typeof_check(&self, expr: NodeIndex, target: NodeIndex) -> Option<String> {
        let expr = self.skip_parenthesized(expr);
        let node = self.arena.get(expr)?;

        // Check for typeof expression
        if node.kind == syntax_kind_ext::TYPE_OF_EXPRESSION {
            if let Some(unary) = self.arena.get_unary_expr(node) {
                if self.is_matching_reference(unary.operand, target) {
                    // This is typeof target - now find what it's compared to
                    // The comparison value should be on the other side of the binary expr
                    // We'll need to check from the parent context
                }
            }
        }

        // For now, check if this is the string literal being compared
        if node.kind == SyntaxKind::StringLiteral as u16 {
            if let Some(lit) = self.arena.get_literal(node) {
                return Some(lit.text.clone());
            }
        }

        None
    }

    /// Check if two references point to the same symbol.
    fn is_matching_reference(&self, a: NodeIndex, b: NodeIndex) -> bool {
        let sym_a = self.reference_symbol(a);
        let sym_b = self.reference_symbol(b);
        sym_a.is_some() && sym_a == sym_b
    }

    fn reference_symbol(&self, idx: NodeIndex) -> Option<crate::binder::SymbolId> {
        let idx = self.skip_parenthesized(idx);
        self.binder.get_node_symbol(idx)
            .or_else(|| self.binder.resolve_identifier(self.arena, idx))
    }

    /// Check if an expression is null or undefined keyword.
    fn is_null_or_undefined(&self, idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(idx) else { return false };
        node.kind == SyntaxKind::NullKeyword as u16 ||
        node.kind == SyntaxKind::UndefinedKeyword as u16
    }

    /// Check if an expression is specifically the null keyword.
    fn is_null_keyword(&self, idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(idx) else { return false };
        node.kind == SyntaxKind::NullKeyword as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thin_parser::ThinParserState;

    fn get_if_condition(arena: &ThinNodeArena, root: NodeIndex, stmt_index: usize) -> NodeIndex {
        let root_node = arena.get(root).expect("root node");
        let source_file = arena.get_source_file(root_node).expect("source file");
        let if_idx = *source_file.statements.nodes.get(stmt_index).expect("if statement");
        let if_node = arena.get(if_idx).expect("if node");
        let if_data = arena.get_if_statement(if_node).expect("if data");
        if_data.expression
    }

    #[test]
    fn test_truthiness_false_branch_narrows_to_falsy() {
        let source = r#"
let x: string | number | boolean | null | undefined;
if (x) {}
"#;

        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let arena = parser.get_arena();
        let types = TypeInterner::new();
        let analyzer = FlowAnalyzer::new(arena, &binder, &types);

        let condition_idx = get_if_condition(arena, root, 1);
        let union = types.union(vec![
            TypeId::STRING,
            TypeId::NUMBER,
            TypeId::BOOLEAN,
            TypeId::NULL,
            TypeId::UNDEFINED,
        ]);
        let narrowed = analyzer.narrow_type_by_condition(union, condition_idx, condition_idx, false);

        let falsy_string = types.literal_string("");
        let falsy_number = types.literal_number(0.0);
        let falsy_boolean = types.literal_boolean(false);

        let key = types.lookup(narrowed).expect("narrowed type");
        match key {
            TypeKey::Union(members) => {
                let members = types.type_list(members);
                assert!(members.contains(&falsy_string));
                assert!(members.contains(&falsy_number));
                assert!(members.contains(&falsy_boolean));
                assert!(members.contains(&TypeId::NULL));
                assert!(members.contains(&TypeId::UNDEFINED));
            }
            _ => panic!("Expected falsy union, got {:?}", key),
        }
    }

    #[test]
    fn test_typeof_false_branch_excludes_type() {
        let source = r#"
let x: string | number;
if (typeof x === "string") {}
"#;

        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(parser.get_arena(), root);

        let arena = parser.get_arena();
        let types = TypeInterner::new();
        let analyzer = FlowAnalyzer::new(arena, &binder, &types);

        let condition_idx = get_if_condition(arena, root, 1);
        let condition_node = arena.get(condition_idx).expect("condition node");
        let binary = arena.get_binary_expr(condition_node).expect("binary condition");
        let typeof_node = arena.get(binary.left).expect("typeof node");
        let unary = arena.get_unary_expr(typeof_node).expect("typeof data");
        let target_idx = unary.operand;

        let union = types.union(vec![TypeId::STRING, TypeId::NUMBER]);
        let narrowed = analyzer.narrow_type_by_condition(union, condition_idx, target_idx, false);
        assert_eq!(narrowed, TypeId::NUMBER);
    }
}
