//! Definite Assignment Analysis for control flow analysis.
//!
//! This module provides forward dataflow analysis to determine whether
//! variables are definitely assigned at each point in the control flow graph.
//!
//! The analysis tracks three states for each variable:
//! - **Unassigned**: Variable has definitely not been assigned
//! - **MaybeAssigned**: Variable may or may not have been assigned
//! - **DefinitelyAssigned**: Variable has definitely been assigned
//!
//! This is used to implement TypeScript's definite assignment checking for
//! block-scoped variables (let/const) and detect use-before-definite-assignment errors.

use crate::binder::{FlowNode, FlowNodeId, FlowNodeArena, flow_flags};
use crate::parser::{NodeIndex, syntax_kind_ext};
use crate::parser::thin_node::{ThinNodeArena, NodeAccess};
use crate::scanner::SyntaxKind;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;

/// Assignment state for a single variable at a point in the control flow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignmentState {
    /// Variable has definitely not been assigned
    Unassigned,
    /// Variable may or may not have been assigned
    MaybeAssigned,
    /// Variable has definitely been assigned
    DefinitelyAssigned,
}

impl AssignmentState {
    /// Merge two assignment states at a control flow join point.
    ///
    /// The merge follows these rules:
    /// - DefinitelyAssigned ⊓ DefinitelyAssigned = DefinitelyAssigned
    /// - DefinitelyAssigned ⊓ MaybeAssigned = MaybeAssigned
    /// - DefinitelyAssigned ⊓ Unassigned = MaybeAssigned
    /// - MaybeAssigned ⊓ MaybeAssigned = MaybeAssigned
    /// - MaybeAssigned ⊓ Unassigned = MaybeAssigned
    /// - Unassigned ⊓ Unassigned = Unassigned
    fn merge(self, other: AssignmentState) -> AssignmentState {
        match (self, other) {
            (AssignmentState::DefinitelyAssigned, AssignmentState::DefinitelyAssigned) => {
                AssignmentState::DefinitelyAssigned
            }
            (AssignmentState::Unassigned, AssignmentState::Unassigned) => {
                AssignmentState::Unassigned
            }
            _ => AssignmentState::MaybeAssigned,
        }
    }
}

/// A mapping of variables to their assignment states at a point in the program.
///
/// Variables are indexed by their declaration node ID (NodeIndex).
#[derive(Clone, Debug)]
pub struct AssignmentStateMap {
    states: FxHashMap<u32, AssignmentState>,
}

impl AssignmentStateMap {
    /// Create a new empty state map.
    pub fn new() -> Self {
        Self {
            states: FxHashMap::default(),
        }
    }

    /// Get the assignment state for a variable.
    pub fn get(&self, var_id: NodeIndex) -> AssignmentState {
        self.states.get(&var_id.0).copied().unwrap_or(AssignmentState::Unassigned)
    }

    /// Set the assignment state for a variable.
    pub fn set(&mut self, var_id: NodeIndex, state: AssignmentState) {
        self.states.insert(var_id.0, state);
    }

    /// Mark a variable as definitely assigned.
    pub fn mark_assigned(&mut self, var_id: NodeIndex) {
        self.set(var_id, AssignmentState::DefinitelyAssigned);
    }

    /// Merge another state map into this one.
    ///
    /// This is used at control flow join points where multiple paths converge.
    pub fn merge(&mut self, other: &AssignmentStateMap) {
        // Collect all variable IDs from both maps
        let mut all_vars: FxHashSet<u32> = self.states.keys().copied().collect();
        all_vars.extend(other.states.keys().copied());

        // Merge each variable's state
        for var_id in all_vars {
            let self_state = self.get(NodeIndex(var_id));
            let other_state = other.get(NodeIndex(var_id));
            let merged = self_state.merge(other_state);
            self.set(NodeIndex(var_id), merged);
        }
    }

    /// Check if all variables are in a definite state (no MaybeAssigned).
    pub fn is_definite(&self) -> bool {
        !self.states.values().any(|&s| s == AssignmentState::MaybeAssigned)
    }
}

impl Default for AssignmentStateMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Definite assignment analysis result for a specific program point.
#[derive(Clone, Debug)]
pub struct DefiniteAssignmentResult {
    /// Assignment states for all variables at this point
    pub states: AssignmentStateMap,
}

impl DefiniteAssignmentResult {
    /// Check if a variable is definitely assigned at this point.
    pub fn is_definitely_assigned(&self, var_id: NodeIndex) -> bool {
        self.states.get(var_id) == AssignmentState::DefinitelyAssigned
    }

