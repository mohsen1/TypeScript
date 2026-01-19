//! Scope chain management for the binder
//!
//! This module implements scope management for TypeScript binding.
//! It handles:
//! - Creating and managing scope chains
//! - Block scoping for let/const
//! - Module/namespace scoping
//! - Class and function scoping

use crate::symbols::{SymbolId, SymbolTable};

/// The kind of scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    /// Global/module scope (SourceFile)
    Global,
    /// Module scope (for ES modules)
    Module,
    /// Namespace scope
    Namespace,
    /// Function scope
    Function,
    /// Block scope (for let/const in blocks)
    Block,
    /// Class scope
    Class,
    /// Interface scope
    Interface,
    /// Enum scope
    Enum,
    /// Type literal scope
    TypeLiteral,
    /// Object literal scope
    ObjectLiteral,
    /// Catch clause scope
    Catch,
    /// For loop scope
    ForLoop,
}

impl ScopeKind {
    /// Check if this scope kind creates a new container for locals
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            ScopeKind::Global
                | ScopeKind::Module
                | ScopeKind::Namespace
                | ScopeKind::Function
                | ScopeKind::Class
                | ScopeKind::Interface
                | ScopeKind::Enum
                | ScopeKind::TypeLiteral
                | ScopeKind::ObjectLiteral
        )
    }

    /// Check if this scope kind is block-scoped
    pub fn is_block_scoped(&self) -> bool {
        matches!(
            self,
            ScopeKind::Block | ScopeKind::Catch | ScopeKind::ForLoop
        )
    }

    /// Check if this scope can have exports
    pub fn can_have_exports(&self) -> bool {
        matches!(
            self,
            ScopeKind::Global | ScopeKind::Module | ScopeKind::Namespace
        )
    }

    /// Check if this scope can have members
    pub fn can_have_members(&self) -> bool {
        matches!(
            self,
            ScopeKind::Class
                | ScopeKind::Interface
                | ScopeKind::TypeLiteral
                | ScopeKind::ObjectLiteral
        )
    }
}

/// Unique identifier for a scope
pub type ScopeId = u32;

/// Represents a single scope in the scope chain
#[derive(Debug, Clone)]
pub struct Scope {
    /// Unique identifier for this scope
    pub id: ScopeId,
    /// The kind of scope
    pub kind: ScopeKind,
    /// Parent scope in the scope chain
    pub parent: Option<ScopeId>,
    /// Local symbols declared in this scope
    pub locals: SymbolTable,
    /// Symbol associated with this scope (e.g., function symbol, class symbol)
    pub symbol: Option<SymbolId>,
    /// Whether this scope is in strict mode
    pub strict_mode: bool,
    /// The node ID that created this scope
    pub node_id: Option<u32>,
}

impl Scope {
    /// Create a new scope
    pub fn new(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>) -> Self {
        Scope {
            id,
            kind,
            parent,
            locals: SymbolTable::new(),
            symbol: None,
            strict_mode: false,
            node_id: None,
        }
    }

    /// Create a new scope with strict mode inherited from parent
    pub fn with_strict_mode(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>, strict: bool) -> Self {
        let mut scope = Self::new(id, kind, parent);
        scope.strict_mode = strict;
        scope
    }
}

/// Manages the scope chain during binding
#[derive(Debug)]
pub struct ScopeChain {
    /// All scopes created during binding
    scopes: Vec<Scope>,
    /// Stack of active scope IDs
    scope_stack: Vec<ScopeId>,
    /// Stack of container scope IDs (for function-scoped variables)
    container_stack: Vec<ScopeId>,
    /// Stack of block scope IDs (for block-scoped variables)
    block_scope_stack: Vec<ScopeId>,
    /// Next scope ID to assign
    next_id: ScopeId,
}

impl ScopeChain {
    /// Create a new scope chain
    pub fn new() -> Self {
        ScopeChain {
            scopes: Vec::new(),
            scope_stack: Vec::new(),
            container_stack: Vec::new(),
            block_scope_stack: Vec::new(),
            next_id: 0,
        }
    }

    /// Create a new scope chain with a global scope already pushed
    pub fn with_global_scope() -> Self {
        let mut chain = Self::new();
        chain.push_scope(ScopeKind::Global);
        chain
    }

    /// Push a new scope onto the chain
    pub fn push_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let parent = self.scope_stack.last().copied();
        let strict = parent
            .and_then(|id| self.get_scope(id))
            .map_or(false, |s| s.strict_mode);

        let id = self.next_id;
        self.next_id += 1;

        let scope = Scope::with_strict_mode(id, kind, parent, strict);
        self.scopes.push(scope);
        self.scope_stack.push(id);

