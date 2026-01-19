//! Check Context
//!
//! Context for type checking operations with cycle detection.

use std::collections::HashSet;

/// Context for type checking operations
pub struct CheckContext {
    /// Resolution stack for cycle detection
    resolution_stack: HashSet<u64>,
    /// Maximum resolution depth
    max_depth: u32,
    /// Current resolution depth
    current_depth: u32,
    /// Whether we're in speculative mode
    pub speculative: bool,
}

impl CheckContext {
    /// Creates a new check context
    pub fn new() -> Self {
        Self {
            resolution_stack: HashSet::new(),
            max_depth: 100,
            current_depth: 0,
            speculative: false,
        }
    }

    /// Creates a context with custom max depth
    pub fn with_max_depth(max_depth: u32) -> Self {
        Self {
            max_depth,
            ..Self::new()
        }
    }

    /// Enters a resolution for the given ID
    pub fn enter_resolution(&mut self, id: u64) -> bool {
        if self.current_depth >= self.max_depth {
            return false;
        }
        if self.resolution_stack.contains(&id) {
            return false;
        }
        self.resolution_stack.insert(id);
        self.current_depth += 1;
        true
    }

    /// Exits a resolution for the given ID
    pub fn exit_resolution(&mut self, id: u64) {
        self.resolution_stack.remove(&id);
        self.current_depth = self.current_depth.saturating_sub(1);
    }

    /// Checks if resolving the given ID would cause a cycle
    pub fn would_cycle(&self, id: u64) -> bool {
        self.resolution_stack.contains(&id)
    }

    /// Gets the current resolution depth
    pub fn depth(&self) -> u32 {
        self.current_depth
    }
}

impl Default for CheckContext {
    fn default() -> Self {
        Self::new()
    }
}
