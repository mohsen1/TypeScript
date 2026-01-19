//! Control Flow Graph Implementation
//!
//! This module builds the control flow graph for type narrowing and
//! reachability analysis.

use std::collections::HashSet;

/// Unique identifier for flow nodes
pub type FlowNodeId = u32;

/// Special flow node IDs
pub const UNREACHABLE_FLOW: FlowNodeId = 0;
pub const START_FLOW: FlowNodeId = 1;

/// Flags for flow nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FlowFlags(u32);

impl FlowFlags {
    pub const UNREACHABLE: FlowFlags = FlowFlags(1 << 0);
    pub const START: FlowFlags = FlowFlags(1 << 1);
    pub const BRANCH_LABEL: FlowFlags = FlowFlags(1 << 2);
    pub const LOOP_LABEL: FlowFlags = FlowFlags(1 << 3);
    pub const ASSIGNMENT: FlowFlags = FlowFlags(1 << 4);
    pub const TRUE_CONDITION: FlowFlags = FlowFlags(1 << 5);
    pub const FALSE_CONDITION: FlowFlags = FlowFlags(1 << 6);
    pub const SWITCH_CLAUSE: FlowFlags = FlowFlags(1 << 7);
    pub const ARRAY_MUTATION: FlowFlags = FlowFlags(1 << 8);
    pub const CALL: FlowFlags = FlowFlags(1 << 9);
    pub const NARROWED_TYPE: FlowFlags = FlowFlags(1 << 10);
    pub const SHARED: FlowFlags = FlowFlags(1 << 11);

    pub const LABEL: FlowFlags = FlowFlags(Self::BRANCH_LABEL.0 | Self::LOOP_LABEL.0);
    pub const CONDITION: FlowFlags = FlowFlags(Self::TRUE_CONDITION.0 | Self::FALSE_CONDITION.0);

    pub fn contains(&self, other: FlowFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn intersects(&self, other: FlowFlags) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for FlowFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        FlowFlags(self.0 | rhs.0)
    }
}

/// A node in the control flow graph
#[derive(Debug, Clone)]
pub enum FlowNode {
    /// Unreachable code
    Unreachable,
    /// Start of function/script
    Start,
    /// A label that can be the target of jumps
    Label {
        flags: FlowFlags,
        antecedents: Vec<FlowNodeId>,
    },
    /// An assignment to a variable
    Assignment {
        antecedent: FlowNodeId,
        node_id: u32, // Reference to AST node
    },
    /// A condition (true or false branch)
    Condition {
        flags: FlowFlags,
        antecedent: FlowNodeId,
        node_id: u32, // Reference to AST node
    },
    /// A switch clause
    SwitchClause {
        antecedent: FlowNodeId,
        clause_start: u32,
        clause_end: u32,
    },
    /// An array mutation (push, pop, etc.)
    ArrayMutation {
        antecedent: FlowNodeId,
        node_id: u32,
    },
    /// A function call
    Call {
        antecedent: FlowNodeId,
        node_id: u32,
    },
    /// A type narrowing through type guards
    NarrowedType {
        antecedent: FlowNodeId,
        node_id: u32,
        narrowed_type: NarrowedTypeInfo,
    },
}

/// Information about a narrowed type
#[derive(Debug, Clone)]
pub struct NarrowedTypeInfo {
    pub kind: NarrowingKind,
    pub target_symbol: u32,
}

/// Types of narrowing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NarrowingKind {
    /// typeof x === "string"
    TypeOf,
    /// x instanceof Foo
    InstanceOf,
    /// x === value
    Equality,
    /// x !== value
    Inequality,
    /// "prop" in x
    InOperator,
    /// User-defined type guard
    TypeGuard,
    /// Truthiness check
    Truthiness,
    /// Non-null assertion
    NonNull,
}

/// Builder for the control flow graph
#[derive(Debug)]
pub struct FlowGraphBuilder {
    nodes: Vec<FlowNode>,
    current_flow: FlowNodeId,
    /// Stack of break targets
    break_targets: Vec<FlowNodeId>,
    /// Stack of continue targets
    continue_targets: Vec<FlowNodeId>,
    /// Stack of return targets
    return_target: Option<FlowNodeId>,
    /// Stack of exception targets
    exception_target: Option<FlowNodeId>,
    /// Label stacks for labeled statements
    active_labels: Vec<ActiveLabel>,
}

