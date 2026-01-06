//! Symbol resolution for LSP operations.
//!
//! The ThinBinder maps declaration nodes to symbols, but LSP needs to resolve
//! identifier *usages* to symbols as well. This module provides a lightweight
//! scope walker that reconstructs scope chains on demand.

use crate::parser::thin_node::{ThinNodeArena, NodeAccess};
use crate::parser::{NodeIndex, syntax_kind_ext};
use crate::scanner::SyntaxKind;
use crate::binder::{SymbolId, SymbolTable};
use crate::thin_binder::ThinBinderState;
use rustc_hash::FxHashMap;

/// A lightweight scope chain reconstructed on demand.
///
/// This mimics the binder's scope logic but focuses on resolving identifiers
/// to symbols, rather than creating new symbols.
pub struct ScopeWalker<'a> {
    arena: &'a ThinNodeArena,
    binder: &'a ThinBinderState,
    /// Stack of active scopes (maps name -> SymbolId)
    scope_stack: Vec<SymbolTable>,
}

impl<'a> ScopeWalker<'a> {
    /// Create a new scope walker.
    pub fn new(arena: &'a ThinNodeArena, binder: &'a ThinBinderState) -> Self {
        Self {
            arena,
            binder,
            // Start with file-level scope
            scope_stack: vec![binder.file_locals.clone()],
        }
    }

    /// Push a new scope onto the stack.
    fn push_scope(&mut self) {
        self.scope_stack.push(SymbolTable::new());
    }

