//! Parallel Processing Module
//!
//! Provides parallel file parsing and processing using Rayon.
//! This enables significant speedups on multi-core machines.
//!
//! # Architecture
//!
//! The compilation pipeline has these parallelization opportunities:
//!
//! 1. **Parsing** - Each file can be parsed independently (embarrassingly parallel)
//! 2. **Binding** - After parsing, binding can be parallelized per-file
//! 3. **Type Checking** - Function bodies can be checked in parallel
//!    (once global symbols are merged)
//!
//! # Usage
//!
//! ```rust,ignore
//! use wasm::parallel::parse_files_parallel;
//!
//! let files = vec![
//!     ("src/a.ts".to_string(), "let a = 1;".to_string()),
//!     ("src/b.ts".to_string(), "let b = 2;".to_string()),
//! ];
//!
//! let results = parse_files_parallel(files);
//! // results is Vec<ParseResult> with parsed ASTs
//! ```

use rayon::prelude::*;
use std::sync::Arc;
use crate::thin_parser::{ParseDiagnostic, ThinParserState};
use crate::thin_binder::ThinBinderState;
use crate::binder::{Scope, ScopeId, SymbolArena, SymbolId, SymbolTable};
use crate::parser::NodeIndex;
use crate::parser::thin_node::ThinNodeArena;
use rustc_hash::FxHashMap;

/// Result of parsing a single file
pub struct ParseResult {
    /// File name
    pub file_name: String,
    /// The parsed source file node index
    pub source_file: NodeIndex,
    /// The arena containing all nodes
    pub arena: ThinNodeArena,
    /// Parse diagnostics
    pub parse_diagnostics: Vec<ParseDiagnostic>,
}

/// Parse multiple files in parallel using ThinParser
///
/// Each file is parsed independently, producing its own arena.
/// This is optimal for initial parsing before symbol resolution.
///
/// # Arguments
/// * `files` - Vector of (file_name, source_text) pairs
///
/// # Returns
/// Vector of ParseResult for each file
pub fn parse_files_parallel(files: Vec<(String, String)>) -> Vec<ParseResult> {
    files
        .into_par_iter()
        .map(|(file_name, source_text)| {
            let mut parser = ThinParserState::new(file_name.clone(), source_text);
            let source_file = parser.parse_source_file();

            // Consume the parser and take its arena/diagnostics
            let (arena, parse_diagnostics) = parser.into_parts();

            ParseResult {
                file_name,
                source_file,
                arena,
                parse_diagnostics,
            }
        })
        .collect()
}

/// Parse a single file (for comparison/testing)
pub fn parse_file_single(file_name: String, source_text: String) -> ParseResult {
    let mut parser = ThinParserState::new(file_name.clone(), source_text);
    let source_file = parser.parse_source_file();

    // Consume the parser and take its arena/diagnostics
    let (arena, parse_diagnostics) = parser.into_parts();

    ParseResult {
        file_name,
        source_file,
        arena,
        parse_diagnostics,
    }
}

/// Statistics about parallel parsing performance
#[derive(Debug, Clone)]
pub struct ParallelStats {
    /// Number of files parsed
    pub file_count: usize,
    /// Total source bytes
    pub total_bytes: usize,
    /// Total nodes created
    pub total_nodes: usize,
    /// Number of parse errors
    pub error_count: usize,
}

// =============================================================================
// Parallel Binding
// =============================================================================

/// Result of binding a single file
pub struct BindResult {
    /// File name
    pub file_name: String,
    /// The parsed source file node index
    pub source_file: NodeIndex,
    /// The arena containing all nodes
    pub arena: Arc<ThinNodeArena>,
    /// Symbols created in this file
    pub symbols: SymbolArena,
    /// File-level symbol table (exports, declarations)
    pub file_locals: SymbolTable,
    /// Node-to-symbol mapping
    pub node_symbols: FxHashMap<u32, SymbolId>,
    /// Persistent scopes for stateless checking
    pub scopes: Vec<Scope>,
    /// Map from AST node to scope ID
    pub node_scope_ids: FxHashMap<u32, ScopeId>,
    /// Parse diagnostics
    pub parse_diagnostics: Vec<ParseDiagnostic>,
}