#[derive(Debug)]
struct ActiveLabel {
    name: Option<String>,
    break_target: FlowNodeId,
    continue_target: Option<FlowNodeId>,
}

impl FlowGraphBuilder {
    pub fn new() -> Self {
        let mut builder = FlowGraphBuilder {
            nodes: Vec::new(),
            current_flow: START_FLOW,
            break_targets: Vec::new(),
            continue_targets: Vec::new(),
            return_target: None,
            exception_target: None,
            active_labels: Vec::new(),
        };

        // Create the initial nodes
        builder.nodes.push(FlowNode::Unreachable);
        builder.nodes.push(FlowNode::Start);

        builder
    }

    /// Get the current flow node
    pub fn current_flow(&self) -> FlowNodeId {
        self.current_flow
    }

    /// Set the current flow node
    pub fn set_current_flow(&mut self, flow: FlowNodeId) {
        self.current_flow = flow;
    }

    /// Check if current flow is reachable
    pub fn is_reachable(&self) -> bool {
        self.current_flow != UNREACHABLE_FLOW
    }

    /// Create a branch label
    pub fn create_branch_label(&mut self) -> FlowNodeId {
        let id = self.nodes.len() as FlowNodeId;
        self.nodes.push(FlowNode::Label {
            flags: FlowFlags::BRANCH_LABEL,
            antecedents: Vec::new(),
        });
        id
    }

    /// Create a loop label
    pub fn create_loop_label(&mut self) -> FlowNodeId {
        let id = self.nodes.len() as FlowNodeId;
        self.nodes.push(FlowNode::Label {
            flags: FlowFlags::LOOP_LABEL,
            antecedents: Vec::new(),
        });
        id
    }

    /// Add an antecedent to a label
    pub fn add_antecedent(&mut self, label: FlowNodeId, antecedent: FlowNodeId) {
        if antecedent == UNREACHABLE_FLOW {
            return;
        }

        if let Some(FlowNode::Label { antecedents, .. }) = self.nodes.get_mut(label as usize) {
            if !antecedents.contains(&antecedent) {
                antecedents.push(antecedent);
            }
        }
    }

    /// Finish a label and return the resulting flow
    pub fn finish_flow_label(&mut self, label: FlowNodeId) -> FlowNodeId {
        if let Some(FlowNode::Label { antecedents, .. }) = self.nodes.get(label as usize) {
            match antecedents.len() {
                0 => UNREACHABLE_FLOW,
                1 => antecedents[0],
                _ => label,
            }
        } else {
            label
        }
    }

    /// Create an assignment flow node
    pub fn create_assignment(&mut self, node_id: u32) -> FlowNodeId {
        if !self.is_reachable() {
            return UNREACHABLE_FLOW;
        }

        let id = self.nodes.len() as FlowNodeId;
        self.nodes.push(FlowNode::Assignment {
            antecedent: self.current_flow,
            node_id,
        });
        self.current_flow = id;
        id
    }

    /// Create a condition flow node
    pub fn create_condition(&mut self, node_id: u32, is_true_branch: bool) -> FlowNodeId {
        if !self.is_reachable() {
            return UNREACHABLE_FLOW;
        }

        let flags = if is_true_branch {
            FlowFlags::TRUE_CONDITION
        } else {
            FlowFlags::FALSE_CONDITION
        };

        let id = self.nodes.len() as FlowNodeId;
        self.nodes.push(FlowNode::Condition {
            flags,
            antecedent: self.current_flow,
            node_id,
        });
        id
    }

    /// Create a call flow node
    pub fn create_call(&mut self, node_id: u32) -> FlowNodeId {
        if !self.is_reachable() {
            return UNREACHABLE_FLOW;
        }

        let id = self.nodes.len() as FlowNodeId;
        self.nodes.push(FlowNode::Call {
            antecedent: self.current_flow,
            node_id,
        });
        self.current_flow = id;
        id
    }

    /// Create a narrowed type flow node
    pub fn create_narrowing(&mut self, node_id: u32, kind: NarrowingKind, target_symbol: u32) -> FlowNodeId {
        if !self.is_reachable() {
            return UNREACHABLE_FLOW;
        }

        let id = self.nodes.len() as FlowNodeId;
        self.nodes.push(FlowNode::NarrowedType {
            antecedent: self.current_flow,
            node_id,
            narrowed_type: NarrowedTypeInfo { kind, target_symbol },
        });
        self.current_flow = id;
        id
    }

