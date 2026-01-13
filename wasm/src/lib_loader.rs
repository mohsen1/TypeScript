//! Lib Loader - Load and merge lib.d.ts symbols into the binder.
//!
//! This module provides functionality for loading standard library type definitions
//! (like lib.d.ts) and merging their global symbols into the binder's root scope.
//! This enables proper resolution of built-in types like `Object`, `Function`, `console`, etc.

use crate::binder::{SymbolId, SymbolTable};
use crate::parser::thin_node::ThinNodeArena;
use crate::thin_binder::ThinBinderState;
use std::sync::Arc;

/// Loaded lib file with its arena and binder state.
#[derive(Clone)]
pub struct LibFile {
    /// File name (e.g., "lib.d.ts")
    pub file_name: String,
    /// The arena (shared via Arc for cross-file resolution)
    pub arena: Arc<ThinNodeArena>,
    /// The binder state with bound symbols
    pub binder: Arc<ThinBinderState>,
}

impl LibFile {
    /// Create a new LibFile from a parsed and bound lib file.
    pub fn new(file_name: String, arena: Arc<ThinNodeArena>, binder: Arc<ThinBinderState>) -> Self {
        Self {
            file_name,
            arena,
            binder,
        }
    }

    /// Get the file locals (global symbols) from this lib file.
    pub fn file_locals(&self) -> &SymbolTable {
        &self.binder.file_locals
    }
}

/// Merge lib file symbols into a target symbol table.
///
/// This is called during binder initialization to ensure global symbols
/// from lib.d.ts (like `Object`, `Function`, `console`, etc.) are available
/// during type checking.
pub fn merge_lib_symbols(target: &mut SymbolTable, lib_files: &[Arc<LibFile>]) {
    for lib in lib_files {
        for (name, sym_id) in lib.binder.file_locals.iter() {
            // Only add if not already defined (user code can override lib symbols)
            if !target.has(name) {
                target.set(name.clone(), *sym_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_lib_symbols() {
        use crate::binder::{Symbol, SymbolArena, symbol_flags};

        let mut target = SymbolTable::new();
        let mut arena = SymbolArena::new();

        // Create some lib symbols
        let object_id = arena.alloc(symbol_flags::VALUE, "Object".to_string());
        let function_id = arena.alloc(symbol_flags::VALUE, "Function".to_string());
        let console_id = arena.alloc(symbol_flags::VALUE, "console".to_string());

        // Create a lib binder with these symbols
        let mut lib_file_locals = SymbolTable::new();
        lib_file_locals.set("Object".to_string(), object_id);
        lib_file_locals.set("Function".to_string(), function_id);
        lib_file_locals.set("console".to_string(), console_id);

        let mut lib_binder =
            ThinBinderState::from_bound_state(arena, lib_file_locals, Default::default());

        let lib = Arc::new(LibFile::new(
            "lib.d.ts".to_string(),
            Arc::new(ThinNodeArena::new()),
            Arc::new(lib_binder),
        ));

        // Add a user symbol that should override lib symbol
        let mut user_arena = SymbolArena::new();
        let user_object_id = user_arena.alloc(symbol_flags::VALUE, "Object".to_string());
        target.set("Object".to_string(), user_object_id);

        // Merge lib symbols
        merge_lib_symbols(&mut target, &[lib]);

        // User's Object should not be overridden
        assert_eq!(target.get("Object"), Some(user_object_id));
        // Function and console should be added
        assert_eq!(target.get("Function"), Some(function_id));
        assert_eq!(target.get("console"), Some(console_id));
    }
}
