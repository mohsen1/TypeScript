//! Checker Context
//!
//! Holds the shared state used throughout the type checking process.
//! This separates state from logic, allowing specialized checkers (expressions, statements)
//! to borrow the context mutably.

use std::collections::{HashMap, HashSet, VecDeque};
use std::cell::RefCell;
use std::sync::Arc;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::parser::NodeIndex;
use crate::parser::thin_node::ThinNodeArena;
use crate::thin_binder::ThinBinderState;
use crate::solver::{TypeEnvironment, TypeId, TypeInterner};
use crate::checker::types::diagnostics::Diagnostic;
use crate::binder::SymbolId;

/// Info about the enclosing class for static member suggestions and abstract property checks.
#[derive(Clone, Debug)]
pub struct EnclosingClassInfo {
    /// Name of the class.
    pub name: String,
    /// Node index for the class declaration.
    pub class_idx: NodeIndex,
    /// Member node indices for symbol lookup.
    pub member_nodes: Vec<NodeIndex>,
    /// Whether we're in a constructor (for error 2715 checking).
    pub in_constructor: bool,
    /// Whether this is a `declare class` (ambient context for error 1183).
    pub is_declared: bool,
}

/// Persistent cache for type checking results across LSP queries.
/// This cache survives between LSP requests but is invalidated when the file changes.
#[derive(Clone, Debug, Default)]
pub struct TypeCache {
    /// Cached types for symbols.
    pub symbol_types: FxHashMap<SymbolId, TypeId>,

    /// Cached types for nodes.
    pub node_types: FxHashMap<u32, TypeId>,

    /// Type parameter names for type_to_string.
    pub type_parameter_names: FxHashMap<TypeId, String>,

    /// Cache for type relation results (subtype checking).
    pub relation_cache: FxHashMap<(TypeId, TypeId, u8), bool>,

    /// Symbol dependency graph (symbol -> referenced symbols).
    pub symbol_dependencies: FxHashMap<SymbolId, FxHashSet<SymbolId>>,
}

impl TypeCache {
    /// Invalidate cached symbol types that depend on the provided roots.
    /// Returns the number of affected symbols.
    pub fn invalidate_symbols(&mut self, roots: &[SymbolId]) -> usize {
        if roots.is_empty() {
            return 0;
        }

        let mut reverse: FxHashMap<SymbolId, Vec<SymbolId>> = FxHashMap::default();
        for (symbol, deps) in &self.symbol_dependencies {
            for dep in deps {
                reverse.entry(*dep).or_default().push(*symbol);
            }
        }

        let mut affected: FxHashSet<SymbolId> = FxHashSet::default();
        let mut pending = VecDeque::new();
        for &root in roots {
            if affected.insert(root) {
                pending.push_back(root);
            }
        }

        while let Some(sym_id) = pending.pop_front() {
            if let Some(dependents) = reverse.get(&sym_id) {
                for &dependent in dependents {
                    if affected.insert(dependent) {
                        pending.push_back(dependent);
                    }
                }
            }
        }

        for sym_id in &affected {
            self.symbol_types.remove(sym_id);
            self.symbol_dependencies.remove(sym_id);
        }
        self.node_types.clear();

        affected.len()
    }
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

    /// Cached type environment for resolving Ref types during assignability checks.
    pub type_environment: RefCell<Option<TypeEnvironment>>,

    /// Symbol dependency graph (symbol -> referenced symbols).
    pub symbol_dependencies: FxHashMap<SymbolId, FxHashSet<SymbolId>>,

    /// Stack of symbols currently being evaluated for dependency tracking.
    pub symbol_dependency_stack: Vec<SymbolId>,

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

    /// Stack of expected return types for functions.
    pub return_type_stack: Vec<TypeId>,

    /// Current enclosing class info.
    pub enclosing_class: Option<EnclosingClassInfo>,

    /// Type environment for symbol resolution with type parameters.
    /// Used by the evaluator to expand Application types.
    pub type_env: RefCell<TypeEnvironment>,

    /// All arenas for cross-file resolution (indexed by file_idx from Symbol.decl_file_idx).
    /// Set during multi-file type checking to allow resolving declarations across files.
    pub all_arenas: Option<Vec<Arc<ThinNodeArena>>>,