/// Parse and bind multiple files in parallel
///
/// Each file is parsed and bound independently. The binding creates
/// file-local symbols which can later be merged into a global scope.
///
/// # Arguments
/// * `files` - Vector of (file_name, source_text) pairs
///
/// # Returns
/// Vector of BindResult for each file
pub fn parse_and_bind_parallel(files: Vec<(String, String)>) -> Vec<BindResult> {
    files
        .into_par_iter()
        .map(|(file_name, source_text)| {
            // Parse
            let mut parser = ThinParserState::new(file_name.clone(), source_text);
            let source_file = parser.parse_source_file();

            let (arena, parse_diagnostics) = parser.into_parts();

            // Bind
            let mut binder = ThinBinderState::new();
            binder.bind_source_file(&arena, source_file);

            BindResult {
                file_name,
                source_file,
                arena: Arc::new(arena),
                symbols: binder.symbols,
                file_locals: binder.file_locals,
                node_symbols: binder.node_symbols,
                scopes: binder.scopes,
                node_scope_ids: binder.node_scope_ids,
                parse_diagnostics,
            }
        })
        .collect()
}

/// Bind a single file (for comparison/testing)
pub fn parse_and_bind_single(file_name: String, source_text: String) -> BindResult {
    let mut parser = ThinParserState::new(file_name.clone(), source_text);
    let source_file = parser.parse_source_file();

    let (arena, parse_diagnostics) = parser.into_parts();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(&arena, source_file);

    BindResult {
        file_name,
        source_file,
        arena: Arc::new(arena),
        symbols: binder.symbols,
        file_locals: binder.file_locals,
        node_symbols: binder.node_symbols,
        scopes: binder.scopes,
        node_scope_ids: binder.node_scope_ids,
        parse_diagnostics,
    }
}

/// Statistics about parallel binding performance
#[derive(Debug, Clone)]
pub struct BindStats {
    /// Number of files bound
    pub file_count: usize,
    /// Total nodes across all files
    pub total_nodes: usize,
    /// Total symbols created
    pub total_symbols: usize,
    /// Number of parse errors
    pub parse_error_count: usize,
}

/// Parse and bind files with statistics
pub fn parse_and_bind_with_stats(files: Vec<(String, String)>) -> (Vec<BindResult>, BindStats) {
    let file_count = files.len();
    let results = parse_and_bind_parallel(files);

    let total_nodes: usize = results.iter().map(|r| r.arena.len()).sum();
    let total_symbols: usize = results.iter().map(|r| r.symbols.len()).sum();
    let parse_error_count: usize = results.iter().map(|r| r.parse_diagnostics.len()).sum();

    let stats = BindStats {
        file_count,
        total_nodes,
        total_symbols,
        parse_error_count,
    };

    (results, stats)
}

// =============================================================================
// Symbol Merging
// =============================================================================

/// A bound file ready for type checking
pub struct BoundFile {
    /// File name
    pub file_name: String,
    /// The parsed source file node index
    pub source_file: NodeIndex,
    /// The arena containing all nodes (owned by this file)
    pub arena: Arc<ThinNodeArena>,
    /// Node-to-symbol mapping (symbol IDs are global after merge)
    pub node_symbols: FxHashMap<u32, SymbolId>,
    /// Persistent scopes (symbol IDs are global after merge)
    pub scopes: Vec<Scope>,
    /// Map from AST node to scope ID
    pub node_scope_ids: FxHashMap<u32, ScopeId>,
    /// Parse diagnostics
    pub parse_diagnostics: Vec<ParseDiagnostic>,
}

use crate::solver::TypeInterner;

/// Merged program state after parallel binding
pub struct MergedProgram {
    /// All bound files
    pub files: Vec<BoundFile>,
    /// Global symbol arena (all symbols from all files, with remapped IDs)
    pub symbols: SymbolArena,
    /// Symbol-to-arena mapping for declaration lookup
    pub symbol_arenas: FxHashMap<SymbolId, Arc<ThinNodeArena>>,
    /// Global symbol table (exports from all files)
    pub globals: SymbolTable,
    /// Per-file symbol tables (file-local symbols, symbol IDs remapped)
    pub file_locals: Vec<SymbolTable>,
    /// Global type interner - shared across all threads for type deduplication
    pub type_interner: TypeInterner,
}

/// Merge bind results into a unified program state
///
/// This is a sequential operation that combines:
/// - All symbol arenas into a single global arena
/// - All file_locals into the global scope (for now, simple merge)
/// - Remaps symbol IDs in node_symbols to use global IDs
///
/// # Arguments
/// * `results` - Vector of BindResult from parallel binding
///
/// # Returns
/// MergedProgram with unified symbol space
pub fn merge_bind_results(results: Vec<BindResult>) -> MergedProgram {
    let refs: Vec<&BindResult> = results.iter().collect();
    merge_bind_results_ref(&refs)
}

