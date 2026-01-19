//! Scope Chain Implementation
//!
//! This module handles scope management for the TypeScript binder,
//! including function-scoped and block-scoped variable handling.

use super::symbols::{SymbolId, SymbolTable};

/// Types of scopes in TypeScript
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// Global scope
    Global,
    /// Module scope (file-level)
    Module,
    /// Function scope
    Function,
    /// Block scope (if, for, while, etc.)
    Block,
    /// Class scope
    Class,
    /// Enum scope
    Enum,
    /// Namespace scope
    Namespace,
    /// Switch statement scope
    Switch,
    /// Catch clause scope
    Catch,
    /// With statement scope (deprecated but supported)
    With,
}

impl ScopeKind {
    /// Whether this scope kind is a function scope container
    pub fn is_function_scope_container(&self) -> bool {
        matches!(self, ScopeKind::Global | ScopeKind::Module | ScopeKind::Function)
    }

    /// Whether this scope kind is a block scope container
    pub fn is_block_scope_container(&self) -> bool {
        matches!(
            self,
            ScopeKind::Block | ScopeKind::Switch | ScopeKind::Catch |
            ScopeKind::Class | ScopeKind::Enum | ScopeKind::Namespace
        )
    }
}

/// A scope represents a lexical scope in the program
#[derive(Debug)]
pub struct Scope {
    pub id: u32,
    pub kind: ScopeKind,
    /// Symbols declared in this scope
    pub locals: SymbolTable,
    /// Parent scope ID
    pub parent: Option<u32>,
    /// Whether this scope is strict mode
    pub strict_mode: bool,
    /// The container for function-scoped variables
    pub function_scope_container: Option<u32>,
}

impl Scope {
    pub fn new(id: u32, kind: ScopeKind, parent: Option<u32>) -> Self {
        Scope {
            id,
            kind,
            locals: SymbolTable::new(),
            parent,
            strict_mode: false,
            function_scope_container: None,
        }
    }

    pub fn add_local(&mut self, name: impl Into<String>, symbol_id: SymbolId) {
        self.locals.set(name, symbol_id);
    }

    pub fn get_local(&self, name: &str) -> Option<SymbolId> {
        self.locals.get(name)
    }

    pub fn has_local(&self, name: &str) -> bool {
        self.locals.has(name)
    }
}

/// Manages the scope chain during binding
#[derive(Debug)]
pub struct ScopeChain {
    /// All scopes in the program
    scopes: Vec<Scope>,
    /// Current scope ID
    current: u32,
    /// Stack of scope IDs
    scope_stack: Vec<u32>,
    /// Stack of function scope containers
    container_stack: Vec<u32>,
    /// Stack of block scope containers
    block_stack: Vec<u32>,
    /// Next scope ID
    next_id: u32,
}

impl ScopeChain {
    pub fn new() -> Self {
        let mut chain = ScopeChain {
            scopes: Vec::new(),
            current: 0,
            scope_stack: Vec::new(),
            container_stack: Vec::new(),
            block_stack: Vec::new(),
            next_id: 1,
        };

        // Create global scope
        let global_id = chain.create_scope(ScopeKind::Global, None);
        chain.current = global_id;
        chain.scope_stack.push(global_id);
        chain.container_stack.push(global_id);

        chain
    }

    fn create_scope(&mut self, kind: ScopeKind, parent: Option<u32>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let scope = Scope::new(id, kind, parent);
        self.scopes.push(scope);
        id
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self, kind: ScopeKind) -> u32 {
        let parent = Some(self.current);
        let id = self.create_scope(kind, parent);

        // Update function scope container reference
        if kind.is_function_scope_container() {
            self.scopes[(id - 1) as usize].function_scope_container = Some(id);
            self.container_stack.push(id);
        } else if let Some(&container) = self.container_stack.last() {
            self.scopes[(id - 1) as usize].function_scope_container = Some(container);
        }

        // Track block scope containers
        if kind.is_block_scope_container() {
            self.block_stack.push(id);
        }

        // Inherit strict mode from parent
        if let Some(parent_id) = parent {
            let parent_strict = self.scopes[(parent_id - 1) as usize].strict_mode;
            self.scopes[(id - 1) as usize].strict_mode = parent_strict;
        }

        self.scope_stack.push(id);
        self.current = id;
        id
    }

    /// Pop the current scope
    pub fn pop_scope(&mut self) {
        if self.scope_stack.len() <= 1 {
            return; // Don't pop the global scope
        }

        let popped_id = self.scope_stack.pop().unwrap();
        let kind = self.scopes[(popped_id - 1) as usize].kind;

        if kind.is_function_scope_container() {
            self.container_stack.pop();
        }

        if kind.is_block_scope_container() {
            self.block_stack.pop();
        }

        self.current = *self.scope_stack.last().unwrap();
    }

    /// Get the current scope
    pub fn current_scope(&self) -> &Scope {
        &self.scopes[(self.current - 1) as usize]
    }

    /// Get the current scope mutably
    pub fn current_scope_mut(&mut self) -> &mut Scope {
        &mut self.scopes[(self.current - 1) as usize]
    }

    /// Get a scope by ID
    pub fn get_scope(&self, id: u32) -> Option<&Scope> {
        if id == 0 || id as usize > self.scopes.len() {
            None
        } else {
            Some(&self.scopes[(id - 1) as usize])
        }
    }

