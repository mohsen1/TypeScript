//! ThinBinder - Binder implementation using ThinNodeArena.
//!
//! This is a clean implementation of the binder that works directly with
//! ThinNode and ThinNodeArena, avoiding the old Node enum pattern matching.

// Allow dead code for binder infrastructure methods that will be used in future phases
#![allow(dead_code)]

use crate::parser::thin_node::{ThinNodeArena, ThinNode};
use crate::parser::{NodeIndex, NodeList, syntax_kind_ext};
use crate::scanner::SyntaxKind;
use crate::binder::{
    SymbolId, SymbolArena, SymbolTable, Symbol, symbol_flags,
    FlowNodeArena, FlowNodeId, flow_flags,
    ContainerKind, ScopeContext, Scope, ScopeId,
};
use crate::parser::node_flags;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;

/// Binder state using ThinNodeArena.
pub struct ThinBinderState {
    /// Arena for symbol storage
    pub symbols: SymbolArena,
    /// Current symbol table (local scope)
    pub current_scope: SymbolTable,
    /// Stack of parent scopes
    scope_stack: Vec<SymbolTable>,
    /// File-level locals (for module resolution)
    pub file_locals: SymbolTable,
    /// Flow nodes for control flow analysis
    pub flow_nodes: FlowNodeArena,
    /// Current flow node
    current_flow: FlowNodeId,
    /// Unreachable flow node
    unreachable_flow: FlowNodeId,
    /// Scope chain - stack of scope contexts (legacy, for hoisting)
    scope_chain: Vec<ScopeContext>,
    /// Current scope index in scope_chain
    current_scope_idx: usize,
    /// Node-to-symbol mapping
    pub node_symbols: FxHashMap<u32, SymbolId>,
    /// Symbol-to-arena mapping for cross-file declaration lookup
    pub symbol_arenas: FxHashMap<SymbolId, Arc<ThinNodeArena>>,
    /// Node-to-flow mapping: tracks which flow node was active at each AST node
    /// Used by the checker for control flow analysis (type narrowing)
    pub node_flow: FxHashMap<u32, FlowNodeId>,
    /// Flow node after each top-level statement (for incremental binding).
    top_level_flow: FxHashMap<u32, FlowNodeId>,
    /// Map case/default clause nodes to their containing switch statement.
    switch_clause_to_switch: FxHashMap<u32, NodeIndex>,
    /// Hoisted var declarations
    hoisted_vars: Vec<(String, NodeIndex)>,
    /// Hoisted function declarations
    hoisted_functions: Vec<NodeIndex>,

    // ===== Persistent Scope System (for stateless checking) =====
    /// Persistent scopes - enables querying scope information without traversal order
    pub scopes: Vec<Scope>,
    /// Map from AST node (that creates a scope) to its ScopeId
    pub node_scope_ids: FxHashMap<u32, ScopeId>,
    /// Current active ScopeId during binding
    current_scope_id: ScopeId,
}

impl ThinBinderState {
    pub fn new() -> Self {
        let mut flow_nodes = FlowNodeArena::new();
        let unreachable_flow = flow_nodes.alloc(flow_flags::UNREACHABLE);

        ThinBinderState {
            symbols: SymbolArena::new(),
            current_scope: SymbolTable::new(),
            scope_stack: Vec::new(),
            file_locals: SymbolTable::new(),
            flow_nodes,
            current_flow: FlowNodeId::NONE,
            unreachable_flow,
            scope_chain: Vec::new(),
            current_scope_idx: 0,
            node_symbols: FxHashMap::default(),
            symbol_arenas: FxHashMap::default(),
            node_flow: FxHashMap::default(),
            top_level_flow: FxHashMap::default(),
            switch_clause_to_switch: FxHashMap::default(),
            hoisted_vars: Vec::new(),
            hoisted_functions: Vec::new(),
            scopes: Vec::new(),
            node_scope_ids: FxHashMap::default(),
            current_scope_id: ScopeId::NONE,
        }
    }

    pub fn reset(&mut self) {
        self.symbols.clear();
        self.current_scope.clear();
        self.scope_stack.clear();
        self.file_locals.clear();
        self.flow_nodes.clear();
        self.unreachable_flow = self.flow_nodes.alloc(flow_flags::UNREACHABLE);
        self.current_flow = FlowNodeId::NONE;
        self.scope_chain.clear();
        self.current_scope_idx = 0;
        self.node_symbols.clear();
        self.symbol_arenas.clear();
        self.node_flow.clear();
        self.top_level_flow.clear();
        self.switch_clause_to_switch.clear();
        self.hoisted_vars.clear();
        self.hoisted_functions.clear();
        self.scopes.clear();
        self.node_scope_ids.clear();
        self.current_scope_id = ScopeId::NONE;
    }

    /// Create a ThinBinderState from existing bound state.
    ///
    /// This is used for type checking after parallel binding and symbol merging.
    /// The symbols and node_symbols come from the merged program state.
    pub fn from_bound_state(
        symbols: SymbolArena,
        file_locals: SymbolTable,
        node_symbols: FxHashMap<u32, SymbolId>,
    ) -> Self {
        let mut flow_nodes = FlowNodeArena::new();
        let unreachable_flow = flow_nodes.alloc(flow_flags::UNREACHABLE);

        ThinBinderState {
            symbols,
            current_scope: SymbolTable::new(),
            scope_stack: Vec::new(),
            file_locals,
            flow_nodes,
            current_flow: FlowNodeId::NONE,
            unreachable_flow,
            scope_chain: Vec::new(),
            current_scope_idx: 0,
            node_symbols,
            symbol_arenas: FxHashMap::default(),
            node_flow: FxHashMap::default(),
            top_level_flow: FxHashMap::default(),
            switch_clause_to_switch: FxHashMap::default(),
            hoisted_vars: Vec::new(),
            hoisted_functions: Vec::new(),
            scopes: Vec::new(),
            node_scope_ids: FxHashMap::default(),
            current_scope_id: ScopeId::NONE,
        }
    }

    /// Create a ThinBinderState from existing bound state, preserving scopes.
    pub fn from_bound_state_with_scopes(
        symbols: SymbolArena,
        file_locals: SymbolTable,
        node_symbols: FxHashMap<u32, SymbolId>,
        scopes: Vec<Scope>,
        node_scope_ids: FxHashMap<u32, ScopeId>,
    ) -> Self {
        let mut flow_nodes = FlowNodeArena::new();
        let unreachable_flow = flow_nodes.alloc(flow_flags::UNREACHABLE);

        ThinBinderState {
            symbols,
            current_scope: SymbolTable::new(),
            scope_stack: Vec::new(),
            file_locals,
            flow_nodes,
            current_flow: FlowNodeId::NONE,
            unreachable_flow,
            scope_chain: Vec::new(),
            current_scope_idx: 0,
            node_symbols,
            symbol_arenas: FxHashMap::default(),
            node_flow: FxHashMap::default(),
            top_level_flow: FxHashMap::default(),
            switch_clause_to_switch: FxHashMap::default(),
            hoisted_vars: Vec::new(),
            hoisted_functions: Vec::new(),
            scopes,
            node_scope_ids,
            current_scope_id: ScopeId::NONE,
        }
    }

    /// Resolve an identifier to a symbol by walking up the persistent scope tree.
    /// This method enables stateless checking - the checker can query scope information
    /// without maintaining a traversal-order-dependent stack.
    ///
    /// Returns the SymbolId for the identifier, or None if not found.
    pub fn resolve_identifier(&self, arena: &ThinNodeArena, node_idx: NodeIndex) -> Option<SymbolId> {
        let node = arena.get(node_idx)?;

        // Get the identifier text
        let name = if let Some(ident) = arena.get_identifier(node) {
            &ident.escaped_text
        } else {
            return None;
        };

        if let Some(mut scope_id) = self.find_enclosing_scope(arena, node_idx) {
            // Walk up the scope chain
            while !scope_id.is_none() {
                if let Some(scope) = self.scopes.get(scope_id.0 as usize) {
                    if let Some(sym_id) = scope.table.get(name) {
                        return Some(sym_id);
                    }
                    scope_id = scope.parent;
                } else {
                    break;
                }
            }
        }

        // Fallback for bound-state binders without persistent scopes.
        if let Some(sym_id) = self.resolve_parameter_fallback(arena, node_idx, name) {
            return Some(sym_id);
        }

        // Finally check file locals / globals
        self.file_locals.get(name)
    }

    fn resolve_parameter_fallback(
        &self,
        arena: &ThinNodeArena,
        node_idx: NodeIndex,
        name: &str,
    ) -> Option<SymbolId> {
        if self.scopes.is_empty() {
            let mut current = node_idx;
            while !current.is_none() {
                let node = arena.get(current)?;
                if let Some(func) = arena.get_function(node) {
                    for &param_idx in &func.parameters.nodes {
                        let param_node = arena.get(param_idx)?;
                        let param = arena.get_parameter(param_node)?;
                        let ident_node = arena.get(param.name)?;
                        let ident = arena.get_identifier(ident_node)?;
                        if ident.escaped_text == name {
                            return self.node_symbols.get(&param.name.0).copied();
                        }
                    }
                }
                let ext = arena.get_extended(current)?;
                current = ext.parent;
            }
        }
        None
    }