pub fn merge_bind_results_ref(results: &[&BindResult]) -> MergedProgram {
    // Calculate total symbols needed
    let total_symbols: usize = results.iter().map(|r| r.symbols.len()).sum();

    // Create global symbol arena with pre-allocated capacity
    let mut global_symbols = SymbolArena::with_capacity(total_symbols);
    let mut symbol_arenas = FxHashMap::default();
    let mut globals = SymbolTable::new();
    let mut files = Vec::with_capacity(results.len());
    let mut file_locals_list = Vec::with_capacity(results.len());

    for result in results {
        // Copy symbols from this file to global arena, getting new IDs
        let mut id_remap: FxHashMap<SymbolId, SymbolId> = FxHashMap::default();
        for i in 0..result.symbols.len() {
            let old_id = SymbolId(i as u32);
            if let Some(sym) = result.symbols.get(old_id) {
                let new_id = global_symbols.alloc(sym.flags, sym.escaped_name.clone());
                id_remap.insert(old_id, new_id);
                symbol_arenas.insert(new_id, Arc::clone(&result.arena));
            }
        }

        let remap_symbol_table = |table: &SymbolTable,
                                  id_remap: &FxHashMap<SymbolId, SymbolId>|
         -> SymbolTable {
            let mut remapped = SymbolTable::new();
            for (name, old_sym_id) in table.iter() {
                if let Some(&new_sym_id) = id_remap.get(old_sym_id) {
                    remapped.set(name.clone(), new_sym_id);
                }
            }
            remapped
        };

        for (old_id, &new_id) in id_remap.iter() {
            let Some(old_sym) = result.symbols.get(*old_id) else {
                continue;
            };
            if let Some(new_sym) = global_symbols.get_mut(new_id) {
                let mut updated = old_sym.clone();
                updated.id = new_id;
                updated.parent = id_remap.get(&old_sym.parent).copied().unwrap_or(SymbolId::NONE);
                updated.value_declaration = old_sym.value_declaration;
                updated.declarations = old_sym.declarations.clone();
                updated.is_exported = old_sym.is_exported;
                updated.exports = old_sym.exports.as_ref().map(|table| {
                    Box::new(remap_symbol_table(table.as_ref(), &id_remap))
                });
                updated.members = old_sym.members.as_ref().map(|table| {
                    Box::new(remap_symbol_table(table.as_ref(), &id_remap))
                });
                *new_sym = updated;
            }
        }

        // Remap node_symbols to use global IDs
        let mut remapped_node_symbols = FxHashMap::default();
        for (node_idx, old_sym_id) in result.node_symbols.iter() {
            if let Some(&new_sym_id) = id_remap.get(old_sym_id) {
                remapped_node_symbols.insert(*node_idx, new_sym_id);
            }
        }

        // Remap file_locals to use global IDs
        let mut remapped_file_locals = SymbolTable::new();
        for (name, old_sym_id) in result.file_locals.iter() {
            if let Some(&new_sym_id) = id_remap.get(old_sym_id) {
                remapped_file_locals.set(name.clone(), new_sym_id);
                // Also add to globals (all top-level declarations visible globally)
                globals.set(name.clone(), new_sym_id);
            }
        }

        let mut remapped_scopes = Vec::with_capacity(result.scopes.len());
        for scope in &result.scopes {
            let mut table = SymbolTable::new();
            for (name, old_sym_id) in scope.table.iter() {
                if let Some(&new_sym_id) = id_remap.get(old_sym_id) {
                    table.set(name.clone(), new_sym_id);
                }
            }
            remapped_scopes.push(Scope {
                parent: scope.parent,
                table,
                kind: scope.kind,
                container_node: scope.container_node,
            });
        }

        file_locals_list.push(remapped_file_locals);

        files.push(BoundFile {
            file_name: result.file_name.clone(),
            source_file: result.source_file,
            arena: Arc::clone(&result.arena),
            node_symbols: remapped_node_symbols,
            scopes: remapped_scopes,
            node_scope_ids: result.node_scope_ids.clone(),
            parse_diagnostics: result.parse_diagnostics.clone(),
        });
    }

    MergedProgram {
        files,
        symbols: global_symbols,
        symbol_arenas,
        globals,
        file_locals: file_locals_list,
        type_interner: TypeInterner::new(),
    }
}

/// Full pipeline: Parse → Bind (parallel) → Merge (sequential)
///
/// This is the main entry point for multi-file compilation.
pub fn compile_files(files: Vec<(String, String)>) -> MergedProgram {
    let bind_results = parse_and_bind_parallel(files);
    merge_bind_results(bind_results)
}