    /// Lib file contexts for global type resolution (lib.es5.d.ts, lib.dom.d.ts, etc.).
    /// Each entry is a (arena, binder) pair from a pre-parsed lib file.
    /// Used as a fallback when resolving type references not found in the main file.
    pub lib_contexts: Vec<LibContext>,
}

/// Context for a lib file (arena + binder) for global type resolution.
pub struct LibContext {
    /// The AST arena for this lib file.
    pub arena: Arc<ThinNodeArena>,
    /// The binder state with symbols from this lib file.
    pub binder: Arc<ThinBinderState>,
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
            type_environment: RefCell::new(None),
            symbol_dependencies: FxHashMap::default(),
            symbol_dependency_stack: Vec::new(),
            diagnostics: Vec::new(),
            symbol_resolution_stack: Vec::new(),
            symbol_resolution_set: HashSet::new(),
            node_resolution_stack: Vec::new(),
            node_resolution_set: HashSet::new(),
            type_parameter_scope: HashMap::new(),
            contextual_type: None,
            instantiation_depth: RefCell::new(0),
            call_depth: RefCell::new(0),
            return_type_stack: Vec::new(),
            enclosing_class: None,
            type_env: RefCell::new(TypeEnvironment::new()),
            all_arenas: None,
            lib_contexts: Vec::new(),
        }
    }

    /// Create a new CheckerContext with a persistent cache.
    /// This allows reusing type checking results from previous queries.
    pub fn with_cache(
        arena: &'a ThinNodeArena,
        binder: &'a ThinBinderState,
        types: &'a TypeInterner,
        file_name: String,
        cache: TypeCache,
    ) -> Self {
        CheckerContext {
            arena,
            binder,
            types,
            file_name,
            symbol_types: cache.symbol_types,
            node_types: cache.node_types,
            type_parameter_names: cache.type_parameter_names,
            relation_cache: RefCell::new(cache.relation_cache),
            type_environment: RefCell::new(None),
            symbol_dependencies: cache.symbol_dependencies,
            symbol_dependency_stack: Vec::new(),
            diagnostics: Vec::new(),
            symbol_resolution_stack: Vec::new(),
            symbol_resolution_set: HashSet::new(),
            node_resolution_stack: Vec::new(),
            node_resolution_set: HashSet::new(),
            type_parameter_scope: HashMap::new(),
            contextual_type: None,
            instantiation_depth: RefCell::new(0),
            call_depth: RefCell::new(0),
            return_type_stack: Vec::new(),
            enclosing_class: None,
            type_env: RefCell::new(TypeEnvironment::new()),
            all_arenas: None,
            lib_contexts: Vec::new(),
        }
    }

    /// Set lib contexts for global type resolution.
    pub fn set_lib_contexts(&mut self, lib_contexts: Vec<LibContext>) {
        self.lib_contexts = lib_contexts;
    }

    /// Set all arenas for cross-file resolution.
    pub fn set_all_arenas(&mut self, arenas: Vec<Arc<ThinNodeArena>>) {
        self.all_arenas = Some(arenas);
    }

    /// Get the arena for a specific file index.
    /// Returns the current arena if file_idx is u32::MAX (single-file mode).
    pub fn get_arena_for_file(&self, file_idx: u32) -> &ThinNodeArena {
        if file_idx == u32::MAX {
            return self.arena;
        }
        if let Some(ref arenas) = self.all_arenas {
            if let Some(arena) = arenas.get(file_idx as usize) {
                return arena.as_ref();
            }
        }
        self.arena
    }

    /// Extract the persistent cache from this context.
    /// This allows saving type checking results for future queries.
    pub fn extract_cache(self) -> TypeCache {
        TypeCache {
            symbol_types: self.symbol_types,
            node_types: self.node_types,
            type_parameter_names: self.type_parameter_names,
            relation_cache: self.relation_cache.into_inner(),
            symbol_dependencies: self.symbol_dependencies,
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

    /// Check if a modifier list contains a specific modifier kind.
    pub fn has_modifier(&self, modifiers: &Option<crate::parser::NodeList>, kind: u16) -> bool {
        if let Some(mods) = modifiers {
            for &idx in &mods.nodes {
                if let Some(node) = self.arena.get(idx) {
                    if node.kind == kind {
                        return true;
                    }
                }
            }
        }
        false
    }
}
