//! Container Tracking for Binding
//!
//! This module handles tracking of container nodes during binding.
//! Containers are nodes that can have locals (symbol tables).

use super::symbols::SymbolId;

/// Node flags that affect binding behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeFlags(u32);

impl NodeFlags {
    pub const NONE: NodeFlags = NodeFlags(0);
    pub const HAS_IMPLICIT_RETURN: NodeFlags = NodeFlags(1 << 0);
    pub const HAS_EXPLICIT_RETURN: NodeFlags = NodeFlags(1 << 1);
    pub const GLOBAL_AUGMENTATION: NodeFlags = NodeFlags(1 << 2);
    pub const HAS_ASYNC_FUNCTIONS: NodeFlags = NodeFlags(1 << 3);
    pub const DISALLOW_IN_CONTEXT: NodeFlags = NodeFlags(1 << 4);
    pub const YIELD_CONTEXT: NodeFlags = NodeFlags(1 << 5);
    pub const DECORATOR_CONTEXT: NodeFlags = NodeFlags(1 << 6);
    pub const AWAIT_CONTEXT: NodeFlags = NodeFlags(1 << 7);
    pub const THIS_NODE_HAS_ERROR: NodeFlags = NodeFlags(1 << 8);
    pub const JAVASCRIPT_FILE: NodeFlags = NodeFlags(1 << 9);
    pub const THIS_NODE_OR_ANY_SUB_HAS_ERROR: NodeFlags = NodeFlags(1 << 10);
    pub const HAS_AGGREGATED_CHILDREN: NodeFlags = NodeFlags(1 << 11);
    pub const AMBIENT_MODULE: NodeFlags = NodeFlags(1 << 12);
    pub const IN_WITH_STATEMENT: NodeFlags = NodeFlags(1 << 13);
    pub const JSON_FILE: NodeFlags = NodeFlags(1 << 14);
    pub const TYPE_CACHED: NodeFlags = NodeFlags(1 << 15);

    pub const BLOCK_SCOPED: NodeFlags = NodeFlags(1 << 16);
    pub const LET: NodeFlags = NodeFlags(1 << 17);
    pub const CONST: NodeFlags = NodeFlags(1 << 18);
    pub const USING: NodeFlags = NodeFlags(1 << 19);
    pub const AWAIT_USING: NodeFlags = NodeFlags(1 << 20);
    pub const NAMESPACE_EXPORT: NodeFlags = NodeFlags(1 << 21);
    pub const EXPORT_CONTEXT: NodeFlags = NodeFlags(1 << 22);

    pub const HAS_RETURN: NodeFlags = NodeFlags(Self::HAS_IMPLICIT_RETURN.0 | Self::HAS_EXPLICIT_RETURN.0);

    pub fn contains(&self, other: NodeFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn intersects(&self, other: NodeFlags) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for NodeFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        NodeFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for NodeFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Reference to an AST node
pub type NodeId = u32;

/// Container flags for determining what kind of container a node is
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContainerFlags(u32);

impl ContainerFlags {
    pub const NONE: ContainerFlags = ContainerFlags(0);
    /// The current node is a container
    pub const IS_CONTAINER: ContainerFlags = ContainerFlags(1 << 0);
    /// The current node is a block-scoped container
    pub const IS_BLOCK_SCOPED_CONTAINER: ContainerFlags = ContainerFlags(1 << 1);
    /// The current node is a control flow container
    pub const IS_CONTROL_FLOW_CONTAINER: ContainerFlags = ContainerFlags(1 << 2);
    /// The current node is a function-like
    pub const IS_FUNCTION_LIKE: ContainerFlags = ContainerFlags(1 << 3);
    /// The current node is a function expression
    pub const IS_FUNCTION_EXPRESSION: ContainerFlags = ContainerFlags(1 << 4);
    /// This function has a local scope
    pub const HAS_LOCALS: ContainerFlags = ContainerFlags(1 << 5);
    /// Contains 'this' references
    pub const IS_INTERFACE: ContainerFlags = ContainerFlags(1 << 6);
    /// Is object literal
    pub const IS_OBJECT_LITERAL_OR_CLASS_EXPRESSION_METHOD: ContainerFlags = ContainerFlags(1 << 7);

    pub fn contains(&self, other: ContainerFlags) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn intersects(&self, other: ContainerFlags) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for ContainerFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        ContainerFlags(self.0 | rhs.0)
    }
}

/// Information about a container node
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    /// Node ID of the container
    pub node_id: NodeId,
    /// Container flags
    pub flags: ContainerFlags,
    /// Symbol associated with this container (if any)
    pub symbol: Option<SymbolId>,
    /// Parent container
    pub parent: Option<NodeId>,
    /// Whether this container is strict mode
    pub strict_mode: bool,
}

