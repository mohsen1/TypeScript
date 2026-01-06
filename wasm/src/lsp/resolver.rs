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

    /// Check if a node creates a new scope.
    fn node_creates_scope(&self, node_idx: NodeIndex) -> bool {
        let Some(node) = self.arena.get(node_idx) else { return false; };
        matches!(
            node.kind,
            k if k == syntax_kind_ext::BLOCK
                || k == syntax_kind_ext::FUNCTION_DECLARATION
                || k == syntax_kind_ext::ARROW_FUNCTION
                || k == syntax_kind_ext::FUNCTION_EXPRESSION
                || k == syntax_kind_ext::METHOD_DECLARATION
                || k == syntax_kind_ext::FOR_STATEMENT
                || k == syntax_kind_ext::FOR_IN_STATEMENT
                || k == syntax_kind_ext::FOR_OF_STATEMENT
                || k == syntax_kind_ext::CATCH_CLAUSE
                || k == syntax_kind_ext::CLASS_DECLARATION
                || k == syntax_kind_ext::CLASS_EXPRESSION
        )
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

    /// Iterate over direct children of a node using proper typed accessors.
    ///
    /// The callback `f` receives the walker and the child node index.
    /// It should return `Some(T)` to stop iteration with a result, or `None` to continue.
    fn for_each_child<T, F>(&mut self, node_idx: NodeIndex, mut f: F) -> Option<T>
    where
        F: FnMut(&mut Self, NodeIndex) -> Option<T>,
    {
        let node = self.arena.get(node_idx)?;

        match node.kind {
            // --- Source File & Blocks ---
            k if k == syntax_kind_ext::SOURCE_FILE => {
                if let Some(sf) = self.arena.get_source_file(node) {
                    for &stmt in &sf.statements.nodes {
                        if let Some(res) = f(self, stmt) { return Some(res); }
                    }
                }
            }
            k if k == syntax_kind_ext::BLOCK => {
                if let Some(block) = self.arena.get_block(node) {
                    for &stmt in &block.statements.nodes {
                        if let Some(res) = f(self, stmt) { return Some(res); }
                    }
                }
            }
            k if k == syntax_kind_ext::MODULE_BLOCK => {
                if let Some(mod_block) = self.arena.get_module_block(node) {
                    if let Some(ref stmts) = mod_block.statements {
                        for &stmt in &stmts.nodes {
                            if let Some(res) = f(self, stmt) { return Some(res); }
                        }
                    }
                }
            }

            // --- Declarations ---
            k if k == syntax_kind_ext::FUNCTION_DECLARATION
              || k == syntax_kind_ext::FUNCTION_EXPRESSION
              || k == syntax_kind_ext::ARROW_FUNCTION => {
                if let Some(func) = self.arena.get_function(node) {
                    if !func.name.is_none() { if let Some(res) = f(self, func.name) { return Some(res); } }
                    for &param in &func.parameters.nodes {
                        if let Some(res) = f(self, param) { return Some(res); }
                    }
                    if !func.type_annotation.is_none() { if let Some(res) = f(self, func.type_annotation) { return Some(res); } }
                    if !func.body.is_none() { if let Some(res) = f(self, func.body) { return Some(res); } }
                }
            }
            k if k == syntax_kind_ext::METHOD_DECLARATION => {
                if let Some(method) = self.arena.get_method_decl(node) {
                    if !method.name.is_none() { if let Some(res) = f(self, method.name) { return Some(res); } }
                    for &param in &method.parameters.nodes {
                        if let Some(res) = f(self, param) { return Some(res); }
                    }
                    if !method.body.is_none() { if let Some(res) = f(self, method.body) { return Some(res); } }
                }
            }
            k if k == syntax_kind_ext::CONSTRUCTOR => {
                if let Some(ctor) = self.arena.get_constructor(node) {
                    for &param in &ctor.parameters.nodes {
                        if let Some(res) = f(self, param) { return Some(res); }
                    }
                    if !ctor.body.is_none() { if let Some(res) = f(self, ctor.body) { return Some(res); } }
                }
            }

            k if k == syntax_kind_ext::CLASS_DECLARATION || k == syntax_kind_ext::CLASS_EXPRESSION => {
                if let Some(class) = self.arena.get_class(node) {
                    if !class.name.is_none() { if let Some(res) = f(self, class.name) { return Some(res); } }
                    for &member in &class.members.nodes {
                        if let Some(res) = f(self, member) { return Some(res); }
                    }
                }
            }

            k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
                if let Some(var) = self.arena.get_variable(node) {
                    for &decl_list in &var.declarations.nodes {
                         if let Some(res) = f(self, decl_list) { return Some(res); }
                    }
                }
            }
            k if k == syntax_kind_ext::VARIABLE_DECLARATION_LIST => {
                if let Some(list) = self.arena.get_variable(node) {
                    for &decl in &list.declarations.nodes {
                        if let Some(res) = f(self, decl) { return Some(res); }
                    }
                }
            }
            k if k == syntax_kind_ext::VARIABLE_DECLARATION => {
                if let Some(decl) = self.arena.get_variable_declaration(node) {
                    if let Some(res) = f(self, decl.name) { return Some(res); }
                    if !decl.initializer.is_none() { if let Some(res) = f(self, decl.initializer) { return Some(res); } }
                }
            }

            // --- Statements ---
            k if k == syntax_kind_ext::IF_STATEMENT => {
                if let Some(stmt) = self.arena.get_if_statement(node) {
                    if let Some(res) = f(self, stmt.expression) { return Some(res); }
                    if let Some(res) = f(self, stmt.then_statement) { return Some(res); }
                    if !stmt.else_statement.is_none() { if let Some(res) = f(self, stmt.else_statement) { return Some(res); } }
                }
            }
            k if k == syntax_kind_ext::RETURN_STATEMENT => {
                if let Some(ret) = self.arena.get_return_statement(node) {
                    if !ret.expression.is_none() { if let Some(res) = f(self, ret.expression) { return Some(res); } }
                }
            }
            k if k == syntax_kind_ext::EXPRESSION_STATEMENT => {
                if let Some(expr) = self.arena.get_expression_statement(node) {
                    if let Some(res) = f(self, expr.expression) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::FOR_STATEMENT => {
                if let Some(loop_data) = self.arena.get_loop(node) {
                    if !loop_data.initializer.is_none() { if let Some(res) = f(self, loop_data.initializer) { return Some(res); } }
                    if !loop_data.condition.is_none() { if let Some(res) = f(self, loop_data.condition) { return Some(res); } }
                    if !loop_data.incrementor.is_none() { if let Some(res) = f(self, loop_data.incrementor) { return Some(res); } }
                    if let Some(res) = f(self, loop_data.statement) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::FOR_IN_STATEMENT || k == syntax_kind_ext::FOR_OF_STATEMENT => {
                if let Some(for_in_of) = self.arena.get_for_in_of(node) {
                    if let Some(res) = f(self, for_in_of.initializer) { return Some(res); }
                    if let Some(res) = f(self, for_in_of.expression) { return Some(res); }
                    if let Some(res) = f(self, for_in_of.statement) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::WHILE_STATEMENT || k == syntax_kind_ext::DO_STATEMENT => {
                if let Some(loop_data) = self.arena.get_loop(node) {
                    if !loop_data.condition.is_none() { if let Some(res) = f(self, loop_data.condition) { return Some(res); } }
                    if let Some(res) = f(self, loop_data.statement) { return Some(res); }
                }
            }

            // --- Expressions ---
            k if k == syntax_kind_ext::BINARY_EXPRESSION => {
                if let Some(bin) = self.arena.get_binary_expr(node) {
                    if let Some(res) = f(self, bin.left) { return Some(res); }
                    if let Some(res) = f(self, bin.right) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::CALL_EXPRESSION || k == syntax_kind_ext::NEW_EXPRESSION => {
                if let Some(call) = self.arena.get_call_expr(node) {
                    if let Some(res) = f(self, call.expression) { return Some(res); }
                    if let Some(ref args) = call.arguments {
                        for &arg in &args.nodes {
                            if let Some(res) = f(self, arg) { return Some(res); }
                        }
                    }
                }
            }
            k if k == syntax_kind_ext::PROPERTY_ACCESS_EXPRESSION || k == syntax_kind_ext::ELEMENT_ACCESS_EXPRESSION => {
                if let Some(access) = self.arena.get_access_expr(node) {
                    if let Some(res) = f(self, access.expression) { return Some(res); }
                    if let Some(res) = f(self, access.name_or_argument) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::PARENTHESIZED_EXPRESSION => {
                if let Some(paren) = self.arena.get_parenthesized(node) {
                    if let Some(res) = f(self, paren.expression) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::OBJECT_LITERAL_EXPRESSION || k == syntax_kind_ext::ARRAY_LITERAL_EXPRESSION => {
                if let Some(lit) = self.arena.get_literal_expr(node) {
                    for &elem in &lit.elements.nodes {
                         if let Some(res) = f(self, elem) { return Some(res); }
                    }
                }
            }
            k if k == syntax_kind_ext::PROPERTY_ASSIGNMENT => {
                if let Some(prop) = self.arena.get_property_assignment(node) {
                    if let Some(res) = f(self, prop.name) { return Some(res); }
                    if let Some(res) = f(self, prop.initializer) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::CONDITIONAL_EXPRESSION => {
                if let Some(cond) = self.arena.get_conditional_expr(node) {
                    if let Some(res) = f(self, cond.condition) { return Some(res); }
                    if let Some(res) = f(self, cond.when_true) { return Some(res); }
                    if let Some(res) = f(self, cond.when_false) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::PREFIX_UNARY_EXPRESSION || k == syntax_kind_ext::POSTFIX_UNARY_EXPRESSION => {
                if let Some(unary) = self.arena.get_unary_expr(node) {
                    if let Some(res) = f(self, unary.operand) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::SHORTHAND_PROPERTY_ASSIGNMENT => {
                if let Some(prop) = self.arena.get_shorthand_property(node) {
                    if let Some(res) = f(self, prop.name) { return Some(res); }
                    if !prop.object_assignment_initializer.is_none() {
                        if let Some(res) = f(self, prop.object_assignment_initializer) { return Some(res); }
                    }
                }
            }
            k if k == syntax_kind_ext::SPREAD_ELEMENT || k == syntax_kind_ext::SPREAD_ASSIGNMENT => {
                if let Some(spread) = self.arena.get_spread(node) {
                    if let Some(res) = f(self, spread.expression) { return Some(res); }
                }
            }

            // --- Control Flow ---
            k if k == syntax_kind_ext::TRY_STATEMENT => {
                if let Some(try_stmt) = self.arena.get_try(node) {
                    if let Some(res) = f(self, try_stmt.try_block) { return Some(res); }
                    if !try_stmt.catch_clause.is_none() {
                        if let Some(res) = f(self, try_stmt.catch_clause) { return Some(res); }
                    }
                    if !try_stmt.finally_block.is_none() {
                        if let Some(res) = f(self, try_stmt.finally_block) { return Some(res); }
                    }
                }
            }
            k if k == syntax_kind_ext::CATCH_CLAUSE => {
                if let Some(catch) = self.arena.get_catch_clause(node) {
                    if !catch.variable_declaration.is_none() {
                        if let Some(res) = f(self, catch.variable_declaration) { return Some(res); }
                    }
                    if let Some(res) = f(self, catch.block) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::SWITCH_STATEMENT => {
                if let Some(switch) = self.arena.get_switch(node) {
                    if let Some(res) = f(self, switch.expression) { return Some(res); }
                    if let Some(res) = f(self, switch.case_block) { return Some(res); }
                }
            }
            k if k == syntax_kind_ext::CASE_CLAUSE || k == syntax_kind_ext::DEFAULT_CLAUSE => {
                if let Some(case) = self.arena.get_case_clause(node) {
                    if !case.expression.is_none() {
                        if let Some(res) = f(self, case.expression) { return Some(res); }
                    }
                    for &stmt in &case.statements.nodes {
                        if let Some(res) = f(self, stmt) { return Some(res); }
                    }
                }
            }

            // --- Default: no children or not yet implemented ---
            _ => {}
        }

        None
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

        let Some(node) = self.arena.get(current) else { return None; };

        // Optimization: Don't descend if target is not within current node's range
        if let Some(target_node) = self.arena.get(target) {
            if target_node.pos < node.pos || target_node.pos >= node.end {
                return None;
            }
        }

        // Check if this node creates a new scope
        let creates_scope = self.node_creates_scope(current);

        if creates_scope {
            self.push_scope();
            // Register declarations visible in this scope
            self.register_local_declarations(current);
        }

        path.push(current);

        // Recurse into children using for_each_child
        let result = self.for_each_child(current, |walker, child_idx| {
            walker.walk_to_node(child_idx, target, path)
        });

        path.pop();

        if creates_scope {
            self.pop_scope();
        }

        result
    }

    /// Register local declarations from a container node.
    fn register_local_declarations(&mut self, container: NodeIndex) {
        // Iterate over direct children to find declarations
        self.for_each_child(container, |walker, child_idx| {
            // Check if this child has a symbol associated in the binder
            if let Some(&sym_id) = walker.binder.node_symbols.get(&child_idx.0) {
                if let Some(symbol) = walker.binder.symbols.get(sym_id) {
                    walker.declare_local(symbol.escaped_name.clone(), sym_id);
                }
            }

            // For VariableDeclarationList, we need to go one level deeper
            if let Some(node) = walker.arena.get(child_idx) {
                if node.kind == syntax_kind_ext::VARIABLE_DECLARATION_LIST {
                    walker.for_each_child(child_idx, |w, decl_idx| {
                        if let Some(&sym_id) = w.binder.node_symbols.get(&decl_idx.0) {
                            if let Some(symbol) = w.binder.symbols.get(sym_id) {
                                w.declare_local(symbol.escaped_name.clone(), sym_id);
                            }
                        }
                        None::<()> // Continue iteration
                    });
                }
            }

            None::<()> // Continue iteration
        });
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
        let Some(node) = self.arena.get(current) else { return; };

        // 1. Manage Scope (same logic as walk_to_node)
        let creates_scope = self.node_creates_scope(current);

        if creates_scope {
            self.push_scope();
            self.register_local_declarations(current);
        }

        // 2. Check if this is an identifier with matching text
        if node.kind == SyntaxKind::Identifier as u16 {
            if let Some(text) = self.arena.get_identifier_text(current) {
                if text == target_name {
                    // Check if this is a declaration
                    if let Some(&sym_id) = self.binder.node_symbols.get(&current.0) {
                        // This is a declaration, check if it's the right symbol
                        if sym_id == target_symbol {
                            refs.push(current);
                        }
                    } else {
                        // It's a usage - resolve using CURRENT scope stack (O(1))
                        if let Some(resolved_sym) = self.resolve_name(text) {
                            if resolved_sym == target_symbol {
                                refs.push(current);
                            }
                        }
                    }
                }
            }
        }

        // 3. Recurse into children
        self.for_each_child(current, |walker, child_idx| {
            walker.collect_references(child_idx, target_name, target_symbol, refs);
            None::<()> // Continue iteration
        });

        // 4. Pop Scope
        if creates_scope {
            self.pop_scope();
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
