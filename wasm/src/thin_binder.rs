//! ThinBinder - Binder implementation using ThinNodeArena.
//!
//! This is a clean implementation of the binder that works directly with
//! ThinNode and ThinNodeArena, avoiding the old Node enum pattern matching.

// Allow dead code for binder infrastructure methods that will be used in future phases
#![allow(dead_code)]

use crate::parser::thin_node::{ThinNodeArena, ThinNode};
use crate::parser::{NodeIndex, NodeList, syntax_kind_ext};
use crate::binder::{
    SymbolId, SymbolArena, SymbolTable, Symbol, symbol_flags,
    FlowNodeArena, FlowNodeId, flow_flags,
    ContainerKind, ScopeContext,
};
use crate::parser::node_flags;
use rustc_hash::FxHashMap;

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
    /// Scope chain - stack of scope contexts
    scope_chain: Vec<ScopeContext>,
    /// Current scope index in scope_chain
    current_scope_idx: usize,
    /// Node-to-symbol mapping
    pub node_symbols: FxHashMap<u32, SymbolId>,
    /// Hoisted var declarations
    hoisted_vars: Vec<(String, NodeIndex)>,
    /// Hoisted function declarations
    hoisted_functions: Vec<NodeIndex>,
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
            hoisted_vars: Vec::new(),
            hoisted_functions: Vec::new(),
        }
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
            hoisted_vars: Vec::new(),
            hoisted_functions: Vec::new(),
        }
    }

    /// Bind a source file using ThinNodeArena.
    pub fn bind_source_file(&mut self, arena: &ThinNodeArena, root: NodeIndex) {
        // Initialize scope chain with source file scope
        self.scope_chain.clear();
        self.scope_chain.push(ScopeContext::new(ContainerKind::SourceFile, root, None));
        self.current_scope_idx = 0;
        self.current_scope = SymbolTable::new();

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
                }
            }
        }

        // Store file locals
        self.file_locals = std::mem::take(&mut self.current_scope);
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
                        let sym_id = self.symbols.alloc(symbol_flags::FUNCTION, name.to_string());
                        // Set the value_declaration to this function declaration node
                        if let Some(sym) = self.symbols.get_mut(sym_id) {
                            sym.value_declaration = func_idx;
                            sym.declarations.push(func_idx);
                        }
                        self.current_scope.set(name.to_string(), sym_id);
                        self.node_symbols.insert(func_idx.0, sym_id);
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

            // If statement
            k if k == syntax_kind_ext::IF_STATEMENT => {
                if let Some(if_stmt) = arena.get_if_statement(node) {
                    self.bind_node(arena, if_stmt.then_statement);
                    if !if_stmt.else_statement.is_none() {
                        self.bind_node(arena, if_stmt.else_statement);
                    }
                }
            }

            // While/do statement
            k if k == syntax_kind_ext::WHILE_STATEMENT ||
                 k == syntax_kind_ext::DO_STATEMENT => {
                if let Some(loop_data) = arena.get_loop(node) {
                    self.bind_node(arena, loop_data.statement);
                }
            }

            // For statement
            k if k == syntax_kind_ext::FOR_STATEMENT => {
                if let Some(loop_data) = arena.get_loop(node) {
                    self.enter_scope(ContainerKind::Block, idx);
                    self.bind_node(arena, loop_data.initializer);
                    self.bind_node(arena, loop_data.statement);
                    self.exit_scope();
                }
            }

            // For-in/for-of
            k if k == syntax_kind_ext::FOR_IN_STATEMENT ||
                 k == syntax_kind_ext::FOR_OF_STATEMENT => {
                if let Some(for_data) = arena.get_for_in_of(node) {
                    self.enter_scope(ContainerKind::Block, idx);
                    self.bind_node(arena, for_data.initializer);
                    self.bind_node(arena, for_data.statement);
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

            // Import declarations
            k if k == syntax_kind_ext::IMPORT_DECLARATION => {
                self.bind_import_declaration(arena, node, idx);
            }

            // Export declarations - bind the exported declaration
            k if k == syntax_kind_ext::EXPORT_DECLARATION => {
                self.bind_export_declaration(arena, node, idx);
            }

            // Module/namespace declarations
            k if k == syntax_kind_ext::MODULE_DECLARATION => {
                self.bind_module_declaration(arena, node, idx);
            }
            k if k == syntax_kind_ext::MODULE_BLOCK => {
                if let Some(block) = arena.get_block(node) {
                    for &stmt_idx in &block.statements.nodes {
                        self.bind_node(arena, stmt_idx);
                    }
                }
            }

            // Expression statements - traverse into the expression
            k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                if let Some(expr_stmt) = arena.get_expression_statement(node) {
                    self.bind_node(arena, expr_stmt.expression);
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

    // Scope management

    fn enter_scope(&mut self, kind: ContainerKind, node: NodeIndex) {
        let parent = Some(self.current_scope_idx);
        self.scope_chain.push(ScopeContext::new(kind, node, parent));
        self.current_scope_idx = self.scope_chain.len() - 1;
        self.push_scope();
    }

    fn exit_scope(&mut self) {
        self.pop_scope();
        if let Some(ctx) = self.scope_chain.get(self.current_scope_idx) {
            if let Some(parent) = ctx.parent_idx {
                self.current_scope_idx = parent;
            }
        }
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
            if let Some(name) = self.get_identifier_name(arena, decl.name) {
                // Determine if block-scoped (let/const) or function-scoped (var)
                let flags = if (node.flags as u32 & (node_flags::LET | node_flags::CONST)) != 0 {
                    symbol_flags::BLOCK_SCOPED_VARIABLE
                } else {
                    symbol_flags::FUNCTION_SCOPED_VARIABLE
                };

                let sym_id = self.symbols.alloc(flags, name.to_string());
                // Set the value_declaration to this variable declaration node
                if let Some(sym) = self.symbols.get_mut(sym_id) {
                    sym.value_declaration = idx;
                    sym.declarations.push(idx);
                }
                self.current_scope.set(name.to_string(), sym_id);
                self.node_symbols.insert(idx.0, sym_id);
            }
        }
    }

    fn bind_function_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(func) = arena.get_function(node) {
            // Function declaration creates a symbol in the current scope
            if let Some(name) = self.get_identifier_name(arena, func.name) {
                // Check if already bound (hoisted)
                if !self.current_scope.has(name) {
                    let sym_id = self.symbols.alloc(symbol_flags::FUNCTION, name.to_string());
                    // Set the value_declaration to this function declaration node
                    if let Some(sym) = self.symbols.get_mut(sym_id) {
                        sym.value_declaration = idx;
                        sym.declarations.push(idx);
                    }
                    self.current_scope.set(name.to_string(), sym_id);
                    self.node_symbols.insert(idx.0, sym_id);
                }
            }

            // Enter function scope and bind body
            self.enter_scope(ContainerKind::Function, idx);

            // Bind parameters
            for &param_idx in &func.parameters.nodes {
                self.bind_parameter(arena, param_idx);
            }

            // Bind body
            self.bind_node(arena, func.body);

            self.exit_scope();
        }
    }

    fn bind_parameter(&mut self, arena: &ThinNodeArena, idx: NodeIndex) {
        if let Some(node) = arena.get(idx) {
            if let Some(param) = arena.get_parameter(node) {
                if let Some(name) = self.get_identifier_name(arena, param.name) {
                    let sym_id = self.symbols.alloc(symbol_flags::FUNCTION_SCOPED_VARIABLE, name.to_string());
                    self.current_scope.set(name.to_string(), sym_id);
                    self.node_symbols.insert(idx.0, sym_id);
                }
            }
        }
    }

    /// Bind an arrow function expression - creates a scope and binds the body.
    fn bind_arrow_function(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(func) = arena.get_function(node) {
            // Enter function scope
            self.enter_scope(ContainerKind::Function, idx);

            // Bind parameters
            for &param_idx in &func.parameters.nodes {
                self.bind_parameter(arena, param_idx);
            }

            // Bind body (could be a block or an expression)
            self.bind_node(arena, func.body);

            self.exit_scope();
        }
    }

    /// Bind a function expression - creates a scope and binds the body.
    fn bind_function_expression(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(func) = arena.get_function(node) {
            // Enter function scope
            self.enter_scope(ContainerKind::Function, idx);

            // Bind parameters
            for &param_idx in &func.parameters.nodes {
                self.bind_parameter(arena, param_idx);
            }

            // Bind body
            self.bind_node(arena, func.body);

            self.exit_scope();
        }
    }

    fn bind_class_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(class) = arena.get_class(node) {
            if let Some(name) = self.get_identifier_name(arena, class.name) {
                // Start with CLASS flag
                let mut flags = symbol_flags::CLASS;

                // Add ABSTRACT flag if class has 'abstract' modifier
                if self.has_abstract_modifier(arena, &class.modifiers) {
                    flags |= symbol_flags::ABSTRACT;
                }

                let sym_id = self.symbols.alloc(flags, name.to_string());
                self.current_scope.set(name.to_string(), sym_id);
                self.node_symbols.insert(idx.0, sym_id);
            }

            // Enter class scope for members
            self.enter_scope(ContainerKind::Class, idx);

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
                        if let Some(name) = self.get_identifier_name(arena, method.name) {
                            let mut flags = symbol_flags::METHOD;
                            if self.has_abstract_modifier(arena, &method.modifiers) {
                                flags |= symbol_flags::ABSTRACT;
                            }
                            if self.has_static_modifier(arena, &method.modifiers) {
                                flags |= symbol_flags::STATIC;
                            }
                            let sym_id = self.symbols.alloc(flags, name.to_string());
                            self.current_scope.set(name.to_string(), sym_id);
                            self.node_symbols.insert(idx.0, sym_id);
                        }
                    }
                }
                k if k == syntax_kind_ext::PROPERTY_DECLARATION => {
                    if let Some(prop) = arena.get_property_decl(node) {
                        if let Some(name) = self.get_identifier_name(arena, prop.name) {
                            let mut flags = symbol_flags::PROPERTY;
                            if self.has_abstract_modifier(arena, &prop.modifiers) {
                                flags |= symbol_flags::ABSTRACT;
                            }
                            if self.has_static_modifier(arena, &prop.modifiers) {
                                flags |= symbol_flags::STATIC;
                            }
                            let sym_id = self.symbols.alloc(flags, name.to_string());
                            self.current_scope.set(name.to_string(), sym_id);
                            self.node_symbols.insert(idx.0, sym_id);
                        }
                    }
                }
                k if k == syntax_kind_ext::CONSTRUCTOR => {
                    let sym_id = self.symbols.alloc(symbol_flags::CONSTRUCTOR, "constructor".to_string());
                    self.current_scope.set("constructor".to_string(), sym_id);
                    self.node_symbols.insert(idx.0, sym_id);
                }
                _ => {}
            }
        }
    }

    fn bind_interface_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(iface) = arena.get_interface(node) {
            if let Some(name) = self.get_identifier_name(arena, iface.name) {
                let sym_id = self.symbols.alloc(symbol_flags::INTERFACE, name.to_string());
                self.current_scope.set(name.to_string(), sym_id);
                self.node_symbols.insert(idx.0, sym_id);
            }
        }
    }

    fn bind_type_alias_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(alias) = arena.get_type_alias(node) {
            if let Some(name) = self.get_identifier_name(arena, alias.name) {
                let sym_id = self.symbols.alloc(symbol_flags::TYPE_ALIAS, name.to_string());
                self.current_scope.set(name.to_string(), sym_id);
                self.node_symbols.insert(idx.0, sym_id);
            }
        }
    }

    fn bind_enum_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(enum_decl) = arena.get_enum(node) {
            if let Some(name) = self.get_identifier_name(arena, enum_decl.name) {
                let sym_id = self.symbols.alloc(symbol_flags::REGULAR_ENUM, name.to_string());
                self.current_scope.set(name.to_string(), sym_id);
                self.node_symbols.insert(idx.0, sym_id);
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

    fn bind_switch_statement(&mut self, arena: &ThinNodeArena, node: &ThinNode, _idx: NodeIndex) {
        if let Some(switch_data) = arena.get_switch(node) {
            // Case block contains case clauses
            if let Some(case_block_node) = arena.get(switch_data.case_block) {
                if let Some(case_block) = arena.get_block(case_block_node) {
                    for &clause_idx in &case_block.statements.nodes {
                        if let Some(clause_node) = arena.get(clause_idx) {
                            if let Some(clause) = arena.get_case_clause(clause_node) {
                                for &stmt_idx in &clause.statements.nodes {
                                    self.bind_node(arena, stmt_idx);
                                }
                            }
                        }
                    }
                }
            }
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
                            self.current_scope.set(name.to_string(), sym_id);
                        }
                    }

                    // Named imports
                    if !clause.named_bindings.is_none() {
                        if let Some(bindings_node) = arena.get(clause.named_bindings) {
                            if let Some(named) = arena.get_named_imports(bindings_node) {
                                for &spec_idx in &named.elements.nodes {
                                    if let Some(spec_node) = arena.get(spec_idx) {
                                        if let Some(spec) = arena.get_specifier(spec_node) {
                                            let local_name = if !spec.name.is_none() {
                                                self.get_identifier_name(arena, spec.name)
                                            } else {
                                                self.get_identifier_name(arena, spec.property_name)
                                            };

                                            if let Some(name) = local_name {
                                                let sym_id = self.symbols.alloc(symbol_flags::ALIAS, name.to_string());
                                                self.current_scope.set(name.to_string(), sym_id);
                                                self.node_symbols.insert(spec_idx.0, sym_id);
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
                    }
                    // Namespace export: export * as ns from 'mod'
                    else if let Some(name) = self.get_identifier_name(arena, export.export_clause) {
                        let sym_id = self.symbols.alloc(symbol_flags::ALIAS, name.to_string());
                        self.current_scope.set(name.to_string(), sym_id);
                        self.node_symbols.insert(export.export_clause.0, sym_id);
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
        kind == syntax_kind_ext::ENUM_DECLARATION
    }

    fn bind_module_declaration(&mut self, arena: &ThinNodeArena, node: &ThinNode, idx: NodeIndex) {
        if let Some(module) = arena.get_module(node) {
            if let Some(name) = self.get_identifier_name(arena, module.name) {
                let sym_id = self.symbols.alloc(symbol_flags::VALUE_MODULE, name.to_string());
                self.current_scope.set(name.to_string(), sym_id);
                self.node_symbols.insert(idx.0, sym_id);
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
}

impl Default for ThinBinderState {
    fn default() -> Self {
        Self::new()
    }
}

