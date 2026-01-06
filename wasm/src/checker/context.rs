//! Checker Context
//!
//! Holds the shared state used throughout the type checking process.
//! This separates state from logic, allowing specialized checkers (expressions, statements)
//! to borrow the context mutably.

use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use rustc_hash::FxHashMap;

use crate::parser::NodeIndex;
use crate::parser::thin_node::ThinNodeArena;
use crate::thin_binder::ThinBinderState;
use crate::solver::{TypeId, TypeInterner};
use crate::checker::types::diagnostics::Diagnostic;
use crate::binder::SymbolId;

/// Info about the enclosing class for static member suggestions and abstract property checks.
#[derive(Clone, Debug)]
pub struct EnclosingClassInfo {
    /// Name of the class.
    pub name: String,
    /// Member node indices for symbol lookup.
    pub member_nodes: Vec<NodeIndex>,
    /// Whether we're in a constructor (for error 2715 checking).
    pub in_constructor: bool,
    /// Whether this is a `declare class` (ambient context for error 1183).
    pub is_declared: bool,
}

/// Shared state for type checking.
pub struct CheckerContext<'a> {
    /// The ThinNodeArena containing the AST.
    pub arena: &'a ThinNodeArena,

    /// The binder state with symbols.
    pub binder: &'a ThinBinderState,

    /// Type interner for structural type interning.
    pub types: &'a TypeInterner,

    /// Current file name.
    pub file_name: String,

    // --- Caches ---

    /// Cached types for symbols.
    pub symbol_types: FxHashMap<SymbolId, TypeId>,

    /// Cached types for nodes.
    pub node_types: FxHashMap<u32, TypeId>,

    /// Type parameter names for type_to_string.
    pub type_parameter_names: FxHashMap<TypeId, String>,

    /// Cache for type relation results.
    pub relation_cache: RefCell<FxHashMap<(TypeId, TypeId, u8), bool>>,

    // --- Diagnostics ---

    /// Diagnostics produced during type checking.
    pub diagnostics: Vec<Diagnostic>,

    // --- Recursion Guards ---

    /// Stack of symbols being resolved.
    pub symbol_resolution_stack: Vec<SymbolId>,
    /// O(1) lookup set for symbol resolution stack.
    pub symbol_resolution_set: HashSet<SymbolId>,

    /// Stack of nodes being resolved.
    pub node_resolution_stack: Vec<NodeIndex>,
    /// O(1) lookup set for node resolution stack.
    pub node_resolution_set: HashSet<NodeIndex>,

    // --- Scopes & Context ---

    /// Current type parameter scope.
    pub type_parameter_scope: HashMap<String, TypeId>,

    /// Contextual type for expression being checked.
    pub contextual_type: Option<TypeId>,

    /// Current depth of recursive type instantiation.
    pub instantiation_depth: RefCell<u32>,

    /// Current depth of call expression resolution.
    pub call_depth: RefCell<u32>,

    /// Stack of local scopes for function parameters and block-scoped variables.
    pub local_scope_stack: Vec<FxHashMap<String, TypeId>>,

    /// Stack of expected return types for functions.
    pub return_type_stack: Vec<TypeId>,

    /// Current enclosing class info.
    pub enclosing_class: Option<EnclosingClassInfo>,
}

impl<'a> CheckerContext<'a> {
    /// Create a new CheckerContext.
    pub fn new(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        types: &'a TypeInterner,
        file_name: String,
    ) -> Self {
        CheckerContext {
            arena,
            binder,
            types,
            file_name,
            symbol_types: FxHashMap::default(),
            node_types: FxHashMap::default(),
            type_parameter_names: FxHashMap::default(),
            relation_cache: RefCell::new(FxHashMap::default()),
            diagnostics: Vec::new(),
            symbol_resolution_stack: Vec::new(),
            symbol_resolution_set: HashSet::new(),
            node_resolution_stack: Vec::new(),
            node_resolution_set: HashSet::new(),
            type_parameter_scope: HashMap::new(),
            contextual_type: None,
            instantiation_depth: RefCell::new(0),
            call_depth: RefCell::new(0),
            local_scope_stack: Vec::new(),
            return_type_stack: Vec::new(),
            enclosing_class: None,
        }
    }

    /// Add an error diagnostic.
    pub fn error(&mut self, start: u32, length: u32, message: String, code: u32) {
        self.diagnostics.push(Diagnostic::error(
            self.file_name.clone(),
            start,
            length,
            message,
            code,
        ));
    }

    /// Get node span (pos, end) from index.
    pub fn get_node_span(&self, idx: NodeIndex) -> Option<(u32, u32)> {
        let node = self.arena.get(idx)?;
        Some((node.pos, node.end))
    }

    // --- Scope Management ---

    /// Push a new local scope.
    pub fn push_local_scope(&mut self) {
        self.local_scope_stack.push(FxHashMap::default());
    }

    /// Pop the current local scope.
    pub fn pop_local_scope(&mut self) {
        self.local_scope_stack.pop();
    }

    /// Add a local variable to the current scope.
    pub fn add_local(&mut self, name: String, type_id: TypeId) {
        if let Some(scope) = self.local_scope_stack.last_mut() {
            scope.insert(name, type_id);
        }
    }

    /// Look up a local variable in the scope stack.
    pub fn lookup_local(&self, name: &str) -> Option<TypeId> {
        for scope in self.local_scope_stack.iter().rev() {
            if let Some(&type_id) = scope.get(name) {
                return Some(type_id);
            }
        }
        None
    }

    /// Push an expected return type onto the stack.
    pub fn push_return_type(&mut self, return_type: TypeId) {
        self.return_type_stack.push(return_type);
    }

    /// Pop the expected return type from the stack.
    pub fn pop_return_type(&mut self) {
        self.return_type_stack.pop();
    }

    /// Get the current expected return type.
    pub fn current_return_type(&self) -> Option<TypeId> {
        self.return_type_stack.last().copied()
    }
}