    /// Get a scope by ID mutably
    pub fn get_scope_mut(&mut self, id: u32) -> Option<&mut Scope> {
        if id == 0 || id as usize > self.scopes.len() {
            None
        } else {
            Some(&mut self.scopes[(id - 1) as usize])
        }
    }

    /// Get the current function scope container
    pub fn current_container(&self) -> Option<u32> {
        self.container_stack.last().copied()
    }

    /// Get the current block scope container
    pub fn current_block_container(&self) -> Option<u32> {
        self.block_stack.last().copied()
    }

    /// Set strict mode for the current scope
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.scopes[(self.current - 1) as usize].strict_mode = strict;
    }

    /// Look up a symbol in the scope chain
    pub fn lookup(&self, name: &str) -> Option<(SymbolId, u32)> {
        let mut scope_id = self.current;

        loop {
            let scope = &self.scopes[(scope_id - 1) as usize];
            if let Some(symbol_id) = scope.locals.get(name) {
                return Some((symbol_id, scope_id));
            }

            match scope.parent {
                Some(parent) => scope_id = parent,
                None => return None,
            }
        }
    }

    /// Add a symbol to the current scope
    pub fn add_to_current_scope(&mut self, name: impl Into<String>, symbol_id: SymbolId) {
        self.current_scope_mut().add_local(name, symbol_id);
    }

    /// Add a function-scoped variable to the appropriate container
    pub fn add_function_scoped(&mut self, name: impl Into<String>, symbol_id: SymbolId) {
        if let Some(container_id) = self.current_container() {
            self.scopes[(container_id - 1) as usize].add_local(name, symbol_id);
        }
    }

    /// Add a block-scoped variable to the current scope
    pub fn add_block_scoped(&mut self, name: impl Into<String>, symbol_id: SymbolId) {
        self.add_to_current_scope(name, symbol_id);
    }

    /// Get the depth of the scope chain
    pub fn depth(&self) -> usize {
        self.scope_stack.len()
    }

    /// Check if currently in strict mode
    pub fn in_strict_mode(&self) -> bool {
        self.current_scope().strict_mode
    }
}

impl Default for ScopeChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_creation() {
        let chain = ScopeChain::new();
        assert_eq!(chain.current_scope().kind, ScopeKind::Global);
    }

    #[test]
    fn test_scope_push_pop() {
        let mut chain = ScopeChain::new();

        let func_id = chain.push_scope(ScopeKind::Function);
        assert_eq!(chain.current_scope().kind, ScopeKind::Function);
        assert_eq!(chain.current, func_id);

        let block_id = chain.push_scope(ScopeKind::Block);
        assert_eq!(chain.current_scope().kind, ScopeKind::Block);
        assert_eq!(chain.current, block_id);

        chain.pop_scope();
        assert_eq!(chain.current, func_id);

        chain.pop_scope();
        assert_eq!(chain.current_scope().kind, ScopeKind::Global);
    }

    #[test]
    fn test_symbol_lookup() {
        let mut chain = ScopeChain::new();

        // Add symbol to global scope
        chain.add_to_current_scope("global_var", 1);

        // Push function scope and add local
        chain.push_scope(ScopeKind::Function);
        chain.add_to_current_scope("local_var", 2);

        // Can find both
        assert!(chain.lookup("global_var").is_some());
        assert!(chain.lookup("local_var").is_some());

        // Pop function scope
        chain.pop_scope();

        // Can only find global now
        assert!(chain.lookup("global_var").is_some());
        assert!(chain.lookup("local_var").is_none());
    }

    #[test]
    fn test_function_scoped_variables() {
        let mut chain = ScopeChain::new();

        chain.push_scope(ScopeKind::Function);
        chain.push_scope(ScopeKind::Block);

        // Add function-scoped var from inside block
        chain.add_function_scoped("var_x", 1);

        // Variable should be in function scope, not block scope
        assert!(chain.current_scope().get_local("var_x").is_none());

        chain.pop_scope(); // Pop block
        assert!(chain.current_scope().get_local("var_x").is_some());
    }

    #[test]
    fn test_strict_mode_inheritance() {
        let mut chain = ScopeChain::new();

        chain.push_scope(ScopeKind::Module);
        chain.set_strict_mode(true);
        assert!(chain.in_strict_mode());

        chain.push_scope(ScopeKind::Function);
        // Should inherit strict mode
        assert!(chain.in_strict_mode());

        chain.push_scope(ScopeKind::Block);
        assert!(chain.in_strict_mode());
    }

    #[test]
    fn test_container_tracking() {
        let mut chain = ScopeChain::new();

        // Global is the initial container
        let global_container = chain.current_container();
        assert!(global_container.is_some());

        // Push function - becomes new container
        let func_id = chain.push_scope(ScopeKind::Function);
        assert_eq!(chain.current_container(), Some(func_id));

        // Push block - container stays as function
        chain.push_scope(ScopeKind::Block);
        assert_eq!(chain.current_container(), Some(func_id));

        // Pop block - still function
        chain.pop_scope();
        assert_eq!(chain.current_container(), Some(func_id));

        // Pop function - back to global
        chain.pop_scope();
        assert_eq!(chain.current_container(), global_container);
    }
}