// =============================================================================
// Parallel Type Checking
// =============================================================================

use crate::thin_checker::ThinCheckerState;
use crate::solver::TypeId;
use crate::checker::types::diagnostics::Diagnostic;
use crate::parser::syntax_kind_ext;

/// Result of type checking a single function body
#[derive(Debug)]
pub struct FunctionCheckResult {
    /// Function node index within its file
    pub function_idx: NodeIndex,
    /// File index in the program
    pub file_idx: usize,
    /// Inferred return type
    pub return_type: TypeId,
    /// Diagnostics produced
    pub diagnostics: Vec<Diagnostic>,
}

/// Result of type checking all function bodies in a file
pub struct FileCheckResult {
    /// File index
    pub file_idx: usize,
    /// File name
    pub file_name: String,
    /// Function check results
    pub function_results: Vec<FunctionCheckResult>,
    /// File-level diagnostics
    pub diagnostics: Vec<Diagnostic>,
}

/// Result of parallel type checking
pub struct CheckResult {
    /// Per-file check results
    pub file_results: Vec<FileCheckResult>,
    /// Total functions checked
    pub function_count: usize,
    /// Total diagnostics
    pub diagnostic_count: usize,
}

/// Collect all function declarations from a source file
fn collect_functions(arena: &ThinNodeArena, source_file: NodeIndex) -> Vec<NodeIndex> {
    let mut functions = Vec::new();

    let Some(node) = arena.get(source_file) else {
        return functions;
    };

    let Some(sf) = arena.get_source_file(node) else {
        return functions;
    };

    for &stmt_idx in &sf.statements.nodes {
        collect_functions_from_node(arena, stmt_idx, &mut functions);
    }

    functions
}