impl ContainerInfo {
    pub fn new(node_id: NodeId, flags: ContainerFlags) -> Self {
        ContainerInfo {
            node_id,
            flags,
            symbol: None,
            parent: None,
            strict_mode: false,
        }
    }

    pub fn is_container(&self) -> bool {
        self.flags.contains(ContainerFlags::IS_CONTAINER)
    }

    pub fn is_block_scoped_container(&self) -> bool {
        self.flags.contains(ContainerFlags::IS_BLOCK_SCOPED_CONTAINER)
    }

    pub fn is_control_flow_container(&self) -> bool {
        self.flags.contains(ContainerFlags::IS_CONTROL_FLOW_CONTAINER)
    }

    pub fn is_function_like(&self) -> bool {
        self.flags.contains(ContainerFlags::IS_FUNCTION_LIKE)
    }

    pub fn has_locals(&self) -> bool {
        self.flags.contains(ContainerFlags::HAS_LOCALS)
    }
}

/// Stack for tracking containers during binding
#[derive(Debug)]
pub struct ContainerStack {
    /// Stack of container infos
    containers: Vec<ContainerInfo>,
    /// Stack of block scope containers
    block_scope_containers: Vec<NodeId>,
    /// Current container node
    current_container: Option<NodeId>,
    /// Current block scope container
    current_block_scope_container: Option<NodeId>,
}

impl ContainerStack {
    pub fn new() -> Self {
        ContainerStack {
            containers: Vec::new(),
            block_scope_containers: Vec::new(),
            current_container: None,
            current_block_scope_container: None,
        }
    }

    /// Push a new container
    pub fn push_container(&mut self, info: ContainerInfo) {
        let node_id = info.node_id;
        let is_block_scoped = info.is_block_scoped_container();
        let is_container = info.is_container();

        self.containers.push(info);

        if is_container {
            self.current_container = Some(node_id);
        }

        if is_block_scoped {
            self.block_scope_containers.push(node_id);
            self.current_block_scope_container = Some(node_id);
        }
    }

    /// Pop the current container
    pub fn pop_container(&mut self) -> Option<ContainerInfo> {
        let info = self.containers.pop()?;

        if info.is_container() {
            self.current_container = self.containers.iter().rev()
                .find(|c| c.is_container())
                .map(|c| c.node_id);
        }

        if info.is_block_scoped_container() {
            self.block_scope_containers.pop();
            self.current_block_scope_container = self.block_scope_containers.last().copied();
        }

        Some(info)
    }

    /// Get the current container
    pub fn current_container(&self) -> Option<&ContainerInfo> {
        self.containers.iter().rev().find(|c| c.is_container())
    }

    /// Get the current container mutably
    pub fn current_container_mut(&mut self) -> Option<&mut ContainerInfo> {
        self.containers.iter_mut().rev().find(|c| c.is_container())
    }

    /// Get the current block scope container
    pub fn current_block_scope_container(&self) -> Option<&ContainerInfo> {
        self.containers.iter().rev().find(|c| c.is_block_scoped_container())
    }

    /// Get container info by node ID
    pub fn get_container(&self, node_id: NodeId) -> Option<&ContainerInfo> {
        self.containers.iter().find(|c| c.node_id == node_id)
    }

    /// Check if we're inside a function-like container
    pub fn in_function(&self) -> bool {
        self.containers.iter().any(|c| c.is_function_like())
    }

    /// Check if we're in strict mode
    pub fn in_strict_mode(&self) -> bool {
        self.containers.iter().rev()
            .find(|c| c.is_container())
            .map(|c| c.strict_mode)
            .unwrap_or(false)
    }