    /// Find the enclosing scope for a given node by walking up the AST.
    /// Returns the ScopeId of the nearest scope-creating ancestor node.
    fn find_enclosing_scope(&self, arena: &ThinNodeArena, node_idx: NodeIndex) -> Option<ScopeId> {
        let mut current = node_idx;

        // Walk up the AST using parent pointers to find the nearest scope
        while !current.is_none() {
            // Check if this node creates a scope
            if let Some(&scope_id) = self.node_scope_ids.get(&current.0) {
                return Some(scope_id);
            }

            // Move to parent node
            if let Some(node) = arena.get(current) {
                if let Some(ext) = arena.get_extended(current) {
                    current = ext.parent;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // If no scope found, return the root scope (index 0) if it exists
        if !self.scopes.is_empty() {
            Some(ScopeId(0))
        } else {
            None
        }
    }

    /// Enter a new persistent scope (in addition to legacy scope chain).
    /// This method is called when binding begins for a scope-creating node.
    fn enter_persistent_scope(&mut self, kind: ContainerKind, node: NodeIndex) {
        // Create new scope linked to current
        let new_scope_id = ScopeId(self.scopes.len() as u32);
        let new_scope = Scope::new(self.current_scope_id, kind, node);
        self.scopes.push(new_scope);

        // Map node to this scope
        if !node.is_none() {
            self.node_scope_ids.insert(node.0, new_scope_id);
        }

        // Update current scope
        self.current_scope_id = new_scope_id;
    }

    /// Exit the current persistent scope.
    fn exit_persistent_scope(&mut self) {
        if !self.current_scope_id.is_none() {
            if let Some(scope) = self.scopes.get(self.current_scope_id.0 as usize) {
                self.current_scope_id = scope.parent;
            }
        }
    }

    /// Declare a symbol in the current persistent scope.
    /// This adds the symbol to the persistent scope table for later querying.
    fn declare_in_persistent_scope(&mut self, name: String, sym_id: SymbolId) {
        if !self.current_scope_id.is_none() {
            if let Some(scope) = self.scopes.get_mut(self.current_scope_id.0 as usize) {
                scope.table.set(name, sym_id);
            }
        }
    }

    fn sync_current_scope_to_persistent(&mut self) {
        if self.current_scope_id.is_none() {
            return;
        }
        if let Some(persistent_scope) = self.scopes.get_mut(self.current_scope_id.0 as usize) {
            for (name, &sym_id) in self.current_scope.iter() {
                persistent_scope.table.set(name.clone(), sym_id);
            }
        }
    }

    /// Bind a source file using ThinNodeArena.
    pub fn bind_source_file(&mut self, arena: &ThinNodeArena, root: NodeIndex) {
        // Initialize scope chain with source file scope (legacy)
        self.scope_chain.clear();
        self.scope_chain.push(ScopeContext::new(ContainerKind::SourceFile, root, None));
        self.current_scope_idx = 0;
        self.current_scope = SymbolTable::new();

        // Initialize persistent scope system
        self.scopes.clear();
        self.node_scope_ids.clear();
        self.current_scope_id = ScopeId::NONE;
        self.top_level_flow.clear();

        // Create root persistent scope for the source file
        self.enter_persistent_scope(ContainerKind::SourceFile, root);

        // Create START flow node for the file
        let start_flow = self.flow_nodes.alloc(flow_flags::START);
        self.current_flow = start_flow;

        if let Some(node) = arena.get(root) {
            if let Some(sf) = arena.get_source_file(node) {
                // First pass: collect hoisted declarations
                self.collect_hoisted_declarations(arena, &sf.statements);

                // Process hoisted function declarations first
                self.process_hoisted_functions(arena);

                // Second pass: bind each statement
                for &stmt_idx in &sf.statements.nodes {
                    self.bind_node(arena, stmt_idx);
                    self.top_level_flow.insert(stmt_idx.0, self.current_flow);
                }
            }
        }

        self.sync_current_scope_to_persistent();

        // Store file locals
        self.file_locals = std::mem::take(&mut self.current_scope);
    }

    /// Incrementally bind new statements after a prefix without rebinding the entire file.
    pub fn bind_source_file_incremental(
        &mut self,
        arena: &ThinNodeArena,
        root: NodeIndex,
        prefix_statements: &[NodeIndex],
        old_suffix_statements: &[NodeIndex],
        new_suffix_statements: &[NodeIndex],
        reparse_start: u32,
    ) -> bool {
        let last_prefix = match prefix_statements.last() {
            Some(stmt) => *stmt,
            None => return false,
        };
        let start_flow = match self.top_level_flow.get(&last_prefix.0) {
            Some(flow) => *flow,
            None => return false,
        };
        if self.scopes.is_empty() {
            return false;
        }

        self.prune_incremental_maps(arena, reparse_start);

        let mut prefix_names = FxHashSet::default();
        self.collect_file_scope_names_for_statements(arena, prefix_statements, &mut prefix_names);

        let mut old_suffix_names = FxHashSet::default();
        self.collect_file_scope_names_for_statements(arena, old_suffix_statements, &mut old_suffix_names);

        for name in old_suffix_names {
            if prefix_names.contains(&name) {
                continue;
            }
            self.file_locals.remove(&name);
            if let Some(scope) = self.scopes.get_mut(0) {
                scope.table.remove(&name);
            }
        }

        let mut symbol_nodes = Vec::new();
        self.collect_statement_symbol_nodes(arena, old_suffix_statements, &mut symbol_nodes);
        for node in symbol_nodes {
            if let Some(sym_id) = self.node_symbols.remove(&node.0) {
                if let Some(sym) = self.symbols.get_mut(sym_id) {
                    sym.declarations.retain(|decl| *decl != node);
                    if sym.value_declaration == node {
                        sym.value_declaration = sym.declarations.first().copied().unwrap_or(NodeIndex::NONE);
                    }
                }
            }
        }

        for stmt_idx in old_suffix_statements {
            self.top_level_flow.remove(&stmt_idx.0);
        }

        // Reset transient binding state while keeping existing symbols and scopes.
        self.scope_chain.clear();
        self.scope_chain.push(ScopeContext::new(ContainerKind::SourceFile, root, None));
        self.current_scope_idx = 0;
        self.scope_stack.clear();
        self.current_scope = self.file_locals.clone();
        self.hoisted_vars.clear();
        self.hoisted_functions.clear();
        self.current_scope_id = ScopeId(0);
        self.current_flow = start_flow;

        let new_suffix_list = NodeList {
            nodes: new_suffix_statements.to_vec(),
            pos: 0,
            end: 0,
            has_trailing_comma: false,
        };

        self.collect_hoisted_declarations(arena, &new_suffix_list);
        self.process_hoisted_functions(arena);

        for &stmt_idx in new_suffix_statements {
            self.bind_node(arena, stmt_idx);
            self.top_level_flow.insert(stmt_idx.0, self.current_flow);
        }

        self.sync_current_scope_to_persistent();
        self.file_locals = std::mem::take(&mut self.current_scope);

        true
    }

    fn prune_incremental_maps(&mut self, arena: &ThinNodeArena, reparse_start: u32) {
        if reparse_start == 0 {
            return;
        }

        let keep_node = |node_id: &u32| {
            arena
                .get(NodeIndex(*node_id))
                .map_or(false, |node| node.pos < reparse_start)
        };

        self.node_flow.retain(|node_id, _| keep_node(node_id));
        self.node_scope_ids.retain(|node_id, _| keep_node(node_id));
        self.switch_clause_to_switch.retain(|node_id, _| keep_node(node_id));
    }

    /// Collect hoisted declarations from statements.
    fn collect_hoisted_declarations(&mut self, arena: &ThinNodeArena, statements: &NodeList) {
        for &stmt_idx in &statements.nodes {
            if let Some(node) = arena.get(stmt_idx) {
                match node.kind {
                    k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                        if let Some(var_stmt) = arena.get_variable(node) {
                            // VariableStatement stores declaration_list as first element
                            if let Some(&decl_list_idx) = var_stmt.declarations.nodes.first() {
                                self.collect_hoisted_var_decl(arena, decl_list_idx);
                            }
                        }
                    }
                    k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                        self.hoisted_functions.push(stmt_idx);
                    }
                    k if k == syntax_kind_ext::BLOCK => {
                        if let Some(block) = arena.get_block(node) {
                            self.collect_hoisted_declarations(arena, &block.statements);
                        }
                    }
                    k if k == syntax_kind_ext::IF_STATEMENT => {
                        if let Some(if_stmt) = arena.get_if_statement(node) {
                            self.collect_hoisted_from_node(arena, if_stmt.then_statement);
                            if !if_stmt.else_statement.is_none() {
                                self.collect_hoisted_from_node(arena, if_stmt.else_statement);
                            }
                        }
                    }
                    k if k == syntax_kind_ext::WHILE_STATEMENT ||
                         k == syntax_kind_ext::DO_STATEMENT ||
                         k == syntax_kind_ext::FOR_STATEMENT => {
                        if let Some(loop_data) = arena.get_loop(node) {
                            self.collect_hoisted_from_node(arena, loop_data.statement);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn collect_hoisted_var_decl(&mut self, arena: &ThinNodeArena, decl_list_idx: NodeIndex) {
        if let Some(node) = arena.get(decl_list_idx) {
            if let Some(list) = arena.get_variable(node) {
                // Check if this is a var declaration (not let/const)
                let is_var = (node.flags as u32 & (node_flags::LET | node_flags::CONST)) == 0;
                if is_var {
                    for &decl_idx in &list.declarations.nodes {
                        if let Some(decl_node) = arena.get(decl_idx) {
                            if let Some(decl) = arena.get_variable_declaration(decl_node) {
                                if let Some(name) = self.get_identifier_name(arena, decl.name) {
                                    self.hoisted_vars.push((name.to_string(), decl_idx));
                                } else {
                                    let mut names = Vec::new();
                                    self.collect_binding_identifiers(arena, decl.name, &mut names);
                                    for ident_idx in names {
                                        if let Some(name) = self.get_identifier_name(arena, ident_idx) {
                                            self.hoisted_vars.push((name.to_string(), ident_idx));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_hoisted_from_node(&mut self, arena: &ThinNodeArena, idx: NodeIndex) {
        if let Some(node) = arena.get(idx) {
            if node.kind == syntax_kind_ext::BLOCK {
                if let Some(block) = arena.get_block(node) {
                    self.collect_hoisted_declarations(arena, &block.statements);
                }
            }
        }
    }

    /// Process hoisted function declarations.
    fn process_hoisted_functions(&mut self, arena: &ThinNodeArena) {
        let functions = std::mem::take(&mut self.hoisted_functions);
        for func_idx in functions {
            if let Some(node) = arena.get(func_idx) {
                if let Some(func) = arena.get_function(node) {
                    if let Some(name) = self.get_identifier_name(arena, func.name) {
                        let is_exported = self.has_export_modifier(arena, &func.modifiers);
                        let sym_id = self.declare_symbol(name, symbol_flags::FUNCTION, func_idx, is_exported);

                        // Also add to persistent scope
                        self.declare_in_persistent_scope(name.to_string(), sym_id);
                    }
                }
            }
        }
    }

    /// Bind a node and its children.
    fn bind_node(&mut self, arena: &ThinNodeArena, idx: NodeIndex) {
        if idx.is_none() {
            return;
        }

        let node = match arena.get(idx) {
            Some(n) => n,
            None => return,
        };

        match node.kind {
            k if k == SyntaxKind::Identifier as u16 => {
                self.record_flow(idx);
                return;
            }
            // Variable declarations
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                if let Some(var_stmt) = arena.get_variable(node) {
                    // VariableStatement stores declaration_list as first element
                    if let Some(&decl_list_idx) = var_stmt.declarations.nodes.first() {
                        self.bind_node(arena, decl_list_idx);
                    }
                }
            }
            k if k == syntax_kind_ext::VARIABLE_DECLARATION_LIST => {
                if let Some(list) = arena.get_variable(node) {
                    for &decl_idx in &list.declarations.nodes {
                        self.bind_node(arena, decl_idx);
                    }
                }
            }
            k if k == syntax_kind_ext::VARIABLE_DECLARATION => {
                self.bind_variable_declaration(arena, node, idx);
            }

            // Function declarations
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                self.bind_function_declaration(arena, node, idx);
            }

            // Class declarations
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                self.bind_class_declaration(arena, node, idx);
            }
            k if k == syntax_kind_ext::CLASS_EXPRESSION => {
                self.bind_class_expression(arena, node, idx);
            }

            // Interface declarations
            k if k == syntax_kind_ext::INTERFACE_DECLARATION => {
                self.bind_interface_declaration(arena, node, idx);
            }

            // Type alias declarations
            k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                self.bind_type_alias_declaration(arena, node, idx);
            }

            // Enum declarations
            k if k == syntax_kind_ext::ENUM_DECLARATION => {
                self.bind_enum_declaration(arena, node, idx);
            }

            // Block - creates a new block scope
            k if k == syntax_kind_ext::BLOCK => {
                if let Some(block) = arena.get_block(node) {
                    self.enter_scope(ContainerKind::Block, idx);
                    for &stmt_idx in &block.statements.nodes {
                        self.bind_node(arena, stmt_idx);
                    }
                    self.exit_scope();
                }
            }

            // If statement - build flow graph for type narrowing
            k if k == syntax_kind_ext::IF_STATEMENT => {
                if let Some(if_stmt) = arena.get_if_statement(node) {
                    // Bind the condition expression (record identifiers in it)
                    self.bind_expression(arena, if_stmt.expression);

                    // Save the pre-condition flow
                    let pre_condition_flow = self.current_flow;

                    // Create TRUE_CONDITION flow for the then branch
                    let true_flow = self.create_flow_condition(
                        flow_flags::TRUE_CONDITION,
                        pre_condition_flow,
                        if_stmt.expression,
                    );

                    // Bind the then branch with narrowed flow
                    self.current_flow = true_flow;
                    self.bind_node(arena, if_stmt.then_statement);
                    let after_then_flow = self.current_flow;

                    // Handle else branch if present
                    let after_else_flow = if !if_stmt.else_statement.is_none() {
                        // Create FALSE_CONDITION flow for the else branch
                        let false_flow = self.create_flow_condition(
                            flow_flags::FALSE_CONDITION,
                            pre_condition_flow,
                            if_stmt.expression,
                        );

                        // Bind the else branch with narrowed flow
                        self.current_flow = false_flow;
                        self.bind_node(arena, if_stmt.else_statement);
                        self.current_flow
                    } else {
                        // No else branch - false condition goes directly to merge
                        self.create_flow_condition(
                            flow_flags::FALSE_CONDITION,
                            pre_condition_flow,
                            if_stmt.expression,
                        )
                    };

                    // Create merge point for branches
                    let merge_label = self.create_branch_label();
                    self.add_antecedent(merge_label, after_then_flow);
                    self.add_antecedent(merge_label, after_else_flow);
                    self.current_flow = merge_label;
                }
            }

            // While/do statement
            k if k == syntax_kind_ext::WHILE_STATEMENT ||
                 k == syntax_kind_ext::DO_STATEMENT => {
                if let Some(loop_data) = arena.get_loop(node) {
                    let pre_loop_flow = self.current_flow;
                    let loop_label = self.create_loop_label();
                    if !self.current_flow.is_none() {
                        self.add_antecedent(loop_label, self.current_flow);
                    }
                    self.current_flow = loop_label;

                    if node.kind == syntax_kind_ext::DO_STATEMENT {
                        self.bind_node(arena, loop_data.statement);
                        self.bind_expression(arena, loop_data.condition);

                        let pre_condition_flow = self.current_flow;
                        let true_flow = self.create_flow_condition(
                            flow_flags::TRUE_CONDITION,
                            pre_condition_flow,
                            loop_data.condition,
                        );
                        self.add_antecedent(loop_label, true_flow);

                        let false_flow = self.create_flow_condition(
                            flow_flags::FALSE_CONDITION,
                            pre_condition_flow,
                            loop_data.condition,
                        );
                        let merge_label = self.create_branch_label();
                        self.add_antecedent(merge_label, pre_condition_flow);
                        self.add_antecedent(merge_label, false_flow);
                        self.current_flow = merge_label;
                    } else {
                        self.bind_expression(arena, loop_data.condition);

                        let pre_condition_flow = self.current_flow;
                        let true_flow = self.create_flow_condition(
                            flow_flags::TRUE_CONDITION,
                            pre_condition_flow,
                            loop_data.condition,
                        );
                        self.current_flow = true_flow;
                        self.bind_node(arena, loop_data.statement);
                        self.add_antecedent(loop_label, self.current_flow);

                        let false_flow = self.create_flow_condition(
                            flow_flags::FALSE_CONDITION,
                            pre_condition_flow,
                            loop_data.condition,
                        );
                        let merge_label = self.create_branch_label();
                        self.add_antecedent(merge_label, pre_loop_flow);
                        self.add_antecedent(merge_label, false_flow);
                        self.current_flow = merge_label;
                    }
                }
            }

            // For statement
            k if k == syntax_kind_ext::FOR_STATEMENT => {
                if let Some(loop_data) = arena.get_loop(node) {
                    self.enter_scope(ContainerKind::Block, idx);
                    self.bind_node(arena, loop_data.initializer);

                    let pre_loop_flow = self.current_flow;
                    let loop_label = self.create_loop_label();
                    if !self.current_flow.is_none() {
                        self.add_antecedent(loop_label, self.current_flow);
                    }
                    self.current_flow = loop_label;

                    if !loop_data.condition.is_none() {
                        self.bind_expression(arena, loop_data.condition);
                        let pre_condition_flow = self.current_flow;
                        let true_flow = self.create_flow_condition(
                            flow_flags::TRUE_CONDITION,
                            pre_condition_flow,
                            loop_data.condition,
                        );
                        self.current_flow = true_flow;
                        self.bind_node(arena, loop_data.statement);
                        self.bind_expression(arena, loop_data.incrementor);
                        self.add_antecedent(loop_label, self.current_flow);

                        let false_flow = self.create_flow_condition(
                            flow_flags::FALSE_CONDITION,
                            pre_condition_flow,
                            loop_data.condition,
                        );
                        let merge_label = self.create_branch_label();
                        self.add_antecedent(merge_label, pre_loop_flow);
                        self.add_antecedent(merge_label, false_flow);
                        self.current_flow = merge_label;
                    } else {
                        self.bind_node(arena, loop_data.statement);
                        self.bind_expression(arena, loop_data.incrementor);
                        self.add_antecedent(loop_label, self.current_flow);
                        let merge_label = self.create_branch_label();
                        self.add_antecedent(merge_label, loop_label);
                        self.add_antecedent(merge_label, self.current_flow);
                        self.current_flow = merge_label;
                    }
                    self.exit_scope();
                }
            }

            // For-in/for-of
            k if k == syntax_kind_ext::FOR_IN_STATEMENT ||
                 k == syntax_kind_ext::FOR_OF_STATEMENT => {
                if let Some(for_data) = arena.get_for_in_of(node) {
                    self.enter_scope(ContainerKind::Block, idx);
                    self.bind_node(arena, for_data.initializer);
                    let loop_label = self.create_loop_label();
                    if !self.current_flow.is_none() {
                        self.add_antecedent(loop_label, self.current_flow);
                    }
                    self.current_flow = loop_label;

                    self.bind_expression(arena, for_data.expression);
                    self.bind_node(arena, for_data.statement);
                    self.add_antecedent(loop_label, self.current_flow);
                    let merge_label = self.create_branch_label();
                    self.add_antecedent(merge_label, loop_label);
                    self.add_antecedent(merge_label, self.current_flow);
                    self.current_flow = merge_label;
                    self.exit_scope();
                }
            }

            // Switch statement
            k if k == syntax_kind_ext::SWITCH_STATEMENT => {
                self.bind_switch_statement(arena, node, idx);
            }

            // Try statement
            k if k == syntax_kind_ext::TRY_STATEMENT => {
                self.bind_try_statement(arena, node, idx);
            }

            // Labeled statement
            k if k == syntax_kind_ext::LABELED_STATEMENT => {
                if let Some(labeled) = arena.get_labeled_statement(node) {
                    self.bind_node(arena, labeled.statement);
                }
            }

            // With statement
            k if k == syntax_kind_ext::WITH_STATEMENT => {
                if let Some(with_stmt) = arena.get_with_statement(node) {
                    self.bind_node(arena, with_stmt.expression);
                    self.bind_node(arena, with_stmt.then_statement);
                }
            }

            // Import declarations
            k if k == syntax_kind_ext::IMPORT_DECLARATION => {
                self.bind_import_declaration(arena, node, idx);
            }

            // Import equals declaration (import x = ns.member)
            k if k == syntax_kind_ext::IMPORT_EQUALS_DECLARATION => {
                self.bind_import_equals_declaration(arena, node, idx);
            }

            // Export declarations - bind the exported declaration
            k if k == syntax_kind_ext::EXPORT_DECLARATION => {
                self.bind_export_declaration(arena, node, idx);
            }
            // Export assignment - bind the assigned expression
            k if k == syntax_kind_ext::EXPORT_ASSIGNMENT => {
                if let Some(assign) = arena.get_export_assignment(node) {
                    self.bind_node(arena, assign.expression);
                }
            }

            // Module/namespace declarations
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                self.bind_module_declaration(arena, node, idx);
            }
            k if k == syntax_kind_ext::MODULE_BLOCK => {
                if let Some(block) = arena.get_module_block(node) {
                    if let Some(ref statements) = block.statements {
                        for &stmt_idx in &statements.nodes {
                            self.bind_node(arena, stmt_idx);
                        }
                    }
                }
            }

            // Expression statements - traverse into the expression
            k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                if let Some(expr_stmt) = arena.get_expression_statement(node) {
                    self.bind_node(arena, expr_stmt.expression);
                }
            }

            // Return/throw statements - traverse into the expression
            k if k == syntax_kind_ext::RETURN_STATEMENT || k == syntax_kind_ext::THROW_STATEMENT => {
                if let Some(ret) = arena.get_return_statement(node) {
                    if !ret.expression.is_none() {
                        self.bind_node(arena, ret.expression);
                    }
                }
            }

            // Binary expressions - traverse into operands
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                if let Some(bin) = arena.get_binary_expr(node) {
                    if self.is_assignment_operator(bin.operator_token) {
                        self.bind_node(arena, bin.left);
                        self.bind_node(arena, bin.right);
                        let flow = self.create_flow_assignment(idx);
                        self.current_flow = flow;
                        return;
                    }
                }
                self.bind_binary_expression_iterative(arena, idx);
            }

            // Conditional expressions - traverse into branches
            k if k == syntax_kind_ext::CONDITIONAL_EXPRESSION => {
                if let Some(cond) = arena.get_conditional_expr(node) {
                    self.bind_node(arena, cond.condition);
                    self.bind_node(arena, cond.when_true);
                    self.bind_node(arena, cond.when_false);
                }
            }

            // Property access / element access
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION
                || k == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION => {
                self.record_flow(idx);
                if let Some(access) = arena.get_access_expr(node) {
                    self.bind_node(arena, access.expression);
                    self.bind_node(arena, access.name_or_argument);
                }
            }

            // Prefix/postfix unary expressions
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION
                || k == syntax_kind_ext::POSTFIX_UNARY_EXPRESSION => {
                if let Some(unary) = arena.get_unary_expr(node) {
                    self.bind_node(arena, unary.operand);
                    if unary.operator == SyntaxKind::PlusPlusToken as u16
                        || unary.operator == SyntaxKind::MinusMinusToken as u16
                    {
                        let flow = self.create_flow_assignment(idx);
                        self.current_flow = flow;
                    }
                }
            }

            // Await/yield expressions
            k if k == syntax_kind_ext::AWAIT_EXPRESSION
                || k == syntax_kind_ext::YIELD_EXPRESSION
                || k == syntax_kind_ext::NON_NULL_EXPRESSION => {
                if node.has_data() {
                    if let Some(unary) = arena.unary_exprs_ex.get(node.data_index as usize) {
                        self.bind_node(arena, unary.expression);
                    }
                }
            }

            // Type assertions / as / satisfies
            k if k == syntax_kind_ext::TYPE_ASSERTION
                || k == syntax_kind_ext::AS_EXPRESSION
                || k == syntax_kind_ext::SATISFIES_EXPRESSION => {
                if node.has_data() {
                    if let Some(assertion) = arena.type_assertions.get(node.data_index as usize) {
                        self.bind_node(arena, assertion.expression);
                    }
                }
            }

            // Decorators
            k if k == syntax_kind_ext::DECORATOR => {
                if let Some(decorator) = arena.get_decorator(node) {
                    self.bind_node(arena, decorator.expression);
                }
            }

            // Tagged templates
            k if k == syntax_kind_ext::TAGGED_TEMPLATE_EXPRESSION => {
                if node.has_data() {
                    if let Some(tagged) = arena.tagged_templates.get(node.data_index as usize) {
                        self.bind_node(arena, tagged.tag);
                        self.bind_node(arena, tagged.template);
                    }
                }
            }

            // Template expressions
            k if k == syntax_kind_ext::TEMPLATE_EXPRESSION => {
                if let Some(template) = arena.get_template_expr(node) {
                    self.bind_node(arena, template.head);
                    for &span in &template.template_spans.nodes {
                        self.bind_node(arena, span);
                    }
                }
            }
            k if k == syntax_kind_ext::TEMPLATE_SPAN => {
                if let Some(span) = arena.get_template_span(node) {
                    self.bind_node(arena, span.expression);
                    self.bind_node(arena, span.literal);
                }
            }

            // Object/array literals
            k if k == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION
                || k == syntax_kind_ext::ARRAY_LITERAL_EXPRESSION => {
                if let Some(lit) = arena.get_literal_expr(node) {
                    for &elem in &lit.elements.nodes {
                        self.bind_node(arena, elem);
                    }
                }
            }
            k if k == syntax_kind_ext::PROPERTY_ASSIGNMENT => {
                if let Some(prop) = arena.get_property_assignment(node) {
                    self.bind_node(arena, prop.name);
                    self.bind_node(arena, prop.initializer);
                }
            }
            k if k == syntax_kind_ext::SHORTHAND_PROPERTY_ASSIGNMENT => {
                if let Some(prop) = arena.get_shorthand_property(node) {
                    self.bind_node(arena, prop.name);
                    if !prop.object_assignment_initializer.is_none() {
                        self.bind_node(arena, prop.object_assignment_initializer);
                    }
                }
            }
            k if k == syntax_kind_ext::SPREAD_ELEMENT
                || k == syntax_kind_ext::SPREAD_ASSIGNMENT => {
                if let Some(spread) = arena.get_spread(node) {
                    self.bind_node(arena, spread.expression);
                }
            }
            k if k == syntax_kind_ext::COMPUTED_PROPERTY_NAME => {
                if let Some(computed) = arena.get_computed_property(node) {
                    self.bind_node(arena, computed.expression);
                }
            }

            // Call expressions - traverse into callee and arguments
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                if let Some(call) = arena.get_call_expr(node) {
                    self.bind_node(arena, call.expression);
                    if let Some(args) = &call.arguments {
                        for &arg in &args.nodes {
                            self.bind_node(arena, arg);
                        }
                    }
                    if self.is_array_mutation_call(arena, idx) {
                        let flow = self.create_flow_array_mutation(idx);
                        self.current_flow = flow;
                    }
                }
            }

            // New expressions - traverse into expression and arguments
            k if k == syntax_kind_ext::NEW_EXPRESSION => {
                if let Some(new_expr) = arena.get_call_expr(node) {
                    self.bind_node(arena, new_expr.expression);
                    if let Some(args) = &new_expr.arguments {
                        for &arg in &args.nodes {
                            self.bind_node(arena, arg);
                        }
                    }
                }
            }

            // Parenthesized expressions - traverse into inner expression
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                if let Some(paren) = arena.get_parenthesized(node) {
                    self.bind_node(arena, paren.expression);
                }
            }

            // Arrow function expressions - bind body
            k if k == syntax_kind_ext::ARROW_FUNCTION => {
                self.bind_arrow_function(arena, node, idx);
            }

            // Function expressions - bind body
            k if k == syntax_kind_ext::FUNCTION_EXPRESSION => {
                self.bind_function_expression(arena, node, idx);
            }

            _ => {
                // For other node types, no symbols to create
            }
        }
    }

    /// Get identifier name from a node index.
    fn get_identifier_name<'a>(&self, arena: &'a ThinNodeArena, idx: NodeIndex) -> Option<&'a str> {
        if let Some(node) = arena.get(idx) {
            if let Some(id) = arena.get_identifier(node) {
                return Some(&id.escaped_text);
            }
        }
        None
    }

    fn collect_binding_identifiers(&self, arena: &ThinNodeArena, idx: NodeIndex, out: &mut Vec<NodeIndex>) {
        if idx.is_none() {
            return;
        }

        let Some(node) = arena.get(idx) else {
            return;
        };

        match node.kind {
            k if k == SyntaxKind::Identifier as u16 => {
                out.push(idx);
            }
            k if k == syntax_kind_ext::BINDING_ELEMENT => {
                if let Some(binding) = arena.get_binding_element(node) {
                    self.collect_binding_identifiers(arena, binding.name, out);
                }
            }
            k if k == syntax_kind_ext::OBJECT_BINDING_PATTERN
                || k == syntax_kind_ext::ARRAY_BINDING_PATTERN => {
                if let Some(pattern) = arena.get_binding_pattern(node) {
                    for &elem in &pattern.elements.nodes {
                        if elem.is_none() {
                            continue;
                        }
                        self.collect_binding_identifiers(arena, elem, out);
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_file_scope_names_for_statements(
        &self,
        arena: &ThinNodeArena,
        statements: &[NodeIndex],
        out: &mut FxHashSet<String>,
    ) {
        for &stmt_idx in statements {
            self.collect_file_scope_names_for_statement(arena, stmt_idx, out);
        }
    }

    fn collect_file_scope_names_for_statement(
        &self,
        arena: &ThinNodeArena,
        idx: NodeIndex,
        out: &mut FxHashSet<String>,
    ) {
        let Some(node) = arena.get(idx) else {
            return;
        };

        match node.kind {
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                if let Some(var_stmt) = arena.get_variable(node) {
                    for &decl_list_idx in &var_stmt.declarations.nodes {
                        self.collect_variable_decl_names(arena, decl_list_idx, true, out);
                    }
                }
            }
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                if let Some(func) = arena.get_function(node) {
                    if let Some(name) = self.get_identifier_name(arena, func.name) {
                        out.insert(name.to_string());
                    }
                }
            }
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                if let Some(class) = arena.get_class(node) {
                    if let Some(name) = self.get_identifier_name(arena, class.name) {
                        out.insert(name.to_string());
                    }
                }
            }
            k if k == syntax_kind_ext::INTERFACE_DECLARATION => {
                if let Some(iface) = arena.get_interface(node) {
                    if let Some(name) = self.get_identifier_name(arena, iface.name) {
                        out.insert(name.to_string());
                    }
                }
            }
            k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                if let Some(alias) = arena.get_type_alias(node) {
                    if let Some(name) = self.get_identifier_name(arena, alias.name) {
                        out.insert(name.to_string());
                    }
                }
            }
            k if k == syntax_kind_ext::ENUM_DECLARATION => {
                if let Some(enum_decl) = arena.get_enum(node) {
                    if let Some(name) = self.get_identifier_name(arena, enum_decl.name) {
                        out.insert(name.to_string());
                    }
                }
            }
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                if let Some(module) = arena.get_module(node) {
                    if let Some(name) = self.get_identifier_name(arena, module.name) {
                        out.insert(name.to_string());
                    }
                }
            }
            k if k == syntax_kind_ext::IMPORT_DECLARATION => {
                self.collect_import_names(arena, node, out);
            }
            k if k == syntax_kind_ext::IMPORT_EQUALS_DECLARATION => {
                if let Some(import) = arena.get_import_decl(node) {
                    if let Some(name) = self.get_identifier_name(arena, import.import_clause) {
                        out.insert(name.to_string());
                    }
                }
            }
            k if k == syntax_kind_ext::EXPORT_DECLARATION => {
                if let Some(export) = arena.get_export_decl(node) {
                    if export.export_clause.is_none() {
                        return;
                    }
                    let Some(clause_node) = arena.get(export.export_clause) else {
                        return;
                    };
                    if self.is_declaration(clause_node.kind) {
                        self.collect_file_scope_names_for_statement(arena, export.export_clause, out);
                    } else if clause_node.kind == SyntaxKind::Identifier as u16 {
                        if let Some(name) = self.get_identifier_name(arena, export.export_clause) {
                            out.insert(name.to_string());
                        }
                    }
                }
            }
            k if k == syntax_kind_ext::BLOCK
                || k == syntax_kind_ext::IF_STATEMENT
                || k == syntax_kind_ext::WHILE_STATEMENT
                || k == syntax_kind_ext::DO_STATEMENT
                || k == syntax_kind_ext::FOR_STATEMENT =>
            {
                self.collect_hoisted_file_scope_names(arena, idx, out);
            }
            _ => {}
        }
    }

    fn collect_hoisted_file_scope_names(
        &self,
        arena: &ThinNodeArena,
        idx: NodeIndex,
        out: &mut FxHashSet<String>,
    ) {
        let Some(node) = arena.get(idx) else {
            return;
        };

        match node.kind {
            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                if let Some(var_stmt) = arena.get_variable(node) {
                    if let Some(&decl_list_idx) = var_stmt.declarations.nodes.first() {
                        self.collect_variable_decl_names(arena, decl_list_idx, false, out);
                    }
                }
            }
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                if let Some(func) = arena.get_function(node) {
                    if let Some(name) = self.get_identifier_name(arena, func.name) {
                        out.insert(name.to_string());
                    }
                }
            }
            k if k == syntax_kind_ext::BLOCK => {
                if let Some(block) = arena.get_block(node) {
                    for &stmt_idx in &block.statements.nodes {
                        self.collect_hoisted_file_scope_names(arena, stmt_idx, out);
                    }
                }
            }
            k if k == syntax_kind_ext::IF_STATEMENT => {
                if let Some(if_stmt) = arena.get_if_statement(node) {
                    self.collect_hoisted_file_scope_from_node(arena, if_stmt.then_statement, out);
                    if !if_stmt.else_statement.is_none() {
                        self.collect_hoisted_file_scope_from_node(arena, if_stmt.else_statement, out);
                    }
                }
            }
            k if k == syntax_kind_ext::WHILE_STATEMENT
                || k == syntax_kind_ext::DO_STATEMENT
                || k == syntax_kind_ext::FOR_STATEMENT =>
            {
                if let Some(loop_data) = arena.get_loop(node) {
                    self.collect_hoisted_file_scope_from_node(arena, loop_data.statement, out);
                }
            }
            _ => {}
        }
    }

    fn collect_hoisted_file_scope_from_node(
        &self,
        arena: &ThinNodeArena,
        idx: NodeIndex,
        out: &mut FxHashSet<String>,
    ) {
        if let Some(node) = arena.get(idx) {
            if node.kind == syntax_kind_ext::BLOCK {
                if let Some(block) = arena.get_block(node) {
                    for &stmt_idx in &block.statements.nodes {
                        self.collect_hoisted_file_scope_names(arena, stmt_idx, out);
                    }
                }
            }
        }
    }

    fn collect_variable_decl_names(
        &self,
        arena: &ThinNodeArena,
        decl_list_idx: NodeIndex,
        include_block_scoped: bool,
        out: &mut FxHashSet<String>,
    ) {
        let Some(node) = arena.get(decl_list_idx) else {
            return;
        };
        let Some(list) = arena.get_variable(node) else {
            return;
        };
        let is_var = (node.flags as u32 & (node_flags::LET | node_flags::CONST)) == 0;
        if !include_block_scoped && !is_var {
            return;
        }

        for &decl_idx in &list.declarations.nodes {
            if let Some(decl_node) = arena.get(decl_idx) {
                if let Some(decl) = arena.get_variable_declaration(decl_node) {
                    if let Some(name) = self.get_identifier_name(arena, decl.name) {
                        out.insert(name.to_string());
                    } else {
                        let mut names = Vec::new();
                        self.collect_binding_identifiers(arena, decl.name, &mut names);
                        for ident_idx in names {
                            if let Some(name) = self.get_identifier_name(arena, ident_idx) {
                                out.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_import_names(
        &self,
        arena: &ThinNodeArena,
        node: &ThinNode,
        out: &mut FxHashSet<String>,
    ) {
        if let Some(import) = arena.get_import_decl(node) {
            if let Some(clause_node) = arena.get(import.import_clause) {
                if let Some(clause) = arena.get_import_clause(clause_node) {
                    if !clause.name.is_none() {
                        if let Some(name) = self.get_identifier_name(arena, clause.name) {
                            out.insert(name.to_string());
                        }
                    }
                    if !clause.named_bindings.is_none() {
                        if let Some(bindings_node) = arena.get(clause.named_bindings) {
                            if bindings_node.kind == SyntaxKind::Identifier as u16 {
                                if let Some(name) = self.get_identifier_name(arena, clause.named_bindings) {
                                    out.insert(name.to_string());
                                }
                            } else if let Some(named) = arena.get_named_imports(bindings_node) {
                                for &spec_idx in &named.elements.nodes {
                                    if let Some(spec_node) = arena.get(spec_idx) {
                                        if let Some(spec) = arena.get_specifier(spec_node) {
                                            let local_ident = if !spec.name.is_none() {
                                                spec.name
                                            } else {
                                                spec.property_name
                                            };
                                            if let Some(name) = self.get_identifier_name(arena, local_ident) {
                                                out.insert(name.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_statement_symbol_nodes(
        &self,
        arena: &ThinNodeArena,
        statements: &[NodeIndex],
        out: &mut Vec<NodeIndex>,
    ) {
        for &stmt_idx in statements {
            let Some(node) = arena.get(stmt_idx) else {
                continue;
            };
            match node.kind {
                k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                    if let Some(var_stmt) = arena.get_variable(node) {
                        for &decl_list_idx in &var_stmt.declarations.nodes {
                            self.collect_variable_decl_symbol_nodes(arena, decl_list_idx, out);
                        }
                    }
                }
                k if k == syntax_kind_ext::FUNCTION_DECLARATION
                    || k == syntax_kind_ext::CLASS_DECLARATION
                    || k == syntax_kind_ext::INTERFACE_DECLARATION
                    || k == syntax_kind_ext::TYPE_ALIAS_DECLARATION
                    || k == syntax_kind_ext::ENUM_DECLARATION
                    || k == syntax_kind_ext::MODULE_DECLARATION =>
                {
                    out.push(stmt_idx);
                }
                k if k == syntax_kind_ext::IMPORT_DECLARATION => {
                    self.collect_import_symbol_nodes(arena, node, out);
                }
                k if k == syntax_kind_ext::IMPORT_EQUALS_DECLARATION => {
                    out.push(stmt_idx);
                }
                k if k == syntax_kind_ext::EXPORT_DECLARATION => {
                    self.collect_export_symbol_nodes(arena, node, out);
                }
                _ => {}
            }
        }
    }

    fn collect_variable_decl_symbol_nodes(
        &self,
        arena: &ThinNodeArena,
        decl_list_idx: NodeIndex,
        out: &mut Vec<NodeIndex>,
    ) {
        let Some(node) = arena.get(decl_list_idx) else {
            return;
        };
        let Some(list) = arena.get_variable(node) else {
            return;
        };

        for &decl_idx in &list.declarations.nodes {
            out.push(decl_idx);
            if let Some(decl_node) = arena.get(decl_idx) {
                if let Some(decl) = arena.get_variable_declaration(decl_node) {
                    if let Some(_name) = self.get_identifier_name(arena, decl.name) {
                        out.push(decl.name);
                    } else {
                        let mut names = Vec::new();
                        self.collect_binding_identifiers(arena, decl.name, &mut names);
                        out.extend(names);
                    }
                }
            }
        }
    }

    fn collect_import_symbol_nodes(
        &self,
        arena: &ThinNodeArena,
        node: &ThinNode,
        out: &mut Vec<NodeIndex>,
    ) {
        if let Some(import) = arena.get_import_decl(node) {
            if let Some(clause_node) = arena.get(import.import_clause) {
                if let Some(clause) = arena.get_import_clause(clause_node) {
                    if !clause.name.is_none() {
                        out.push(clause.name);
                    }
                    if !clause.named_bindings.is_none() {
                        if let Some(bindings_node) = arena.get(clause.named_bindings) {
                            if bindings_node.kind == SyntaxKind::Identifier as u16 {
                                out.push(clause.named_bindings);
                            } else if let Some(named) = arena.get_named_imports(bindings_node) {
                                for &spec_idx in &named.elements.nodes {
                                    out.push(spec_idx);
                                    if let Some(spec_node) = arena.get(spec_idx) {
                                        if let Some(spec) = arena.get_specifier(spec_node) {
                                            let local_ident = if !spec.name.is_none() {
                                                spec.name
                                            } else {
                                                spec.property_name
                                            };
                                            if !local_ident.is_none() {
                                                out.push(local_ident);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn collect_export_symbol_nodes(
        &self,
        arena: &ThinNodeArena,
        node: &ThinNode,
        out: &mut Vec<NodeIndex>,
    ) {
        if let Some(export) = arena.get_export_decl(node) {
            if export.export_clause.is_none() {
                return;
            }
            let Some(clause_node) = arena.get(export.export_clause) else {
                return;
            };
            if let Some(named) = arena.get_named_imports(clause_node) {
                for &spec_idx in &named.elements.nodes {
                    out.push(spec_idx);
                }
            } else if self.is_declaration(clause_node.kind) {
                self.collect_statement_symbol_nodes(arena, &[export.export_clause], out);
            } else if clause_node.kind == SyntaxKind::Identifier as u16 {
                out.push(export.export_clause);
            }
        }
    }

    /// Check if modifiers list contains the 'abstract' keyword.
    fn has_abstract_modifier(&self, arena: &ThinNodeArena, modifiers: &Option<NodeList>) -> bool {
        use crate::scanner::SyntaxKind;

        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::AbstractKeyword as u16 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if modifiers list contains the 'static' keyword.
    fn has_static_modifier(&self, arena: &ThinNodeArena, modifiers: &Option<NodeList>) -> bool {
        use crate::scanner::SyntaxKind;

        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::StaticKeyword as u16 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if modifiers list contains the 'export' keyword.
    fn has_export_modifier(&self, arena: &ThinNodeArena, modifiers: &Option<NodeList>) -> bool {
        use crate::scanner::SyntaxKind;

        if let Some(mods) = modifiers {
            for &mod_idx in &mods.nodes {
                if let Some(mod_node) = arena.get(mod_idx) {
                    if mod_node.kind == SyntaxKind::ExportKeyword as u16 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check if a node is exported.
    /// Handles walking up the tree for VariableDeclaration -> VariableStatement.
    fn is_node_exported(&self, arena: &ThinNodeArena, idx: NodeIndex) -> bool {
        let Some(node) = arena.get(idx) else { return false };

        // 1. Check direct modifiers (Function, Class, Interface, Enum, Module, TypeAlias)
        match node.kind {
            k if k == syntax_kind_ext::FUNCTION_DECLARATION => {
                if let Some(func) = arena.get_function(node) {
                    return self.has_export_modifier(arena, &func.modifiers);
                }
            }
            k if k == syntax_kind_ext::CLASS_DECLARATION => {
                if let Some(class) = arena.get_class(node) {
                    return self.has_export_modifier(arena, &class.modifiers);
                }
            }
            k if k == syntax_kind_ext::INTERFACE_DECLARATION => {
                if let Some(iface) = arena.get_interface(node) {
                    return self.has_export_modifier(arena, &iface.modifiers);
                }
            }
            k if k == syntax_kind_ext::TYPE_ALIAS_DECLARATION => {
                if let Some(alias) = arena.get_type_alias(node) {
                    return self.has_export_modifier(arena, &alias.modifiers);
                }
            }
            k if k == syntax_kind_ext::ENUM_DECLARATION => {
                if let Some(enum_decl) = arena.get_enum(node) {
                    return self.has_export_modifier(arena, &enum_decl.modifiers);
                }
            }
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                if let Some(module) = arena.get_module(node) {
                    return self.has_export_modifier(arena, &module.modifiers);
                }
            }
            // 2. Handle VariableDeclaration (walk up to VariableStatement)
            k if k == syntax_kind_ext::VARIABLE_DECLARATION => {
                // Walk up: VariableDeclaration -> VariableDeclarationList -> VariableStatement
                if let Some(ext) = arena.get_extended(idx) {
                    let list_idx = ext.parent;
                    if let Some(list_ext) = arena.get_extended(list_idx) {
                        let stmt_idx = list_ext.parent;
                        if let Some(stmt_node) = arena.get(stmt_idx) {
                            if stmt_node.kind == syntax_kind_ext::VARIABLE_STATEMENT {
                                if let Some(var_stmt) = arena.get_variable(stmt_node) {
                                    return self.has_export_modifier(arena, &var_stmt.modifiers);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        false
    }

    /// Declare a symbol in the current scope, merging when allowed.
    fn declare_symbol(&mut self, name: &str, flags: u32, declaration: NodeIndex, is_exported: bool) -> SymbolId {
        if let Some(existing_id) = self.current_scope.get(name) {
            let existing_flags = self.symbols.get(existing_id).map(|s| s.flags).unwrap_or(0);
            let can_merge = Self::can_merge_flags(existing_flags, flags);

            if let Some(sym) = self.symbols.get_mut(existing_id) {
                if can_merge {
                    sym.flags |= flags;
                    if sym.value_declaration.is_none() && (flags & symbol_flags::VALUE) != 0 {
                        sym.value_declaration = declaration;
                    }
                }

                if !sym.declarations.contains(&declaration) {
                    sym.declarations.push(declaration);
                }
                if is_exported {
                    sym.is_exported = true;
                }
            }

            self.node_symbols.insert(declaration.0, existing_id);
            return existing_id;
        }

        let sym_id = self.symbols.alloc(flags, name.to_string());
        if let Some(sym) = self.symbols.get_mut(sym_id) {
            sym.declarations.push(declaration);
            if sym.value_declaration.is_none() && (flags & symbol_flags::VALUE) != 0 {
                sym.value_declaration = declaration;
            }
            sym.is_exported = is_exported;
        }
        self.current_scope.set(name.to_string(), sym_id);
        self.node_symbols.insert(declaration.0, sym_id);
        sym_id
    }

    /// Check if two symbol flag sets can be merged.
    fn can_merge_flags(existing_flags: u32, new_flags: u32) -> bool {
        if (existing_flags & symbol_flags::INTERFACE) != 0
            && (new_flags & symbol_flags::INTERFACE) != 0
        {
            return true;
        }

        if (existing_flags & symbol_flags::MODULE) != 0
            && (new_flags & symbol_flags::MODULE) != 0
        {
            return true;
        }

        if (existing_flags & symbol_flags::MODULE) != 0 {
            if (new_flags & (symbol_flags::CLASS | symbol_flags::FUNCTION | symbol_flags::ENUM)) != 0 {
                return true;
            }
        }
        if (new_flags & symbol_flags::MODULE) != 0 {
            if (existing_flags & (symbol_flags::CLASS | symbol_flags::FUNCTION | symbol_flags::ENUM)) != 0 {
                return true;
            }
        }

        if (existing_flags & symbol_flags::FUNCTION) != 0
            && (new_flags & symbol_flags::FUNCTION) != 0
        {
            return true;
        }

        false
    }

    // Scope management

    fn enter_scope(&mut self, kind: ContainerKind, node: NodeIndex) {
        // Legacy scope chain management
        let parent = Some(self.current_scope_idx);
        self.scope_chain.push(ScopeContext::new(kind, node, parent));
        self.current_scope_idx = self.scope_chain.len() - 1;
        self.push_scope();

        // Persistent scope management (for stateless checking)
        self.enter_persistent_scope(kind, node);
    }

    fn exit_scope(&mut self) {
        // Capture exports before popping if this is a module/namespace
        if let Some(ctx) = self.scope_chain.get(self.current_scope_idx) {
            match ctx.container_kind {
                ContainerKind::Module => {
                    // Find the symbol for this module/namespace
                    if let Some(sym_id) = self.node_symbols.get(&ctx.container_node.0) {
                        // Filter exports: only include symbols with is_exported = true or EXPORT_VALUE flag
                        let mut exports = SymbolTable::new();
                        for (name, &child_id) in self.current_scope.iter() {
                            if let Some(child) = self.symbols.get(child_id) {
                                // Check explicit export flag OR if it's an EXPORT_VALUE (from export {})
                                if child.is_exported || (child.flags & symbol_flags::EXPORT_VALUE) != 0 {
                                    exports.set(name.clone(), child_id);
                                }
                            }
                        }

                        // Persist filtered exports
                        if let Some(symbol) = self.symbols.get_mut(*sym_id) {
                            if let Some(ref mut existing) = symbol.exports {
                                for (name, &child_id) in exports.iter() {
                                    existing.set(name.clone(), child_id);
                                }
                            } else {
                                symbol.exports = Some(Box::new(exports));
                            }
                        }
                    }
                }
                ContainerKind::Class => {
                    // Find the symbol for this class
                    if let Some(sym_id) = self.node_symbols.get(&ctx.container_node.0) {
                        // Persist the current scope as the class's members
                        if let Some(symbol) = self.symbols.get_mut(*sym_id) {
                            symbol.members = Some(Box::new(self.current_scope.clone()));
                        }
                    }
                }
                _ => {}
            }
        }

        // Copy current scope to persistent scope before popping
        self.sync_current_scope_to_persistent();

        self.pop_scope();
        if let Some(ctx) = self.scope_chain.get(self.current_scope_idx) {
            if let Some(parent) = ctx.parent_idx {
                self.current_scope_idx = parent;
            }
        }

        // Exit persistent scope
        self.exit_persistent_scope();
    }

    fn push_scope(&mut self) {
        let old_scope = std::mem::take(&mut self.current_scope);
        self.scope_stack.push(old_scope);
        self.current_scope = SymbolTable::new();
    }

    fn pop_scope(&mut self) {
        if let Some(parent_scope) = self.scope_stack.pop() {
            self.current_scope = parent_scope;
        }
    }

    // Declaration binding methods

    fn bind_variable_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(decl) = arena.get_variable_declaration(node) {
            let mut decl_flags = node.flags as u32;
            if (decl_flags & (node_flags::LET | node_flags::CONST)) == 0 {
                if let Some(ext) = arena.get_extended(idx) {
                    if let Some(parent_node) = arena.get(ext.parent) {
                        if parent_node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
                            decl_flags |= parent_node.flags as u32;
                        }
                    }
                }
            }
            let is_block_scoped = (decl_flags & (node_flags::LET | node_flags::CONST)) != 0;
            if let Some(name) = self.get_identifier_name(arena, decl.name) {
                // Determine if block-scoped (let/const) or function-scoped (var)
                let flags = if is_block_scoped {
                    symbol_flags::BLOCK_SCOPED_VARIABLE
                } else {
                    symbol_flags::FUNCTION_SCOPED_VARIABLE
                };

                // Check if exported BEFORE allocating symbol
                let is_exported = self.is_node_exported(arena, idx);

                let sym_id = self.declare_symbol(name, flags, idx, is_exported);
                self.node_symbols.insert(decl.name.0, sym_id);
            } else {
                let flags = if is_block_scoped {
                    symbol_flags::BLOCK_SCOPED_VARIABLE
                } else {
                    symbol_flags::FUNCTION_SCOPED_VARIABLE
                };
                let is_exported = self.is_node_exported(arena, idx);

                let mut names = Vec::new();
                self.collect_binding_identifiers(arena, decl.name, &mut names);
                for ident_idx in names {
                    if let Some(name) = self.get_identifier_name(arena, ident_idx) {
                        self.declare_symbol(name, flags, ident_idx, is_exported);
                    }
                }
            }

            if !decl.initializer.is_none() {
                self.bind_node(arena, decl.initializer);
            }
        }
    }

    fn bind_function_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(func) = arena.get_function(node) {
            self.bind_modifiers(arena, &func.modifiers);
            // Function declaration creates a symbol in the current scope
            if let Some(name) = self.get_identifier_name(arena, func.name) {
                let is_exported = self.has_export_modifier(arena, &func.modifiers);
                self.declare_symbol(name, symbol_flags::FUNCTION, idx, is_exported);
            }

            // Enter function scope and bind body
            self.enter_scope(ContainerKind::Function, idx);

            self.with_fresh_flow(|binder| {
                // Bind parameters
                for &param_idx in &func.parameters.nodes {
                    binder.bind_parameter(arena, param_idx);
                }

                // Bind body
                binder.bind_node(arena, func.body);
            });

            self.exit_scope();
        }
    }

    fn bind_parameter(&mut self, arena: &ThinNodeArena, idx: NodeIndex) {
        if let Some(node) = arena.get(idx) {
            if let Some(param) = arena.get_parameter(node) {
                self.bind_modifiers(arena, &param.modifiers);
                if let Some(name) = self.get_identifier_name(arena, param.name) {
                    let sym_id = self.declare_symbol(name, symbol_flags::FUNCTION_SCOPED_VARIABLE, idx, false);
                    self.node_symbols.insert(param.name.0, sym_id);
                } else {
                    let mut names = Vec::new();
                    self.collect_binding_identifiers(arena, param.name, &mut names);
                    for ident_idx in names {
                        if let Some(name) = self.get_identifier_name(arena, ident_idx) {
                            self.declare_symbol(name, symbol_flags::FUNCTION_SCOPED_VARIABLE, ident_idx, false);
                        }
                    }
                }

                if !param.initializer.is_none() {
                    self.bind_node(arena, param.initializer);
                }
            }
        }
    }

    /// Bind an arrow function expression - creates a scope and binds the body.
    fn bind_arrow_function(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(func) = arena.get_function(node) {
            self.bind_modifiers(arena, &func.modifiers);
            // Enter function scope
            self.enter_scope(ContainerKind::Function, idx);

            self.with_fresh_flow(|binder| {
                // Bind parameters
                for &param_idx in &func.parameters.nodes {
                    binder.bind_parameter(arena, param_idx);
                }

                // Bind body (could be a block or an expression)
                binder.bind_node(arena, func.body);
            });

            self.exit_scope();
        }
    }

    /// Bind a function expression - creates a scope and binds the body.
    fn bind_function_expression(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(func) = arena.get_function(node) {
            self.bind_modifiers(arena, &func.modifiers);
            // Enter function scope
            self.enter_scope(ContainerKind::Function, idx);

            self.with_fresh_flow(|binder| {
                // Bind parameters
                for &param_idx in &func.parameters.nodes {
                    binder.bind_parameter(arena, param_idx);
                }

                // Bind body
                binder.bind_node(arena, func.body);
            });

            self.exit_scope();
        }
    }

    fn bind_callable_body(
        &mut self,
        arena: &ThinNodeArena,
        parameters: &NodeList,
        body: NodeIndex,
        idx: NodeIndex,
    ) {
        self.enter_scope(ContainerKind::Function, idx);

        self.with_fresh_flow(|binder| {
            for &param_idx in &parameters.nodes {
                binder.bind_parameter(arena, param_idx);
            }

            if !body.is_none() {
                binder.bind_node(arena, body);
            }
        });

        self.exit_scope();
    }

    fn bind_modifiers(&mut self, arena: &ThinNodeArena, modifiers: &Option<NodeList>) {
        if let Some(list) = modifiers {
            for &modifier_idx in &list.nodes {
                self.bind_node(arena, modifier_idx);
            }
        }
    }

    fn bind_class_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(class) = arena.get_class(node) {
            self.bind_modifiers(arena, &class.modifiers);
            if let Some(name) = self.get_identifier_name(arena, class.name) {
                // Start with CLASS flag
                let mut flags = symbol_flags::CLASS;

                // Add ABSTRACT flag if class has 'abstract' modifier
                if self.has_abstract_modifier(arena, &class.modifiers) {
                    flags |= symbol_flags::ABSTRACT;
                }

                // Check if exported BEFORE allocating symbol
                let is_exported = self.has_export_modifier(arena, &class.modifiers);

                self.declare_symbol(name, flags, idx, is_exported);
            }

            // Enter class scope for members
            self.enter_scope(ContainerKind::Class, idx);

            for &member_idx in &class.members.nodes {
                self.bind_class_member(arena, member_idx);
            }

            self.exit_scope();
        }
    }

    fn bind_class_expression(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(class) = arena.get_class(node) {
            self.bind_modifiers(arena, &class.modifiers);
            self.enter_scope(ContainerKind::Class, idx);

            if let Some(name) = self.get_identifier_name(arena, class.name) {
                let mut flags = symbol_flags::CLASS;
                if self.has_abstract_modifier(arena, &class.modifiers) {
                    flags |= symbol_flags::ABSTRACT;
                }
                let sym_id = self.declare_symbol(name, flags, idx, false);
                self.node_symbols.insert(class.name.0, sym_id);
            }

            for &member_idx in &class.members.nodes {
                self.bind_class_member(arena, member_idx);
            }

            self.exit_scope();
        }
    }

    fn bind_class_member(&mut self, arena: &ThinNodeArena, idx: NodeIndex) {
        if let Some(node) = arena.get(idx) {
            match node.kind {
                k if k == syntax_kind_ext::METHOD_DECLARATION => {
                    if let Some(method) = arena.get_method_decl(node) {
                        self.bind_modifiers(arena, &method.modifiers);
                        if let Some(name) = self.get_identifier_name(arena, method.name) {
                            let mut flags = symbol_flags::METHOD;
                            if self.has_abstract_modifier(arena, &method.modifiers) {
                                flags |= symbol_flags::ABSTRACT;
                            }
                            if self.has_static_modifier(arena, &method.modifiers) {
                                flags |= symbol_flags::STATIC;
                            }
                            let sym_id = self.declare_symbol(name, flags, idx, false);
                            self.node_symbols.insert(method.name.0, sym_id);
                        }
                        self.bind_callable_body(arena, &method.parameters, method.body, idx);
                    }
                }
                k if k == syntax_kind_ext::PROPERTY_DECLARATION => {
                    if let Some(prop) = arena.get_property_decl(node) {
                        self.bind_modifiers(arena, &prop.modifiers);
                        if let Some(name) = self.get_identifier_name(arena, prop.name) {
                            let mut flags = symbol_flags::PROPERTY;
                            if self.has_abstract_modifier(arena, &prop.modifiers) {
                                flags |= symbol_flags::ABSTRACT;
                            }
                            if self.has_static_modifier(arena, &prop.modifiers) {
                                flags |= symbol_flags::STATIC;
                            }
                            let sym_id = self.declare_symbol(name, flags, idx, false);
                            self.node_symbols.insert(prop.name.0, sym_id);
                        }

                        if !prop.initializer.is_none() {
                            self.bind_node(arena, prop.initializer);
                        }
                    }
                }
                k if k == syntax_kind_ext::GET_ACCESSOR || k == syntax_kind_ext::SET_ACCESSOR => {
                    if let Some(accessor) = arena.get_accessor(node) {
                        self.bind_modifiers(arena, &accessor.modifiers);
                        if let Some(name) = self.get_identifier_name(arena, accessor.name) {
                            let mut flags = if node.kind == syntax_kind_ext::GET_ACCESSOR {
                                symbol_flags::GET_ACCESSOR
                            } else {
                                symbol_flags::SET_ACCESSOR
                            };
                            if self.has_abstract_modifier(arena, &accessor.modifiers) {
                                flags |= symbol_flags::ABSTRACT;
                            }
                            if self.has_static_modifier(arena, &accessor.modifiers) {
                                flags |= symbol_flags::STATIC;
                            }
                            let sym_id = self.declare_symbol(name, flags, idx, false);
                            self.node_symbols.insert(accessor.name.0, sym_id);
                        }
                        self.bind_callable_body(arena, &accessor.parameters, accessor.body, idx);
                    }
                }
                k if k == syntax_kind_ext::CONSTRUCTOR => {
                    self.declare_symbol("constructor", symbol_flags::CONSTRUCTOR, idx, false);
                    if let Some(ctor) = arena.get_constructor(node) {
                        self.bind_modifiers(arena, &ctor.modifiers);
                        self.bind_callable_body(arena, &ctor.parameters, ctor.body, idx);
                    }
                }
                k if k == syntax_kind_ext::CLASS_STATIC_BLOCK_DECLARATION => {
                    if let Some(block) = arena.get_block(node) {
                        self.enter_scope(ContainerKind::Block, idx);
                        for &stmt_idx in &block.statements.nodes {
                            self.bind_node(arena, stmt_idx);
                        }
                        self.exit_scope();
                    }
                }
                _ => {}
            }
        }
    }

    fn bind_interface_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(iface) = arena.get_interface(node) {
            if let Some(name) = self.get_identifier_name(arena, iface.name) {
                // Check if exported BEFORE allocating symbol
                let is_exported = self.has_export_modifier(arena, &iface.modifiers);

                self.declare_symbol(name, symbol_flags::INTERFACE, idx, is_exported);
            }
        }
    }

    fn bind_type_alias_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(alias) = arena.get_type_alias(node) {
            if let Some(name) = self.get_identifier_name(arena, alias.name) {
                // Check if exported BEFORE allocating symbol
                let is_exported = self.has_export_modifier(arena, &alias.modifiers);

                self.declare_symbol(name, symbol_flags::TYPE_ALIAS, idx, is_exported);
            }
        }
    }

    fn bind_enum_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(enum_decl) = arena.get_enum(node) {
            if let Some(name) = self.get_identifier_name(arena, enum_decl.name) {
                // Check if exported BEFORE allocating symbol
                let is_exported = self.has_export_modifier(arena, &enum_decl.modifiers);

                self.declare_symbol(name, symbol_flags::REGULAR_ENUM, idx, is_exported);
            }

            // Bind enum members
            self.enter_scope(ContainerKind::Block, idx);
            for &member_idx in &enum_decl.members.nodes {
                if let Some(member_node) = arena.get(member_idx) {
                    if let Some(member) = arena.get_enum_member(member_node) {
                        if let Some(member_name) = self.get_identifier_name(arena, member.name) {
                            let sym_id = self.symbols.alloc(symbol_flags::ENUM_MEMBER, member_name.to_string());
                            self.current_scope.set(member_name.to_string(), sym_id);
                            self.node_symbols.insert(member_idx.0, sym_id);
                        }
                    }
                }
            }
            self.exit_scope();
        }
    }

    fn bind_switch_statement(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(switch_data) = arena.get_switch(node) {
            self.bind_expression(arena, switch_data.expression);

            let pre_switch_flow = self.current_flow;
            let end_label = self.create_branch_label();
            let mut fallthrough_flow = FlowNodeId::NONE;

            // Case block contains case clauses
            if let Some(case_block_node) = arena.get(switch_data.case_block) {
                if let Some(case_block) = arena.get_block(case_block_node) {
                    for &clause_idx in &case_block.statements.nodes {
                        if let Some(clause_node) = arena.get(clause_idx) {
                            if let Some(clause) = arena.get_case_clause(clause_node) {
                                self.switch_clause_to_switch.insert(clause_idx.0, idx);

                                self.current_flow = pre_switch_flow;
                                if !clause.expression.is_none() {
                                    self.bind_expression(arena, clause.expression);
                                }

                                let clause_flow = self.create_switch_clause_flow(
                                    pre_switch_flow,
                                    fallthrough_flow,
                                    clause_idx,
                                );
                                self.current_flow = clause_flow;

                                for &stmt_idx in &clause.statements.nodes {
                                    self.bind_node(arena, stmt_idx);
                                }

                                self.add_antecedent(end_label, self.current_flow);

                                if self.clause_allows_fallthrough(arena, clause) {
                                    fallthrough_flow = self.current_flow;
                                } else {
                                    fallthrough_flow = FlowNodeId::NONE;
                                }
                            }
                        }
                    }
                }
            }

            self.current_flow = end_label;
        }
    }

    fn clause_allows_fallthrough(
        &self,
        arena: &ThinNodeArena,
        clause: &crate::parser::thin_node::CaseClauseData,
    ) -> bool {
        let Some(&last_stmt_idx) = clause.statements.nodes.last() else {
            return true;
        };

        let Some(stmt_node) = arena.get(last_stmt_idx) else {
            return true;
        };

        match stmt_node.kind {
            k if k == syntax_kind_ext::BREAK_STATEMENT
                || k == syntax_kind_ext::RETURN_STATEMENT
                || k == syntax_kind_ext::THROW_STATEMENT
                || k == syntax_kind_ext::CONTINUE_STATEMENT =>
            {
                false
            }
            _ => true,
        }
    }

    fn bind_try_statement(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(try_data) = arena.get_try(node) {
            // Bind try block
            self.bind_node(arena, try_data.try_block);

            // Bind catch clause
            if !try_data.catch_clause.is_none() {
                if let Some(catch_node) = arena.get(try_data.catch_clause) {
                    if let Some(catch) = arena.get_catch_clause(catch_node) {
                        self.enter_scope(ContainerKind::Block, idx);

                        // Bind catch variable
                        if !catch.variable_declaration.is_none() {
                            self.bind_node(arena, catch.variable_declaration);
                        }

                        // Bind catch block
                        self.bind_node(arena, catch.block);

                        self.exit_scope();
                    }
                }
            }

            // Bind finally block
            if !try_data.finally_block.is_none() {
                self.bind_node(arena, try_data.finally_block);
            }
        }
    }

    fn bind_import_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, _idx: NodeIndex) {
        if let Some(import) = arena.get_import_decl(node) {
            if let Some(clause_node) = arena.get(import.import_clause) {
                if let Some(clause) = arena.get_import_clause(clause_node) {
                    // Default import
                    if !clause.name.is_none() {
                        if let Some(name) = self.get_identifier_name(arena, clause.name) {
                            let sym_id = self.symbols.alloc(symbol_flags::ALIAS, name.to_string());
                            if let Some(sym) = self.symbols.get_mut(sym_id) {
                                sym.declarations.push(clause.name);
                            }
                            self.current_scope.set(name.to_string(), sym_id);
                            self.node_symbols.insert(clause.name.0, sym_id);
                        }
                    }

                    // Named imports
                    if !clause.named_bindings.is_none() {
                        if let Some(bindings_node) = arena.get(clause.named_bindings) {
                            if bindings_node.kind == SyntaxKind::Identifier as u16 {
                                if let Some(name) = self.get_identifier_name(arena, clause.named_bindings) {
                                    let sym_id = self.symbols.alloc(symbol_flags::ALIAS, name.to_string());
                                    if let Some(sym) = self.symbols.get_mut(sym_id) {
                                        sym.declarations.push(clause.named_bindings);
                                    }
                                    self.current_scope.set(name.to_string(), sym_id);
                                    self.node_symbols.insert(clause.named_bindings.0, sym_id);
                                }
                            } else if let Some(named) = arena.get_named_imports(bindings_node) {
                                for &spec_idx in &named.elements.nodes {
                                    if let Some(spec_node) = arena.get(spec_idx) {
                                        if let Some(spec) = arena.get_specifier(spec_node) {
                                            let local_ident = if !spec.name.is_none() {
                                                spec.name
                                            } else {
                                                spec.property_name
                                            };
                                            let local_name = self.get_identifier_name(arena, local_ident);

                                            if let Some(name) = local_name {
                                                let sym_id = self.symbols.alloc(symbol_flags::ALIAS, name.to_string());
                                                if let Some(sym) = self.symbols.get_mut(sym_id) {
                                                    sym.declarations.push(local_ident);
                                                }
                                                self.current_scope.set(name.to_string(), sym_id);
                                                self.node_symbols.insert(spec_idx.0, sym_id);
                                                self.node_symbols.insert(local_ident.0, sym_id);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Bind import equals declaration: import x = ns.member or import x = require("...")
    fn bind_import_equals_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(import) = arena.get_import_decl(node) {
            // import_clause holds the alias name (e.g., 'x' in 'import x = ...')
            if let Some(name) = self.get_identifier_name(arena, import.import_clause) {
                // Check if exported (for export import x = ns.member)
                let is_exported = self.has_export_modifier(arena, &import.modifiers);

                // Create symbol with ALIAS flag
                let sym_id = self.symbols.alloc(symbol_flags::ALIAS, name.to_string());

                if let Some(sym) = self.symbols.get_mut(sym_id) {
                    sym.declarations.push(idx);
                    sym.value_declaration = idx;
                    sym.is_exported = is_exported;
                }

                self.current_scope.set(name.to_string(), sym_id);
                self.node_symbols.insert(idx.0, sym_id);
            }
        }
    }

    /// Mark symbols associated with a declaration node as exported.
    /// This is required because the parser wraps exported declarations in ExportDeclaration
    /// nodes instead of attaching modifiers to the declaration itself.
    fn mark_exported_symbols(&mut self, arena: &ThinNodeArena, idx: NodeIndex) {
        // 1. Try direct symbol lookup (Function, Class, Enum, Module, Interface, TypeAlias)
        if let Some(sym_id) = self.node_symbols.get(&idx.0) {
            if let Some(sym) = self.symbols.get_mut(*sym_id) {
                sym.is_exported = true;
            }
            return;
        }

        // 2. Handle VariableStatement -> VariableDeclarationList -> VariableDeclaration
        // Variable statements don't have a symbol; their declarations do.
        if let Some(node) = arena.get(idx) {
            if node.kind == syntax_kind_ext::VARIABLE_STATEMENT {
                if let Some(var) = arena.get_variable(node) {
                    for &list_idx in &var.declarations.nodes {
                        if let Some(list_node) = arena.get(list_idx) {
                            if let Some(list) = arena.get_variable(list_node) {
                                for &decl_idx in &list.declarations.nodes {
                                    if let Some(sym_id) = self.node_symbols.get(&decl_idx.0) {
                                        if let Some(sym) = self.symbols.get_mut(*sym_id) {
                                            sym.is_exported = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn bind_export_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, _idx: NodeIndex) {
        if let Some(export) = arena.get_export_decl(node) {
            // Export clause can be:
            // - NamedExports: export { foo, bar }
            // - NamespaceExport: export * as ns from 'mod'
            // - Declaration: export function/class/const/etc
            // - or NONE for: export * from 'mod'

            if !export.export_clause.is_none() {
                if let Some(clause_node) = arena.get(export.export_clause) {
                    // Check if it's named exports { foo, bar }
                    if let Some(named) = arena.get_named_imports(clause_node) {
                        // Bind each export specifier as an EXPORT_VALUE
                        for &spec_idx in &named.elements.nodes {
                            if let Some(spec_node) = arena.get(spec_idx) {
                                if let Some(spec) = arena.get_specifier(spec_node) {
                                    // For export { foo }, property_name is NONE, name is "foo"
                                    // For export { foo as bar }, property_name is "foo", name is "bar"
                                    let exported_name = if !spec.name.is_none() {
                                        self.get_identifier_name(arena, spec.name)
                                    } else {
                                        self.get_identifier_name(arena, spec.property_name)
                                    };

                                    if let Some(name) = exported_name {
                                        // Create export symbol (EXPORT_VALUE for value exports)
                                        // This marks the name as exported from this module
                                        let sym_id = self.symbols.alloc(symbol_flags::EXPORT_VALUE, name.to_string());
                                        self.node_symbols.insert(spec_idx.0, sym_id);
                                    }
                                }
                            }
                        }
                    }
                    // Check if it's an exported declaration (function, class, variable, etc.)
                    else if self.is_declaration(clause_node.kind) {
                        // Recursively bind the declaration
                        // This handles: export function foo() {}, export class Bar {}, export const x = 1
                        self.bind_node(arena, export.export_clause);

                        // FIX: Explicitly mark the bound symbol(s) as exported
                        // because the inner declaration node lacks the 'export' modifier
                        self.mark_exported_symbols(arena, export.export_clause);
                    }
                    // Namespace export: export * as ns from 'mod'
                    else if let Some(name) = self.get_identifier_name(arena, export.export_clause) {
                        let sym_id = self.symbols.alloc(symbol_flags::ALIAS, name.to_string());
                        self.current_scope.set(name.to_string(), sym_id);
                        self.node_symbols.insert(export.export_clause.0, sym_id);
                    } else if export.is_default_export {
                        // export default <expression> should still bind inner locals.
                        self.bind_node(arena, export.export_clause);
                    }
                }
            }
            // export * from 'mod' - no binding needed, just re-exports
        }
    }

    /// Check if a node kind is a declaration that should be bound
    fn is_declaration(&self, kind: u16) -> bool {
        kind == syntax_kind_ext::FUNCTION_DECLARATION ||
        kind == syntax_kind_ext::CLASS_DECLARATION ||
        kind == syntax_kind_ext::VARIABLE_STATEMENT ||
        kind == syntax_kind_ext::INTERFACE_DECLARATION ||
        kind == syntax_kind_ext::TYPE_ALIAS_DECLARATION ||
        kind == syntax_kind_ext::ENUM_DECLARATION ||
        kind == syntax_kind_ext::MODULE_DECLARATION
    }

    fn bind_module_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(module) = arena.get_module(node) {
            if let Some(name) = self.get_identifier_name(arena, module.name) {
                let is_exported = self.has_export_modifier(arena, &module.modifiers);
                let flags = symbol_flags::VALUE_MODULE | symbol_flags::NAMESPACE_MODULE;
                self.declare_symbol(name, flags, idx, is_exported);
            }

            // Enter module scope
            self.enter_scope(ContainerKind::Module, idx);
            self.bind_node(arena, module.body);
            self.exit_scope();
        }
    }

    // Public accessors

    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id)
    }

    pub fn get_node_symbol(&self, node: NodeIndex) -> Option<SymbolId> {
        self.node_symbols.get(&node.0).copied()
    }

    pub fn get_symbols(&self) -> &SymbolArena {
        &self.symbols
    }

    /// Get the flow node that was active at a given AST node.
    /// Used by the checker for control flow analysis.
    pub fn get_node_flow(&self, node: NodeIndex) -> Option<FlowNodeId> {
        self.node_flow.get(&node.0).copied()
    }

    /// Get the containing switch statement for a case/default clause.
    pub fn get_switch_for_clause(&self, clause: NodeIndex) -> Option<NodeIndex> {
        self.switch_clause_to_switch.get(&clause.0).copied()
    }

    /// Record the current flow node for an AST node.
    /// Called during binding to track flow position for identifiers and other expressions.
    fn record_flow(&mut self, node: NodeIndex) {
        if !self.current_flow.is_none() {
            self.node_flow.insert(node.0, self.current_flow);
        }
    }

    fn with_fresh_flow<F>(&mut self, bind_body: F)
    where
        F: FnOnce(&mut Self),
    {
        let prev_flow = self.current_flow;
        let start_flow = self.flow_nodes.alloc(flow_flags::START);
        self.current_flow = start_flow;
        bind_body(self);
        self.current_flow = prev_flow;
    }

    // =========================================================================
    // Flow graph construction helpers
    // =========================================================================

    /// Create a branch label flow node for merging control flow paths.
    fn create_branch_label(&mut self) -> FlowNodeId {
        self.flow_nodes.alloc(flow_flags::BRANCH_LABEL)
    }

    /// Create a loop label flow node for back-edges.
    fn create_loop_label(&mut self) -> FlowNodeId {
        self.flow_nodes.alloc(flow_flags::LOOP_LABEL)
    }

    /// Create a flow condition node for tracking type narrowing.
    fn create_flow_condition(&mut self, flags: u32, antecedent: FlowNodeId, condition: NodeIndex) -> FlowNodeId {
        let id = self.flow_nodes.alloc(flags);
        if let Some(node) = self.flow_nodes.get_mut(id) {
            node.antecedent.push(antecedent);
            node.node = condition;
        }
        id
    }

    /// Create a flow node for a switch clause with optional fallthrough.
    fn create_switch_clause_flow(
        &mut self,
        pre_switch: FlowNodeId,
        fallthrough: FlowNodeId,
        clause: NodeIndex,
    ) -> FlowNodeId {
        let id = self.flow_nodes.alloc(flow_flags::SWITCH_CLAUSE);
        if let Some(node) = self.flow_nodes.get_mut(id) {
            node.node = clause;
        }
        self.add_antecedent(id, pre_switch);
        self.add_antecedent(id, fallthrough);
        id
    }

    /// Create a flow node for an assignment.
    fn create_flow_assignment(&mut self, assignment: NodeIndex) -> FlowNodeId {
        let id = self.flow_nodes.alloc(flow_flags::ASSIGNMENT);
        if let Some(node) = self.flow_nodes.get_mut(id) {
            node.node = assignment;
            if !self.current_flow.is_none() {
                node.antecedent.push(self.current_flow);
            }
        }
        id
    }

    /// Create a flow node for array mutation (e.g. push/splice).
    fn create_flow_array_mutation(&mut self, call: NodeIndex) -> FlowNodeId {
        let id = self.flow_nodes.alloc(flow_flags::ARRAY_MUTATION);
        if let Some(node) = self.flow_nodes.get_mut(id) {
            node.node = call;
            if !self.current_flow.is_none() {
                node.antecedent.push(self.current_flow);
            }
        }
        id
    }

    /// Add an antecedent to a flow node (for merging branches).
    fn add_antecedent(&mut self, label: FlowNodeId, antecedent: FlowNodeId) {
        if antecedent.is_none() || antecedent == self.unreachable_flow {
            return;
        }
        if let Some(node) = self.flow_nodes.get_mut(label) {
            if !node.antecedent.contains(&antecedent) {
                node.antecedent.push(antecedent);
            }
        }
    }

    // =========================================================================
    // Expression binding for flow analysis
    // =========================================================================

    fn is_assignment_operator(&self, operator: u16) -> bool {
        matches!(
            operator,
            k if k == SyntaxKind::EqualsToken as u16
                || k == SyntaxKind::PlusEqualsToken as u16
                || k == SyntaxKind::MinusEqualsToken as u16
                || k == SyntaxKind::AsteriskEqualsToken as u16
                || k == SyntaxKind::AsteriskAsteriskEqualsToken as u16
                || k == SyntaxKind::SlashEqualsToken as u16
                || k == SyntaxKind::PercentEqualsToken as u16
                || k == SyntaxKind::LessThanLessThanEqualsToken as u16
                || k == SyntaxKind::GreaterThanGreaterThanEqualsToken as u16
                || k == SyntaxKind::GreaterThanGreaterThanGreaterThanEqualsToken as u16
                || k == SyntaxKind::AmpersandEqualsToken as u16
                || k == SyntaxKind::BarEqualsToken as u16
                || k == SyntaxKind::BarBarEqualsToken as u16
                || k == SyntaxKind::AmpersandAmpersandEqualsToken as u16
                || k == SyntaxKind::QuestionQuestionEqualsToken as u16
                || k == SyntaxKind::CaretEqualsToken as u16
        )
    }

    fn is_array_mutation_call(&self, arena: &ThinNodeArena, call_idx: NodeIndex) -> bool {
        let Some(call_node) = arena.get(call_idx) else {
            return false;
        };
        let Some(call) = arena.get_call_expr(call_node) else {
            return false;
        };
        let Some(callee_node) = arena.get(call.expression) else {
            return false;
        };
        let Some(access) = arena.get_access_expr(callee_node) else {
            return false;
        };
        if access.question_dot_token {
            return false;
        }
        let Some(name_node) = arena.get(access.name_or_argument) else {
            return false;
        };
        let name = if let Some(ident) = arena.get_identifier(name_node) {
            ident.escaped_text.as_str()
        } else if let Some(literal) = arena.get_literal(name_node) {
            if name_node.kind == SyntaxKind::StringLiteral as u16 {
                literal.text.as_str()
            } else {
                return false;
            }
        } else {
            return false;
        };

        matches!(
            name,
            "copyWithin"
                | "fill"
                | "pop"
                | "push"
                | "reverse"
                | "shift"
                | "sort"
                | "splice"
                | "unshift"
        )
    }

    // Avoid deep recursion on large left-associative binary expression chains.
    fn bind_binary_expression_iterative(&mut self, arena: &ThinNodeArena, root: NodeIndex) {
        enum WorkItem {
            Visit(NodeIndex),
            PostAssign(NodeIndex),
        }

        let mut stack = vec![WorkItem::Visit(root)];
        while let Some(item) = stack.pop() {
            match item {
                WorkItem::Visit(idx) => {
                    let node = match arena.get(idx) {
                        Some(n) => n,
                        None => continue,
                    };

                    if node.kind == syntax_kind_ext::BINARY_EXPRESSION {
                        if let Some(bin) = arena.get_binary_expr(node) {
                            if self.is_assignment_operator(bin.operator_token) {
                                stack.push(WorkItem::PostAssign(idx));
                                if !bin.right.is_none() {
                                    stack.push(WorkItem::Visit(bin.right));
                                }
                                if !bin.left.is_none() {
                                    stack.push(WorkItem::Visit(bin.left));
                                }
                                continue;
                            }
                            if !bin.right.is_none() {
                                stack.push(WorkItem::Visit(bin.right));
                            }
                            if !bin.left.is_none() {
                                stack.push(WorkItem::Visit(bin.left));
                            }
                        }
                        continue;
                    }

                    self.bind_node(arena, idx);
                }
                WorkItem::PostAssign(idx) => {
                    let flow = self.create_flow_assignment(idx);
                    self.current_flow = flow;
                }
            }
        }
    }

    fn bind_binary_expression_flow_iterative(&mut self, arena: &ThinNodeArena, root: NodeIndex) {
        enum WorkItem {
            Visit(NodeIndex),
            PostAssign(NodeIndex),
        }

        let mut stack = vec![WorkItem::Visit(root)];
        while let Some(item) = stack.pop() {
            match item {
                WorkItem::Visit(idx) => {
                    let node = match arena.get(idx) {
                        Some(n) => n,
                        None => continue,
                    };

                    if node.kind == syntax_kind_ext::BINARY_EXPRESSION {
                        self.record_flow(idx);
                        if let Some(bin) = arena.get_binary_expr(node) {
                            if self.is_assignment_operator(bin.operator_token) {
                                stack.push(WorkItem::PostAssign(idx));
                                if !bin.right.is_none() {
                                    stack.push(WorkItem::Visit(bin.right));
                                }
                                if !bin.left.is_none() {
                                    stack.push(WorkItem::Visit(bin.left));
                                }
                                continue;
                            }
                            if !bin.right.is_none() {
                                stack.push(WorkItem::Visit(bin.right));
                            }
                            if !bin.left.is_none() {
                                stack.push(WorkItem::Visit(bin.left));
                            }
                        }
                        continue;
                    }

                    self.bind_expression(arena, idx);
                }
                WorkItem::PostAssign(idx) => {
                    let flow = self.create_flow_assignment(idx);
                    self.current_flow = flow;
                }
            }
        }
    }

    /// Bind an expression and record flow positions for identifiers.
    /// This is used for condition expressions in if/while/for statements.
    fn bind_expression(&mut self, arena: &ThinNodeArena, idx: NodeIndex) {
        if idx.is_none() {
            return;
        }

        let node = match arena.get(idx) {
            Some(n) => n,
            None => return,
        };

        if node.kind == syntax_kind_ext::BINARY_EXPRESSION {
            if let Some(bin) = arena.get_binary_expr(node) {
                if self.is_assignment_operator(bin.operator_token) {
                    self.record_flow(idx);
                    self.bind_expression(arena, bin.left);
                    self.bind_expression(arena, bin.right);
                    let flow = self.create_flow_assignment(idx);
                    self.current_flow = flow;
                    return;
                }
            }
            self.bind_binary_expression_flow_iterative(arena, idx);
            return;
        }

        // Record flow position for this node
        self.record_flow(idx);

        match node.kind {
            // Identifiers - record flow position for type narrowing
            k if k == SyntaxKind::Identifier as u16 => {
                // Already recorded above
                return;
            }

            // Prefix unary (e.g., typeof x, !x)
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION => {
                if let Some(unary) = arena.get_unary_expr(node) {
                    self.bind_expression(arena, unary.operand);
                    if unary.operator == SyntaxKind::PlusPlusToken as u16
                        || unary.operator == SyntaxKind::MinusMinusToken as u16
                    {
                        let flow = self.create_flow_assignment(idx);
                        self.current_flow = flow;
                    }
                }
                return;
            }

            // Property access (e.g., x.foo) or element access (e.g., x[0])
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION
                || k == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION =>
            {
                if let Some(access) = arena.get_access_expr(node) {
                    self.bind_expression(arena, access.expression);
                    // For element access, also bind the argument
                    if k == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION {
                        self.bind_expression(arena, access.name_or_argument);
                    }
                }
                return;
            }

            // Call expression (e.g., isString(x))
            k if k == syntax_kind_ext::CALL_EXPRESSION => {
                if let Some(call) = arena.get_call_expr(node) {
                    self.bind_expression(arena, call.expression);
                    if let Some(args) = &call.arguments {
                        for &arg in &args.nodes {
                            self.bind_expression(arena, arg);
                        }
                    }
                    if self.is_array_mutation_call(arena, idx) {
                        let flow = self.create_flow_array_mutation(idx);
                        self.current_flow = flow;
                    }
                }
                return;
            }

            // Parenthesized expression
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                if let Some(paren) = arena.get_parenthesized(node) {
                    self.bind_expression(arena, paren.expression);
                }
                return;
            }

            // Type assertion (e.g., x as string)
            k if k == syntax_kind_ext::AS_EXPRESSION || k == syntax_kind_ext::TYPE_ASSERTION => {
                if let Some(as_expr) = arena.get_access_expr(node) {
                    self.bind_expression(arena, as_expr.expression);
                }
                return;
            }

            // Conditional expression (ternary)
            k if k == syntax_kind_ext::CONDITIONAL_EXPRESSION => {
                if let Some(cond) = arena.get_conditional_expr(node) {
                    self.bind_expression(arena, cond.condition);
                    self.bind_expression(arena, cond.when_true);
                    self.bind_expression(arena, cond.when_false);
                }
                return;
            }

            _ => {}
        }

        self.bind_node(arena, idx);
    }
}

impl Default for ThinBinderState {
    fn default() -> Self {
        Self::new()
    }
}