/// Recursively collect functions from a node
fn collect_functions_from_node(arena: &ThinNodeArena, node_idx: NodeIndex, functions: &mut Vec<NodeIndex>) {
    let Some(node) = arena.get(node_idx) else {
        return;
    };

    match node.kind {
        k if k == syntax_kind_ext::FUNCTION_DECLARATION ||
             k == syntax_kind_ext::FUNCTION_EXPRESSION ||
             k == syntax_kind_ext::ARROW_FUNCTION => {
            functions.push(node_idx);
            // Also collect nested functions in the body
            if let Some(func) = arena.get_function(node) {
                if !func.body.is_none() {
                    collect_functions_from_node(arena, func.body, functions);
                }
            }
        }
        k if k == syntax_kind_ext::METHOD_DECLARATION => {
            functions.push(node_idx);
            // Also collect nested functions in the body
            if let Some(method) = arena.get_method_decl(node) {
                if !method.body.is_none() {
                    collect_functions_from_node(arena, method.body, functions);
                }
            }
        }
        k if k == syntax_kind_ext::CLASS_DECLARATION => {
            if let Some(class) = arena.get_class(node) {
                for &member_idx in &class.members.nodes {
                    collect_functions_from_node(arena, member_idx, functions);
                }
            }
        }
        k if k == syntax_kind_ext::BLOCK => {
            if let Some(block) = arena.get_block(node) {
                for &stmt_idx in &block.statements.nodes {
                    collect_functions_from_node(arena, stmt_idx, &mut *functions);
                }
            }
        }
        k if k == syntax_kind_ext::VARIABLE_STATEMENT => {
            // Variable statement contains a declaration list which contains declarations
            if let Some(var_stmt) = arena.get_variable(node) {
                // var_stmt.declarations contains the VARIABLE_DECLARATION_LIST node(s)
                for &decl_list_idx in &var_stmt.declarations.nodes {
                    if let Some(decl_list_node) = arena.get(decl_list_idx) {
                        // The declaration list also uses VariableData
                        if let Some(decl_list) = arena.get_variable(decl_list_node) {
                            // Now decl_list.declarations contains the actual VARIABLE_DECLARATION nodes
                            for &decl_idx in &decl_list.declarations.nodes {
                                if let Some(decl_node) = arena.get(decl_idx) {
                                    if let Some(decl) = arena.get_variable_declaration(decl_node) {
                                        if !decl.initializer.is_none() {
                                            collect_functions_from_node(arena, decl.initializer, functions);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        k if k == syntax_kind_ext::EXPORT_DECLARATION => {
            // Export declarations may contain function/class declarations
            if let Some(export) = arena.get_export_decl(node) {
                if !export.export_clause.is_none() {
                    collect_functions_from_node(arena, export.export_clause, functions);
                }
            }
        }
        _ => {}
    }
}

/// Type check function bodies in parallel
///
/// After binding is complete and symbols are merged, function bodies
/// can be type-checked in parallel because:
/// 1. Each function body only uses local variables and global symbols
/// 2. Local type inference doesn't modify global state
/// 3. Each function is independent
///
/// # Arguments
/// * `program` - The merged program with global symbols
///
/// # Returns
/// CheckResult with diagnostics from all functions
pub fn check_functions_parallel(program: &MergedProgram) -> CheckResult {
    // First, collect all functions from all files (sequential)
    let mut all_functions: Vec<(usize, NodeIndex)> = Vec::new();

    for (file_idx, file) in program.files.iter().enumerate() {
        let functions = collect_functions(&file.arena, file.source_file);
        for func_idx in functions {
            all_functions.push((file_idx, func_idx));
        }
    }

    let function_count = all_functions.len();

    // Check functions in parallel
    // Note: We need to be careful here - ThinCheckerState holds mutable references
    // For now, we group by file and check each file's functions together
    let file_results: Vec<FileCheckResult> = program.files
        .par_iter()
        .enumerate()
        .map(|(file_idx, file)| {
            let functions = collect_functions(&file.arena, file.source_file);

            // Create a binder state from the node_symbols
            let binder = create_binder_from_bound_file(file, program, file_idx);

            // Create checker for this file, using the shared type interner
            let mut checker = ThinCheckerState::new(
                &file.arena,
                &binder,
                &program.type_interner,
                file.file_name.clone(),
            );

            let mut function_results = Vec::new();

            for func_idx in functions {
                // Check the function
                let return_type = checker.get_type_of_node(func_idx);

                function_results.push(FunctionCheckResult {
                    function_idx: func_idx,
                    file_idx,
                    return_type,
                    diagnostics: Vec::new(), // Diagnostics are collected at file level
                });
            }

            // Collect diagnostics from checker
            let diagnostics = std::mem::take(&mut checker.ctx.diagnostics);

            FileCheckResult {
                file_idx,
                file_name: file.file_name.clone(),
                function_results,
                diagnostics,
            }
        })
        .collect();

    let diagnostic_count: usize = file_results.iter()
        .map(|r| r.diagnostics.len())
        .sum();

    CheckResult {
        file_results,
        function_count,
        diagnostic_count,
    }
}

/// Create a ThinBinderState from a BoundFile for type checking
fn create_binder_from_bound_file(file: &BoundFile, program: &MergedProgram, file_idx: usize) -> ThinBinderState {
    // Get file locals for this specific file
    let mut file_locals = SymbolTable::new();

    // Copy from program.file_locals if available
    if file_idx < program.file_locals.len() {
        for (name, &sym_id) in program.file_locals[file_idx].iter() {
            file_locals.set(name.clone(), sym_id);
        }
    }

    // Also add globals (for cross-file references)
    for (name, &sym_id) in program.globals.iter() {
        if !file_locals.has(name) {
            file_locals.set(name.clone(), sym_id);
        }
    }

    let mut binder = ThinBinderState::from_bound_state_with_scopes(
        program.symbols.clone(),
        file_locals,
        file.node_symbols.clone(),
        file.scopes.clone(),
        file.node_scope_ids.clone(),
    );

    binder.symbol_arenas = program.symbol_arenas.clone();
    binder
}

/// Check function bodies with statistics
pub fn check_functions_with_stats(program: &MergedProgram) -> (CheckResult, CheckStats) {
    let result = check_functions_parallel(program);

    let stats = CheckStats {
        file_count: result.file_results.len(),
        function_count: result.function_count,
        diagnostic_count: result.diagnostic_count,
    };

    (result, stats)
}

/// Statistics about parallel type checking
#[derive(Debug, Clone)]
pub struct CheckStats {
    /// Number of files checked
    pub file_count: usize,
    /// Number of functions checked
    pub function_count: usize,
    /// Number of diagnostics produced
    pub diagnostic_count: usize,
}

/// Parse files and collect statistics
pub fn parse_files_with_stats(files: Vec<(String, String)>) -> (Vec<ParseResult>, ParallelStats) {
    let total_bytes: usize = files.iter().map(|(_, src)| src.len()).sum();
    let file_count = files.len();

    let results = parse_files_parallel(files);

    let total_nodes: usize = results.iter().map(|r| r.arena.len()).sum();
    let error_count: usize = results.iter().map(|r| r.parse_diagnostics.len()).sum();

    let stats = ParallelStats {
        file_count,
        total_bytes,
        total_nodes,
        error_count,
    };

    (results, stats)
}

#[cfg(test)]
#[path = "parallel_tests.rs"]
mod tests;
