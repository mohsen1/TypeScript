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
use crate::solver::{TypeId, TypeInterner, NarrowingContext};
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
                    } else {
                        // False branch - could be null/undefined/false/0/""
                        // For now, don't narrow the false branch of truthiness checks
                        return type_id;
                    }
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
                // TODO: handle false branch (exclude type)
            }

            // Check typeof on right side
            if let Some(type_name) = self.get_typeof_check(bin.right, target) {
                if effective_truth {
                    return narrowing.narrow_by_typeof(type_id, &type_name);
                }
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

    /// Check if an expression is `typeof target` and return the compared type name.
    fn get_typeof_check(&self, expr: NodeIndex, target: NodeIndex) -> Option<String> {
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
        let sym_a = self.binder.get_node_symbol(a);
        let sym_b = self.binder.get_node_symbol(b);
        sym_a.is_some() && sym_a == sym_b
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
    // Tests will be added when integrated with the checker
}