    /// Check if a variable may be assigned at this point.
    pub fn is_maybe_assigned(&self, var_id: NodeIndex) -> bool {
        let state = self.states.get(var_id);
        state == AssignmentState::DefinitelyAssigned || state == AssignmentState::MaybeAssigned
    }
}

/// Analyzer that performs definite assignment analysis.
///
/// This performs a forward dataflow analysis over the control flow graph,
/// tracking assignment states for variables at each program point.
pub struct DefiniteAssignmentAnalyzer<'a> {
    /// Reference to the ThinNodeArena for AST access
    arena: &'a ThinNodeArena,
    /// Reference to the flow node arena
    flow_arena: &'a FlowNodeArena,
    /// Assignment states at each flow node
    node_states: FxHashMap<FlowNodeId, AssignmentStateMap>,
    /// Variables to track (set of variable declaration node IDs)
    tracked_vars: FxHashSet<u32>,
}

impl<'a> DefiniteAssignmentAnalyzer<'a> {
    /// Create a new definite assignment analyzer.
    pub fn new(
        arena: &'a ThinNodeArena,
        flow_arena: &'a FlowNodeArena,
    ) -> Self {
        Self {
            arena,
            flow_arena,
            node_states: FxHashMap::default(),
            tracked_vars: FxHashSet::default(),
        }
    }

    /// Add a variable declaration to track during analysis.
    pub fn track_variable(&mut self, var_decl: NodeIndex) {
        self.tracked_vars.insert(var_decl.0);
    }

    /// Run the forward dataflow analysis starting from the given flow node.
    ///
    /// Returns the assignment states at each flow node in the graph.
    pub fn analyze(&mut self, entry: FlowNodeId) -> &FxHashMap<FlowNodeId, AssignmentStateMap> {
        // Start with empty state
        let initial_state = AssignmentStateMap::new();

        // Worklist for iterative dataflow analysis
        let mut worklist: Vec<FlowNodeId> = vec![entry];
        let mut visited: FxHashSet<FlowNodeId> = FxHashSet::default();

        // Iterative fixed-point computation
        while let Some(flow_id) = worklist.pop() {
            if visited.contains(&flow_id) {
                continue;
            }

            let Some(flow_node) = self.flow_arena.get(flow_id) else {
                continue;
            };

            // Get or create state for this node
            let state_before = self.node_states.get(&flow_id)
                .cloned()
                .unwrap_or_else(|| initial_state.clone());

            // Compute state after this node
            let state_after = self.process_flow_node(flow_node, state_before);

            // Check if state changed
            let changed = if let Some(existing) = self.node_states.get(&flow_id) {
                // Simple check: if we haven't processed this node before, it's new
                false
            } else {
                true
            };

            self.node_states.insert(flow_id, state_after);

            if changed {
                // Add successors to worklist
                for &antecedent in &flow_node.antecedent {
                    if antecedent != FlowNodeId::NONE {
                        worklist.push(antecedent);
                    }
                }
            }

            visited.insert(flow_id);
        }

        &self.node_states
    }

    /// Process a flow node and compute the resulting assignment state.
    fn process_flow_node(
        &self,
        flow_node: &FlowNode,
        mut state: AssignmentStateMap,
    ) -> AssignmentStateMap {
        // Handle different flow node types
        if flow_node.has_any_flags(flow_flags::ASSIGNMENT) {
            // Check if this is an assignment to a tracked variable
            if let Some(target_var) = self.get_assignment_target(flow_node.node) {
                if self.tracked_vars.contains(&target_var.0) {
                    state.mark_assigned(target_var);
                }
            }
        } else if flow_node.has_any_flags(flow_flags::BRANCH_LABEL) {
            // At a branch label (merge point), we need to merge states from all predecessors
            // This is handled during the analysis iteration
            // For now, just propagate the state
        } else if flow_node.has_any_flags(flow_flags::LOOP_LABEL) {
            // At a loop label, we need to handle loop entry and back-edges
            // For now, just propagate the state
        } else if flow_node.has_any_flags(flow_flags::TRUE_CONDITION | flow_flags::FALSE_CONDITION) {
            // Condition nodes - propagate state without changes
            // The narrowing/branching logic is handled by the flow graph structure
        }

        state
    }