    /// Push a break target
    pub fn push_break_target(&mut self, target: FlowNodeId) {
        self.break_targets.push(target);
    }

    /// Pop a break target
    pub fn pop_break_target(&mut self) -> Option<FlowNodeId> {
        self.break_targets.pop()
    }

    /// Get current break target
    pub fn current_break_target(&self) -> Option<FlowNodeId> {
        self.break_targets.last().copied()
    }

    /// Push a continue target
    pub fn push_continue_target(&mut self, target: FlowNodeId) {
        self.continue_targets.push(target);
    }

    /// Pop a continue target
    pub fn pop_continue_target(&mut self) -> Option<FlowNodeId> {
        self.continue_targets.pop()
    }

    /// Get current continue target
    pub fn current_continue_target(&self) -> Option<FlowNodeId> {
        self.continue_targets.last().copied()
    }

    /// Set return target
    pub fn set_return_target(&mut self, target: FlowNodeId) {
        self.return_target = Some(target);
    }

    /// Get return target
    pub fn return_target(&self) -> Option<FlowNodeId> {
        self.return_target
    }

    /// Handle a break statement
    pub fn bind_break(&mut self, label: Option<&str>) {
        let target = self.find_break_target(label);
        if let Some(target) = target {
            self.add_antecedent(target, self.current_flow);
        }
        self.current_flow = UNREACHABLE_FLOW;
    }

    /// Handle a continue statement
    pub fn bind_continue(&mut self, label: Option<&str>) {
        let target = self.find_continue_target(label);
        if let Some(target) = target {
            self.add_antecedent(target, self.current_flow);
        }
        self.current_flow = UNREACHABLE_FLOW;
    }

    /// Handle a return statement
    pub fn bind_return(&mut self) {
        if let Some(target) = self.return_target {
            self.add_antecedent(target, self.current_flow);
        }
        self.current_flow = UNREACHABLE_FLOW;
    }

    /// Handle a throw statement
    pub fn bind_throw(&mut self) {
        if let Some(target) = self.exception_target {
            self.add_antecedent(target, self.current_flow);
        }
        self.current_flow = UNREACHABLE_FLOW;
    }

    /// Push an active label
    pub fn push_label(&mut self, name: Option<String>, break_target: FlowNodeId, continue_target: Option<FlowNodeId>) {
        self.active_labels.push(ActiveLabel {
            name,
            break_target,
            continue_target,
        });
    }

    /// Pop an active label
    pub fn pop_label(&mut self) {
        self.active_labels.pop();
    }

    fn find_break_target(&self, label: Option<&str>) -> Option<FlowNodeId> {
        match label {
            Some(name) => {
                for active in self.active_labels.iter().rev() {
                    if active.name.as_deref() == Some(name) {
                        return Some(active.break_target);
                    }
                }
                None
            }
            None => self.break_targets.last().copied(),
        }
    }

    fn find_continue_target(&self, label: Option<&str>) -> Option<FlowNodeId> {
        match label {
            Some(name) => {
                for active in self.active_labels.iter().rev() {
                    if active.name.as_deref() == Some(name) {
                        return active.continue_target;
                    }
                }
                None
            }
            None => self.continue_targets.last().copied(),
        }
    }

    /// Get a flow node by ID
    pub fn get_node(&self, id: FlowNodeId) -> Option<&FlowNode> {
        self.nodes.get(id as usize)
    }

    /// Get the total number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Find all nodes that reach a given node
    pub fn find_reaching_nodes(&self, target: FlowNodeId) -> HashSet<FlowNodeId> {
        let mut reaching = HashSet::new();
        let mut worklist = vec![target];

        while let Some(node_id) = worklist.pop() {
            if reaching.contains(&node_id) || node_id == UNREACHABLE_FLOW {
                continue;
            }

            reaching.insert(node_id);

            if let Some(node) = self.nodes.get(node_id as usize) {
                match node {
                    FlowNode::Label { antecedents, .. } => {
                        worklist.extend(antecedents.iter().copied());
                    }
                    FlowNode::Assignment { antecedent, .. }
                    | FlowNode::Condition { antecedent, .. }
                    | FlowNode::SwitchClause { antecedent, .. }
                    | FlowNode::ArrayMutation { antecedent, .. }
                    | FlowNode::Call { antecedent, .. }
                    | FlowNode::NarrowedType { antecedent, .. } => {
                        worklist.push(*antecedent);
                    }
                    FlowNode::Unreachable | FlowNode::Start => {}
                }
            }
        }

        reaching
    }
}