    /// Set strict mode for the current container
    pub fn set_strict_mode(&mut self, strict: bool) {
        if let Some(container) = self.current_container_mut() {
            container.strict_mode = strict;
        }
    }

    /// Get the depth of the container stack
    pub fn depth(&self) -> usize {
        self.containers.len()
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.containers.is_empty()
    }
}

impl Default for ContainerStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Hoisting information for declarations
#[derive(Debug, Clone)]
pub struct HoistingInfo {
    /// Variables that need to be hoisted (var declarations)
    pub hoisted_vars: Vec<(String, SymbolId)>,
    /// Functions that need to be hoisted
    pub hoisted_functions: Vec<(String, SymbolId)>,
}

impl HoistingInfo {
    pub fn new() -> Self {
        HoistingInfo {
            hoisted_vars: Vec::new(),
            hoisted_functions: Vec::new(),
        }
    }

    pub fn add_hoisted_var(&mut self, name: String, symbol: SymbolId) {
        self.hoisted_vars.push((name, symbol));
    }

    pub fn add_hoisted_function(&mut self, name: String, symbol: SymbolId) {
        self.hoisted_functions.push((name, symbol));
    }

    pub fn is_empty(&self) -> bool {
        self.hoisted_vars.is_empty() && self.hoisted_functions.is_empty()
    }
}

impl Default for HoistingInfo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_flags() {
        let flags = NodeFlags::LET | NodeFlags::BLOCK_SCOPED;
        assert!(flags.contains(NodeFlags::LET));
        assert!(flags.contains(NodeFlags::BLOCK_SCOPED));
        assert!(!flags.contains(NodeFlags::CONST));
    }

    #[test]
    fn test_container_flags() {
        let flags = ContainerFlags::IS_CONTAINER | ContainerFlags::IS_FUNCTION_LIKE;
        assert!(flags.contains(ContainerFlags::IS_CONTAINER));
        assert!(flags.contains(ContainerFlags::IS_FUNCTION_LIKE));
        assert!(!flags.contains(ContainerFlags::IS_INTERFACE));
    }

    #[test]
    fn test_container_stack() {
        let mut stack = ContainerStack::new();

        // Push a module container
        let module = ContainerInfo::new(
            1,
            ContainerFlags::IS_CONTAINER | ContainerFlags::IS_BLOCK_SCOPED_CONTAINER
        );
        stack.push_container(module);

        assert_eq!(stack.current_container().unwrap().node_id, 1);
        assert_eq!(stack.current_block_scope_container().unwrap().node_id, 1);

        // Push a function container
        let mut func = ContainerInfo::new(
            2,
            ContainerFlags::IS_CONTAINER | ContainerFlags::IS_FUNCTION_LIKE |
            ContainerFlags::IS_CONTROL_FLOW_CONTAINER
        );
        func.strict_mode = true;
        stack.push_container(func);

        assert_eq!(stack.current_container().unwrap().node_id, 2);
        assert!(stack.in_strict_mode());
        assert!(stack.in_function());

        // Pop function
        stack.pop_container();
        assert_eq!(stack.current_container().unwrap().node_id, 1);
        assert!(!stack.in_function());
    }

    #[test]
    fn test_hoisting_info() {
        let mut info = HoistingInfo::new();
        assert!(info.is_empty());

        info.add_hoisted_var("x".to_string(), 1);
        info.add_hoisted_function("foo".to_string(), 2);

        assert!(!info.is_empty());
        assert_eq!(info.hoisted_vars.len(), 1);
        assert_eq!(info.hoisted_functions.len(), 1);
    }

    #[test]
    fn test_block_scope_container_tracking() {
        let mut stack = ContainerStack::new();

        // Push function (container but not block-scoped)
        stack.push_container(ContainerInfo::new(
            1,
            ContainerFlags::IS_CONTAINER | ContainerFlags::IS_FUNCTION_LIKE
        ));

        // Push block (block-scoped container)
        stack.push_container(ContainerInfo::new(
            2,
            ContainerFlags::IS_BLOCK_SCOPED_CONTAINER
        ));

        assert_eq!(stack.current_container().unwrap().node_id, 1);
        assert_eq!(stack.current_block_scope_container().unwrap().node_id, 2);

        stack.pop_container();
        assert!(stack.current_block_scope_container().is_none());
    }
}