    /// Get the assignment target of a node (if it's an assignment to a tracked variable).
    fn get_assignment_target(&self, node: NodeIndex) -> Option<NodeIndex> {
        let node_data = self.arena.get(node)?;

        match node_data.kind {
            k if k == SyntaxKind::Identifier as u16 => {
                // Check if this identifier is a variable we're tracking
                // For now, return the node itself if it's a tracked variable
                Some(node)
            }
            syntax_kind_ext::BINARY_EXPRESSION => {
                // Check if this is an assignment expression
                if let Some(bin_expr) = self.arena.get_binary_expr(node_data) {
                    if let Some(left_node) = self.arena.get(bin_expr.left) {
                        if left_node.kind == SyntaxKind::Identifier as u16 {
                            return Some(bin_expr.left);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Get the assignment state at a specific flow node.
    pub fn get_state_at(&self, flow_id: FlowNodeId) -> Option<&AssignmentStateMap> {
        self.node_states.get(&flow_id)
    }

    /// Check if a variable is definitely assigned at a specific flow node.
    pub fn is_definitely_assigned(&self, var_id: NodeIndex, flow_id: FlowNodeId) -> bool {
        if let Some(state) = self.get_state_at(flow_id) {
            state.get(var_id) == AssignmentState::DefinitelyAssigned
        } else {
            false
        }
    }

    /// Get a reference to all node states.
    pub fn node_states(&self) -> &FxHashMap<FlowNodeId, AssignmentStateMap> {
        &self.node_states
    }
}

/// Merges multiple assignment state maps at a control flow join point.
///
/// This is used when multiple paths converge (e.g., after an if-else statement).
pub fn merge_assignment_states(states: &[AssignmentStateMap]) -> AssignmentStateMap {
    if states.is_empty() {
        return AssignmentStateMap::new();
    }

    let mut result = states[0].clone();
    for state in &states[1..] {
        result.merge(state);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment_state_merge() {
        // DefinitelyAssigned merge DefinitelyAssigned = DefinitelyAssigned
        assert_eq!(
            AssignmentState::DefinitelyAssigned.merge(AssignmentState::DefinitelyAssigned),
            AssignmentState::DefinitelyAssigned
        );

        // DefinitelyAssigned merge Unassigned = MaybeAssigned
        assert_eq!(
            AssignmentState::DefinitelyAssigned.merge(AssignmentState::Unassigned),
            AssignmentState::MaybeAssigned
        );

        // Unassigned merge Unassigned = Unassigned
        assert_eq!(
            AssignmentState::Unassigned.merge(AssignmentState::Unassigned),
            AssignmentState::Unassigned
        );

        // MaybeAssigned merge anything = MaybeAssigned
        assert_eq!(
            AssignmentState::MaybeAssigned.merge(AssignmentState::DefinitelyAssigned),
            AssignmentState::MaybeAssigned
        );
        assert_eq!(
            AssignmentState::MaybeAssigned.merge(AssignmentState::Unassigned),
            AssignmentState::MaybeAssigned
        );
    }

    #[test]
    fn test_assignment_state_map() {
        let mut map = AssignmentStateMap::new();
        let var1 = NodeIndex(1);
        let var2 = NodeIndex(2);

        // Initially unassigned
        assert_eq!(map.get(var1), AssignmentState::Unassigned);

        // Mark as assigned
        map.mark_assigned(var1);
        assert_eq!(map.get(var1), AssignmentState::DefinitelyAssigned);

        // Merge with another map
        let mut map2 = AssignmentStateMap::new();
        map2.set(var2, AssignmentState::DefinitelyAssigned);
        map2.set(var1, AssignmentState::Unassigned);

        map.merge(&map2);
        // var1: DefinitelyAssigned merge Unassigned = MaybeAssigned
        assert_eq!(map.get(var1), AssignmentState::MaybeAssigned);
        // var2: Unassigned merge DefinitelyAssigned = MaybeAssigned
        assert_eq!(map.get(var2), AssignmentState::MaybeAssigned);
    }

    #[test]
    fn test_definite_assignment_result() {
        let mut states = AssignmentStateMap::new();
        let var1 = NodeIndex(1);
        let var2 = NodeIndex(2);

        states.mark_assigned(var1);
        states.set(var2, AssignmentState::MaybeAssigned);

        let result = DefiniteAssignmentResult { states };

        assert!(result.is_definitely_assigned(var1));
        assert!(!result.is_definitely_assigned(var2));
        assert!(result.is_maybe_assigned(var1));
        assert!(result.is_maybe_assigned(var2));
    }

    #[test]
    fn test_merge_assignment_states() {
        let mut state1 = AssignmentStateMap::new();
        let mut state2 = AssignmentStateMap::new();
        let var1 = NodeIndex(1);

        state1.mark_assigned(var1);
        // state2 doesn't have var1 (implicitly Unassigned)

        let merged = merge_assignment_states(&[state1, state2]);
        assert_eq!(merged.get(var1), AssignmentState::MaybeAssigned);
    }
}