impl Default for FlowGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_graph_creation() {
        let builder = FlowGraphBuilder::new();
        assert_eq!(builder.current_flow(), START_FLOW);
        assert!(builder.is_reachable());
    }

    #[test]
    fn test_assignment_flow() {
        let mut builder = FlowGraphBuilder::new();
        let assignment = builder.create_assignment(100);
        assert_eq!(builder.current_flow(), assignment);

        match builder.get_node(assignment) {
            Some(FlowNode::Assignment { antecedent, node_id }) => {
                assert_eq!(*antecedent, START_FLOW);
                assert_eq!(*node_id, 100);
            }
            _ => panic!("Expected assignment node"),
        }
    }

    #[test]
    fn test_branch_label() {
        let mut builder = FlowGraphBuilder::new();

        let label = builder.create_branch_label();
        builder.add_antecedent(label, builder.current_flow());

        let assignment = builder.create_assignment(100);
        builder.add_antecedent(label, assignment);

        match builder.get_node(label) {
            Some(FlowNode::Label { antecedents, flags }) => {
                assert!(flags.contains(FlowFlags::BRANCH_LABEL));
                assert_eq!(antecedents.len(), 2);
            }
            _ => panic!("Expected label node"),
        }
    }

    #[test]
    fn test_break_continue() {
        let mut builder = FlowGraphBuilder::new();

        let break_label = builder.create_branch_label();
        let continue_label = builder.create_loop_label();

        builder.push_break_target(break_label);
        builder.push_continue_target(continue_label);

        builder.bind_continue(None);
        assert!(!builder.is_reachable());

        // Reset flow for testing break
        builder.set_current_flow(START_FLOW);
        builder.bind_break(None);
        assert!(!builder.is_reachable());

        builder.pop_break_target();
        builder.pop_continue_target();
    }

    #[test]
    fn test_condition_flow() {
        let mut builder = FlowGraphBuilder::new();

        let true_flow = builder.create_condition(100, true);
        let false_flow = builder.create_condition(100, false);

        match builder.get_node(true_flow) {
            Some(FlowNode::Condition { flags, .. }) => {
                assert!(flags.contains(FlowFlags::TRUE_CONDITION));
            }
            _ => panic!("Expected condition node"),
        }

        match builder.get_node(false_flow) {
            Some(FlowNode::Condition { flags, .. }) => {
                assert!(flags.contains(FlowFlags::FALSE_CONDITION));
            }
            _ => panic!("Expected condition node"),
        }
    }

    #[test]
    fn test_return_flow() {
        let mut builder = FlowGraphBuilder::new();

        let return_label = builder.create_branch_label();
        builder.set_return_target(return_label);

        builder.bind_return();
        assert!(!builder.is_reachable());

        match builder.get_node(return_label) {
            Some(FlowNode::Label { antecedents, .. }) => {
                assert_eq!(antecedents.len(), 1);
                assert_eq!(antecedents[0], START_FLOW);
            }
            _ => panic!("Expected label node"),
        }
    }

    #[test]
    fn test_labeled_break() {
        let mut builder = FlowGraphBuilder::new();

        let outer_label = builder.create_branch_label();
        let inner_label = builder.create_branch_label();

        builder.push_label(Some("outer".to_string()), outer_label, None);
        builder.push_label(None, inner_label, None);

        // Break to labeled statement
        builder.bind_break(Some("outer"));

        match builder.get_node(outer_label) {
            Some(FlowNode::Label { antecedents, .. }) => {
                assert_eq!(antecedents.len(), 1);
            }
            _ => panic!("Expected label node"),
        }

        // Inner label should be empty
        match builder.get_node(inner_label) {
            Some(FlowNode::Label { antecedents, .. }) => {
                assert_eq!(antecedents.len(), 0);
            }
            _ => panic!("Expected label node"),
        }
    }

    #[test]
    fn test_narrowing() {
        let mut builder = FlowGraphBuilder::new();

        let narrow = builder.create_narrowing(100, NarrowingKind::TypeOf, 50);

        match builder.get_node(narrow) {
            Some(FlowNode::NarrowedType { narrowed_type, .. }) => {
                assert_eq!(narrowed_type.kind, NarrowingKind::TypeOf);
                assert_eq!(narrowed_type.target_symbol, 50);
            }
            _ => panic!("Expected narrowed type node"),
        }
    }
}