        // Update container and block scope stacks
        if kind.is_container() {
            self.container_stack.push(id);
        }
        if kind.is_block_scoped() || kind.is_container() {
            self.block_scope_stack.push(id);
        }

        id
    }

    /// Pop the current scope from the chain
    pub fn pop_scope(&mut self) -> Option<ScopeId> {
        let id = self.scope_stack.pop()?;

        // Get the kind first to avoid borrow issues
        let kind = self.get_scope(id).map(|s| s.kind);

        if let Some(kind) = kind {
            if kind.is_container() {
                self.container_stack.pop();
            }
            if kind.is_block_scoped() || kind.is_container() {
                self.block_scope_stack.pop();
            }
        }

        Some(id)
    }

    /// Get the current scope ID
    pub fn current_scope_id(&self) -> Option<ScopeId> {
        self.scope_stack.last().copied()
    }

    /// Get the current scope
    pub fn current_scope(&self) -> Option<&Scope> {
        self.current_scope_id().and_then(|id| self.get_scope(id))
    }

    /// Get a mutable reference to the current scope
    pub fn current_scope_mut(&mut self) -> Option<&mut Scope> {
        self.current_scope_id()
            .and_then(move |id| self.get_scope_mut(id))
    }

    /// Get the current container scope ID (for function-scoped variables)
    pub fn current_container_id(&self) -> Option<ScopeId> {
        self.container_stack.last().copied()
    }

    /// Get the current container scope
    pub fn current_container(&self) -> Option<&Scope> {
        self.current_container_id()
            .and_then(|id| self.get_scope(id))
    }

    /// Get a mutable reference to the current container scope
    pub fn current_container_mut(&mut self) -> Option<&mut Scope> {
        self.current_container_id()
            .and_then(move |id| self.get_scope_mut(id))
    }

    /// Get the current block scope ID (for block-scoped variables)
    pub fn current_block_scope_id(&self) -> Option<ScopeId> {
        self.block_scope_stack.last().copied()
    }

    /// Get the current block scope
    pub fn current_block_scope(&self) -> Option<&Scope> {
        self.current_block_scope_id()
            .and_then(|id| self.get_scope(id))
    }

    /// Get a mutable reference to the current block scope
    pub fn current_block_scope_mut(&mut self) -> Option<&mut Scope> {
        self.current_block_scope_id()
            .and_then(move |id| self.get_scope_mut(id))
    }

    /// Get a scope by ID
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id as usize)
    }

    /// Get a mutable reference to a scope by ID
    pub fn get_scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id as usize)
    }

    /// Get the parent of a scope
    pub fn get_parent(&self, id: ScopeId) -> Option<&Scope> {
        self.get_scope(id)
            .and_then(|s| s.parent)
            .and_then(|pid| self.get_scope(pid))
    }

    /// Set the symbol for the current scope
    pub fn set_current_symbol(&mut self, symbol: SymbolId) {
        if let Some(scope) = self.current_scope_mut() {
            scope.symbol = Some(symbol);
        }
    }

    /// Set strict mode for the current scope
    pub fn set_strict_mode(&mut self, strict: bool) {
        if let Some(scope) = self.current_scope_mut() {
            scope.strict_mode = strict;
        }
    }

    /// Check if we're currently in strict mode
    pub fn is_strict_mode(&self) -> bool {
        self.current_scope().map_or(false, |s| s.strict_mode)
    }

    /// Look up a symbol by name in the current scope chain
    pub fn lookup(&self, name: &str) -> Option<SymbolId> {
        let mut scope_id = self.current_scope_id();
        while let Some(id) = scope_id {
            if let Some(scope) = self.get_scope(id) {
                if let Some(symbol_id) = scope.locals.get(name) {
                    return Some(symbol_id);
                }
                scope_id = scope.parent;
            } else {
                break;
            }
        }
        None
    }

    /// Look up a symbol in container scopes only (for function-scoped variables)
    pub fn lookup_in_containers(&self, name: &str) -> Option<SymbolId> {
        for &scope_id in self.container_stack.iter().rev() {
            if let Some(scope) = self.get_scope(scope_id) {
                if let Some(symbol_id) = scope.locals.get(name) {
                    return Some(symbol_id);
                }
            }
        }
        None
    }

    /// Look up a symbol in block scopes (for block-scoped variables)
    pub fn lookup_in_block_scopes(&self, name: &str) -> Option<SymbolId> {
        for &scope_id in self.block_scope_stack.iter().rev() {
            if let Some(scope) = self.get_scope(scope_id) {
                if let Some(symbol_id) = scope.locals.get(name) {
                    return Some(symbol_id);
                }
            }
        }
        None
    }

    /// Get the depth of the current scope
    pub fn depth(&self) -> usize {
        self.scope_stack.len()
    }

    /// Get all scopes
    pub fn all_scopes(&self) -> &[Scope] {
        &self.scopes
    }

    /// Clear all scopes (for reuse)
    pub fn clear(&mut self) {
        self.scopes.clear();
        self.scope_stack.clear();
        self.container_stack.clear();
        self.block_scope_stack.clear();
        self.next_id = 0;
    }

    /// Find the enclosing scope of a specific kind
    pub fn find_enclosing_scope(&self, kind: ScopeKind) -> Option<&Scope> {
        let mut scope_id = self.current_scope_id();
        while let Some(id) = scope_id {
            if let Some(scope) = self.get_scope(id) {
                if scope.kind == kind {
                    return Some(scope);
                }
                scope_id = scope.parent;
            } else {
                break;
            }
        }
        None
    }

    /// Find the enclosing function scope
    pub fn find_enclosing_function(&self) -> Option<&Scope> {
        let mut scope_id = self.current_scope_id();
        while let Some(id) = scope_id {
            if let Some(scope) = self.get_scope(id) {
                if scope.kind == ScopeKind::Function {
                    return Some(scope);
                }
                scope_id = scope.parent;
            } else {
                break;
            }
        }
        None
    }

    /// Find the enclosing class scope
    pub fn find_enclosing_class(&self) -> Option<&Scope> {
        let mut scope_id = self.current_scope_id();
        while let Some(id) = scope_id {
            if let Some(scope) = self.get_scope(id) {
                if scope.kind == ScopeKind::Class {
                    return Some(scope);
                }
                scope_id = scope.parent;
            } else {
                break;
            }
        }
        None
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
    use crate::arena::ArenaId;

    #[test]
    fn test_scope_creation() {
        let mut chain = ScopeChain::new();
        let global_id = chain.push_scope(ScopeKind::Global);

        assert_eq!(chain.depth(), 1);
        assert_eq!(chain.current_scope_id(), Some(global_id));

        let scope = chain.current_scope().unwrap();
        assert_eq!(scope.kind, ScopeKind::Global);
        assert_eq!(scope.parent, None);
    }

    #[test]
    fn test_nested_scopes() {
        let mut chain = ScopeChain::with_global_scope();

        let fn_id = chain.push_scope(ScopeKind::Function);
        assert_eq!(chain.depth(), 2);

        let block_id = chain.push_scope(ScopeKind::Block);
        assert_eq!(chain.depth(), 3);

        // Block scope should have function as parent
        let block = chain.get_scope(block_id).unwrap();
        assert_eq!(block.parent, Some(fn_id));

        chain.pop_scope();
        assert_eq!(chain.depth(), 2);
        assert_eq!(chain.current_scope_id(), Some(fn_id));
    }

    #[test]
    fn test_container_scope() {
        let mut chain = ScopeChain::with_global_scope();

        // Global is a container
        let global_id = chain.current_scope_id().unwrap();
        assert_eq!(chain.current_container_id(), Some(global_id));

        // Push a function (also a container)
        let fn_id = chain.push_scope(ScopeKind::Function);
        assert_eq!(chain.current_container_id(), Some(fn_id));

        // Push a block (not a container)
        chain.push_scope(ScopeKind::Block);
        // Container should still be the function
        assert_eq!(chain.current_container_id(), Some(fn_id));

        chain.pop_scope();
        chain.pop_scope();
        // Back to global
        assert_eq!(chain.current_container_id(), Some(global_id));
    }

    #[test]
    fn test_symbol_lookup() {
        let mut chain = ScopeChain::with_global_scope();

        // Add a symbol to global scope
        let symbol_id = ArenaId::null();
        chain.current_scope_mut().unwrap().locals.set("globalVar", symbol_id);

        // Push function scope
        chain.push_scope(ScopeKind::Function);

        // Should find globalVar
        assert!(chain.lookup("globalVar").is_some());
        assert!(chain.lookup("nonExistent").is_none());

        // Add local symbol
        let local_id = ArenaId::null();
        chain.current_scope_mut().unwrap().locals.set("localVar", local_id);

        // Should find both
        assert!(chain.lookup("globalVar").is_some());
        assert!(chain.lookup("localVar").is_some());

        chain.pop_scope();

        // Should not find localVar anymore
        assert!(chain.lookup("globalVar").is_some());
        assert!(chain.lookup("localVar").is_none());
    }

    #[test]
    fn test_strict_mode_inheritance() {
        let mut chain = ScopeChain::with_global_scope();
        assert!(!chain.is_strict_mode());

        chain.set_strict_mode(true);
        assert!(chain.is_strict_mode());

        // Push function scope - should inherit strict mode
        chain.push_scope(ScopeKind::Function);
        assert!(chain.is_strict_mode());
    }
}