    /// Pop the current scope from the stack.
    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    /// Register a declaration in the current scope.
    fn declare_local(&mut self, name: String, sym_id: SymbolId) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.set(name, sym_id);
        }
    }

    /// Resolve a name by walking up the scope stack.
    fn resolve_name(&self, name: &str) -> Option<SymbolId> {
        for scope in self.scope_stack.iter().rev() {
            if let Some(id) = scope.get(name) {
                return Some(id);
            }
        }
        None
    }

    /// Resolve an identifier node to its symbol.
    ///
    /// This walks the AST from the root to the target node, maintaining
    /// scope state along the way, then resolves the identifier in the
    /// current scope.
    pub fn resolve_node(&mut self, root: NodeIndex, target: NodeIndex) -> Option<SymbolId> {
        // First check if this is a declaration node
        if let Some(&sym_id) = self.binder.node_symbols.get(&target.0) {
            return Some(sym_id);
        }

        // Otherwise, we need to walk the tree to build scope context
        self.walk_to_node(root, target, &mut Vec::new())
    }

    /// Walk the AST to find a target node, building scope context.
    ///
    /// Returns the symbol ID if the target is an identifier that can be resolved.
    fn walk_to_node(&mut self, current: NodeIndex, target: NodeIndex, path: &mut Vec<NodeIndex>) -> Option<SymbolId> {
        if current == target {
            // Found the target! Try to resolve it
            if let Some(node) = self.arena.get(current) {
                if node.kind == SyntaxKind::Identifier as u16 {
                    // Get the identifier text and resolve it
                    if let Some(text) = self.arena.get_identifier_text(current) {
                        return self.resolve_name(text);
                    }
                }
            }
            return None;
        }

        let Some(node) = self.arena.get(current) else {
            return None;
        };

        // Check if this creates a new scope
        let creates_scope = matches!(
            node.kind,
            syntax_kind_ext::BLOCK
                | syntax_kind_ext::FUNCTION_DECLARATION
                | syntax_kind_ext::ARROW_FUNCTION
                | syntax_kind_ext::FUNCTION_EXPRESSION
                | syntax_kind_ext::METHOD_DECLARATION
                | syntax_kind_ext::FOR_STATEMENT
                | syntax_kind_ext::FOR_IN_STATEMENT
                | syntax_kind_ext::FOR_OF_STATEMENT
                | syntax_kind_ext::CATCH_CLAUSE
                | syntax_kind_ext::CLASS_DECLARATION
                | syntax_kind_ext::CLASS_EXPRESSION
        );

        if creates_scope {
            self.push_scope();

            // Register declarations in this scope (from binder's map)
            // We need to find child declaration nodes and register them
            self.register_local_declarations(current);
        }

        path.push(current);

        // Visit children to find the target
        let result = self.visit_children(current, target, path);

        path.pop();

        if creates_scope {
            self.pop_scope();
        }

        result
    }

    /// Register local declarations from a container node.
    fn register_local_declarations(&mut self, container: NodeIndex) {
        // Find all declaration nodes that are direct children or in child blocks
        // and register them in the current scope
        for (&node_idx, &sym_id) in &self.binder.node_symbols {
            let node_index = NodeIndex(node_idx);
            // Simple heuristic: if the symbol is "nearby" in the index space,
            // it's likely a child declaration
            // A better implementation would track parent-child relationships
            if node_index.0 > container.0 && node_index.0 < container.0 + 1000 {
                if let Some(symbol) = self.binder.symbols.get(sym_id) {
                    self.declare_local(symbol.escaped_name.clone(), sym_id);
                }
            }
        }
    }

    /// Visit children of a node looking for the target.
    fn visit_children(&mut self, current: NodeIndex, target: NodeIndex, path: &mut Vec<NodeIndex>) -> Option<SymbolId> {
        // This is a simplified traversal - a full implementation would
        // use proper child accessors for each node type

        // For now, we'll scan nearby nodes in the arena
        let Some(node) = self.arena.get(current) else {
            return None;
        };

        // Simple approach: check nodes in the range [pos, end)
        let start_idx = current.0;
        let end_idx = (start_idx + 100).min(self.arena.len() as u32);

        for i in (start_idx + 1)..end_idx {
            let child_idx = NodeIndex(i);
            if let Some(child) = self.arena.get(child_idx) {
                // Only visit nodes that are within this node's span
                if child.pos >= node.pos && child.end <= node.end {
                    if let Some(result) = self.walk_to_node(child_idx, target, path) {
                        return Some(result);
                    }
                }
            }
        }

        None
    }

    /// Find all references to a symbol in the AST.
    pub fn find_references(&mut self, root: NodeIndex, target_symbol: SymbolId) -> Vec<NodeIndex> {
        let mut refs = Vec::new();

        // Get the target symbol's name
        let target_name = self.binder.symbols.get(target_symbol)
            .map(|s| s.escaped_name.clone());

        if let Some(name) = target_name {
            self.collect_references(root, &name, target_symbol, &mut refs);
        }

        refs
    }

    /// Recursively collect references to a symbol.
    fn collect_references(
        &mut self,
        current: NodeIndex,
        target_name: &str,
        target_symbol: SymbolId,
        refs: &mut Vec<NodeIndex>,
    ) {
        let Some(node) = self.arena.get(current) else {
            return;
        };

        // Check if this is an identifier with matching text
        if node.kind == SyntaxKind::Identifier as u16 {
            if let Some(text) = self.arena.get_identifier_text(current) {
                if text == target_name {
                    // Resolve it to see if it refers to our target symbol
                    if let Some(resolved) = self.resolve_node(NodeIndex(0), current) {
                        if resolved == target_symbol {
                            refs.push(current);
                        }
                    }
                }
            }
        }

        // Visit children (simplified traversal)
        let start_idx = current.0;
        let end_idx = (start_idx + 100).min(self.arena.len() as u32);

        for i in (start_idx + 1)..end_idx {
            let child_idx = NodeIndex(i);
            if let Some(child) = self.arena.get(child_idx) {
                if child.pos >= node.pos && child.end <= node.end {
                    self.collect_references(child_idx, target_name, target_symbol, refs);
                }
            }
        }
    }
}

#[cfg(test)]
mod resolver_tests {
    use super::*;
    use crate::thin_parser::ThinParserState;
    use crate::thin_binder::ThinBinderState;

    #[test]
    fn test_resolve_simple_variable() {
        // const x = 1; x + 1;
        let source = "const x = 1; x + 1;";
        let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
        let root = parser.parse_source_file();
        let arena = parser.get_arena();

        let mut binder = ThinBinderState::new();
        binder.bind_source_file(arena, root);

        // Should have a symbol for 'x'
        assert!(binder.file_locals.get("x").is_some());
    }
}
