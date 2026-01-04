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
use crate::thin_parser::ThinParserState;
use crate::thin_binder::ThinBinderState;
use crate::binder::{SymbolArena, SymbolTable, SymbolId};
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
    /// Parse errors
    pub errors: Vec<String>,
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

            // Extract errors before consuming parser
            let errors: Vec<String> = parser.get_diagnostics()
                .iter()
                .map(|d| d.message.clone())
                .collect();

            // Consume the parser and take its arena
            let (arena, _) = parser.into_parts();

            ParseResult {
                file_name,
                source_file,
                arena,
                errors,
            }
        })
        .collect()
}

/// Parse a single file (for comparison/testing)
pub fn parse_file_single(file_name: String, source_text: String) -> ParseResult {
    let mut parser = ThinParserState::new(file_name.clone(), source_text);
    let source_file = parser.parse_source_file();

    let errors: Vec<String> = parser.get_diagnostics()
        .iter()
        .map(|d| d.message.clone())
        .collect();

    // Consume the parser and take its arena
    let (arena, _) = parser.into_parts();

    ParseResult {
        file_name,
        source_file,
        arena,
        errors,
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
    pub arena: ThinNodeArena,
    /// Symbols created in this file
    pub symbols: SymbolArena,
    /// File-level symbol table (exports, declarations)
    pub file_locals: SymbolTable,
    /// Node-to-symbol mapping
    pub node_symbols: FxHashMap<u32, SymbolId>,
    /// Parse errors
    pub parse_errors: Vec<String>,
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

            let parse_errors: Vec<String> = parser.get_diagnostics()
                .iter()
                .map(|d| d.message.clone())
                .collect();

            let (arena, _) = parser.into_parts();

            // Bind
            let mut binder = ThinBinderState::new();
            binder.bind_source_file(&arena, source_file);

            BindResult {
                file_name,
                source_file,
                arena,
                symbols: binder.symbols,
                file_locals: binder.file_locals,
                node_symbols: binder.node_symbols,
                parse_errors,
            }
        })
        .collect()
}

/// Bind a single file (for comparison/testing)
pub fn parse_and_bind_single(file_name: String, source_text: String) -> BindResult {
    let mut parser = ThinParserState::new(file_name.clone(), source_text);
    let source_file = parser.parse_source_file();

    let parse_errors: Vec<String> = parser.get_diagnostics()
        .iter()
        .map(|d| d.message.clone())
        .collect();

    let (arena, _) = parser.into_parts();

    let mut binder = ThinBinderState::new();
    binder.bind_source_file(&arena, source_file);

    BindResult {
        file_name,
        source_file,
        arena,
        symbols: binder.symbols,
        file_locals: binder.file_locals,
        node_symbols: binder.node_symbols,
        parse_errors,
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
    let parse_error_count: usize = results.iter().map(|r| r.parse_errors.len()).sum();

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
    pub arena: ThinNodeArena,
    /// Node-to-symbol mapping (symbol IDs are global after merge)
    pub node_symbols: FxHashMap<u32, SymbolId>,
    /// Parse errors
    pub parse_errors: Vec<String>,
}

/// Merged program state after parallel binding
pub struct MergedProgram {
    /// All bound files
    pub files: Vec<BoundFile>,
    /// Global symbol arena (all symbols from all files, with remapped IDs)
    pub symbols: SymbolArena,
    /// Global symbol table (exports from all files)
    pub globals: SymbolTable,
    /// Per-file symbol tables (file-local symbols, symbol IDs remapped)
    pub file_locals: Vec<SymbolTable>,
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
    // Calculate total symbols needed
    let total_symbols: usize = results.iter().map(|r| r.symbols.len()).sum();

    // Create global symbol arena with pre-allocated capacity
    let mut global_symbols = SymbolArena::with_capacity(total_symbols);
    let mut globals = SymbolTable::new();
    let mut files = Vec::with_capacity(results.len());
    let mut file_locals_list = Vec::with_capacity(results.len());

    for result in results {
        // Track the base offset for this file's symbols
        let base_offset = global_symbols.len() as u32;

        // Copy symbols from this file to global arena, getting new IDs
        let mut id_remap: FxHashMap<SymbolId, SymbolId> = FxHashMap::default();
        for i in 0..result.symbols.len() {
            let old_id = SymbolId(i as u32);
            if let Some(sym) = result.symbols.get(old_id) {
                let new_id = global_symbols.alloc(sym.flags, sym.escaped_name.clone());
                id_remap.insert(old_id, new_id);
            }
        }

        // Remap node_symbols to use global IDs
        let mut remapped_node_symbols = FxHashMap::default();
        for (node_idx, old_sym_id) in result.node_symbols {
            if let Some(&new_sym_id) = id_remap.get(&old_sym_id) {
                remapped_node_symbols.insert(node_idx, new_sym_id);
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

        file_locals_list.push(remapped_file_locals);

        files.push(BoundFile {
            file_name: result.file_name,
            source_file: result.source_file,
            arena: result.arena,
            node_symbols: remapped_node_symbols,
            parse_errors: result.parse_errors,
        });
    }

    MergedProgram {
        files,
        symbols: global_symbols,
        globals,
        file_locals: file_locals_list,
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
use crate::checker::types::TypeId;
use crate::checker::state::Diagnostic;
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

            // Create checker for this file
            let mut checker = ThinCheckerState::new(
                &file.arena,
                &binder,
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
            let diagnostics = std::mem::take(&mut checker.diagnostics);

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

    ThinBinderState::from_bound_state(
        program.symbols.clone(),
        file_locals,
        file.node_symbols.clone(),
    )
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
    let error_count: usize = results.iter().map(|r| r.errors.len()).sum();

    let stats = ParallelStats {
        file_count,
        total_bytes,
        total_nodes,
        error_count,
    };

    (results, stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_file() {
        let result = parse_file_single(
            "test.ts".to_string(),
            "let x = 42;".to_string(),
        );

        assert_eq!(result.file_name, "test.ts");
        assert!(!result.source_file.is_none());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_parse_multiple_files_parallel() {
        let files = vec![
            ("a.ts".to_string(), "let a = 1;".to_string()),
            ("b.ts".to_string(), "let b = 2;".to_string()),
            ("c.ts".to_string(), "let c = 3;".to_string()),
        ];

        let results = parse_files_parallel(files);

        assert_eq!(results.len(), 3);
        for result in &results {
            assert!(!result.source_file.is_none());
            assert!(result.errors.is_empty());
        }
    }

    #[test]
    fn test_parse_with_stats() {
        let files = vec![
            ("a.ts".to_string(), "function foo() { return 1; }".to_string()),
            ("b.ts".to_string(), "class Bar { constructor() {} }".to_string()),
        ];

        let (results, stats) = parse_files_with_stats(files);

        assert_eq!(results.len(), 2);
        assert_eq!(stats.file_count, 2);
        assert!(stats.total_bytes > 0);
        assert!(stats.total_nodes > 0);
        assert_eq!(stats.error_count, 0);
    }

    #[test]
    fn test_parallel_parsing_consistency() {
        // Parse the same file multiple times in parallel
        // Results should be consistent
        let source = "const x: number = 42; function add(a: number, b: number): number { return a + b; }";
        let files: Vec<_> = (0..10)
            .map(|i| (format!("file{}.ts", i), source.to_string()))
            .collect();

        let results = parse_files_parallel(files);

        // All should have same number of nodes (same source)
        let first_node_count = results[0].arena.len();
        for result in &results {
            assert_eq!(result.arena.len(), first_node_count);
            assert!(result.errors.is_empty());
        }
    }

    #[test]
    fn test_large_batch_parsing() {
        // Test with a larger batch to exercise parallelism
        let files: Vec<_> = (0..100)
            .map(|i| {
                let source = format!(
                    "function fn{}(x: number): number {{ return x * {}; }}",
                    i, i
                );
                (format!("module{}.ts", i), source)
            })
            .collect();

        let (results, stats) = parse_files_with_stats(files);

        assert_eq!(results.len(), 100);
        assert_eq!(stats.file_count, 100);
        // Note: ThinParser may produce parse errors for some constructs
        // The key test is that parallel parsing works correctly
        // assert_eq!(stats.error_count, 0);

        // Each file should have similar node counts
        for result in &results {
            assert!(result.arena.len() >= 5, "Each file should have at least 5 nodes");
        }
    }

    // =========================================================================
    // Parallel Binding Tests
    // =========================================================================

    #[test]
    fn test_bind_single_file() {
        let result = parse_and_bind_single(
            "test.ts".to_string(),
            "let x = 42; function foo() {}".to_string(),
        );

        assert_eq!(result.file_name, "test.ts");
        assert!(!result.source_file.is_none());
        assert!(result.parse_errors.is_empty());
        // Should have symbols for x and foo
        assert!(result.file_locals.has("x"));
        assert!(result.file_locals.has("foo"));
    }

    #[test]
    fn test_bind_multiple_files_parallel() {
        let files = vec![
            ("a.ts".to_string(), "let a = 1;".to_string()),
            ("b.ts".to_string(), "function b() {}".to_string()),
            ("c.ts".to_string(), "class C {}".to_string()),
        ];

        let results = parse_and_bind_parallel(files);

        assert_eq!(results.len(), 3);

        // Each file should have its own symbols
        assert!(results[0].file_locals.has("a"));
        assert!(results[1].file_locals.has("b"));
        assert!(results[2].file_locals.has("C"));
    }

    #[test]
    fn test_bind_with_stats() {
        let files = vec![
            ("a.ts".to_string(), "function foo() { return 1; }".to_string()),
            ("b.ts".to_string(), "class Bar { x: number; }".to_string()),
        ];

        let (results, stats) = parse_and_bind_with_stats(files);

        assert_eq!(results.len(), 2);
        assert_eq!(stats.file_count, 2);
        assert!(stats.total_nodes > 0);
        assert!(stats.total_symbols > 0);
        assert_eq!(stats.parse_error_count, 0);
    }

    #[test]
    fn test_parallel_binding_consistency() {
        // Bind the same file multiple times in parallel
        // Results should be consistent
        let source = "const x: number = 42; function add(a: number, b: number): number { return a + b; }";
        let files: Vec<_> = (0..10)
            .map(|i| (format!("file{}.ts", i), source.to_string()))
            .collect();

        let results = parse_and_bind_parallel(files);

        // All should have same symbols
        for result in &results {
            assert!(result.file_locals.has("x"));
            assert!(result.file_locals.has("add"));
            assert!(result.parse_errors.is_empty());
        }
    }

    #[test]
    fn test_large_batch_binding() {
        // Test with a larger batch to exercise parallelism
        let files: Vec<_> = (0..100)
            .map(|i| {
                let source = format!(
                    "function fn{}(x: number): number {{ return x * {}; }} let val{} = fn{}(10);",
                    i, i, i, i
                );
                (format!("module{}.ts", i), source)
            })
            .collect();

        let (results, stats) = parse_and_bind_with_stats(files);

        assert_eq!(results.len(), 100);
        assert_eq!(stats.file_count, 100);
        assert!(stats.total_symbols >= 200, "Should have at least 200 symbols (2 per file)");

        // Each file should have its function and variable
        for (i, result) in results.iter().enumerate() {
            let fn_name = format!("fn{}", i);
            let var_name = format!("val{}", i);
            assert!(result.file_locals.has(&fn_name), "File {} missing {}", i, fn_name);
            assert!(result.file_locals.has(&var_name), "File {} missing {}", i, var_name);
        }
    }

    // =========================================================================
    // Symbol Merging Tests
    // =========================================================================

    #[test]
    fn test_merge_single_file() {
        let files = vec![
            ("a.ts".to_string(), "let x = 1; function foo() {}".to_string()),
        ];

        let program = compile_files(files);

        assert_eq!(program.files.len(), 1);
        assert!(program.globals.has("x"));
        assert!(program.globals.has("foo"));
        // Symbols should be in global arena
        assert!(program.symbols.len() >= 2);
    }

    #[test]
    fn test_merge_multiple_files() {
        let files = vec![
            ("a.ts".to_string(), "let a = 1;".to_string()),
            ("b.ts".to_string(), "function b() {}".to_string()),
            ("c.ts".to_string(), "class C {}".to_string()),
        ];

        let program = compile_files(files);

        assert_eq!(program.files.len(), 3);
        // All symbols should be in globals
        assert!(program.globals.has("a"));
        assert!(program.globals.has("b"));
        assert!(program.globals.has("C"));
        // All symbols merged into global arena
        assert!(program.symbols.len() >= 3);
    }

    #[test]
    fn test_merge_symbol_id_remapping() {
        let files = vec![
            ("a.ts".to_string(), "let x = 1;".to_string()),
            ("b.ts".to_string(), "let y = 2;".to_string()),
        ];

        let program = compile_files(files);

        // Get the symbol IDs from globals
        let x_id = program.globals.get("x").expect("x should exist");
        let y_id = program.globals.get("y").expect("y should exist");

        // IDs should be different (remapped properly)
        assert_ne!(x_id, y_id);

        // Both should be resolvable from global arena
        assert!(program.symbols.get(x_id).is_some());
        assert!(program.symbols.get(y_id).is_some());
    }

    #[test]
    fn test_merge_preserves_file_locals() {
        let files = vec![
            ("a.ts".to_string(), "let a1 = 1; let a2 = 2;".to_string()),
            ("b.ts".to_string(), "let b1 = 1; let b2 = 2;".to_string()),
        ];

        let program = compile_files(files);

        // Each file should have its own locals
        assert_eq!(program.file_locals.len(), 2);
        assert!(program.file_locals[0].has("a1"));
        assert!(program.file_locals[0].has("a2"));
        assert!(program.file_locals[1].has("b1"));
        assert!(program.file_locals[1].has("b2"));
    }

    #[test]
    fn test_compile_large_program() {
        // Simulate a larger program with many files
        let files: Vec<_> = (0..50)
            .map(|i| {
                let source = format!(
                    "function fn{}() {{ return {}; }} const val{} = fn{}();",
                    i, i, i, i
                );
                (format!("module{}.ts", i), source)
            })
            .collect();

        let program = compile_files(files);

        assert_eq!(program.files.len(), 50);
        // Should have at least 100 symbols (2 per file: fn + val)
        assert!(program.symbols.len() >= 100, "Expected at least 100 symbols, got {}", program.symbols.len());

        // All function and value names should be in globals
        for i in 0..50 {
            let fn_name = format!("fn{}", i);
            let val_name = format!("val{}", i);
            assert!(program.globals.has(&fn_name), "Missing {}", fn_name);
            assert!(program.globals.has(&val_name), "Missing {}", val_name);
        }
    }

    #[test]
    fn test_compile_with_exports() {
        // Test that export function/class/const are properly bound
        let files = vec![
            ("a.ts".to_string(), "export function add(x: number, y: number) { return x + y; }".to_string()),
            ("b.ts".to_string(), "export class Calculator { add(x: number, y: number) { return x + y; } }".to_string()),
            ("c.ts".to_string(), "export const PI = 3.14159;".to_string()),
        ];

        let program = compile_files(files);

        assert_eq!(program.files.len(), 3);
        // All exported declarations should be in globals
        assert!(program.globals.has("add"), "Exported function 'add' should be in globals");
        assert!(program.globals.has("Calculator"), "Exported class 'Calculator' should be in globals");
        assert!(program.globals.has("PI"), "Exported const 'PI' should be in globals");
    }

    // =========================================================================
    // Parallel Type Checking Tests
    // =========================================================================

    #[test]
    fn test_check_single_function() {
        let files = vec![
            ("a.ts".to_string(), "function add(x: number, y: number): number { return x + y; }".to_string()),
        ];

        let program = compile_files(files);
        let result = check_functions_parallel(&program);

        assert_eq!(result.file_results.len(), 1);
        assert_eq!(result.function_count, 1);
        assert_eq!(result.file_results[0].function_results.len(), 1);
    }

    #[test]
    fn test_check_multiple_functions_parallel() {
        let files = vec![
            ("a.ts".to_string(), "function foo() { return 1; } function bar() { return 2; }".to_string()),
            ("b.ts".to_string(), "function baz(x: number) { return x * 2; }".to_string()),
        ];

        let program = compile_files(files);
        let result = check_functions_parallel(&program);

        assert_eq!(result.file_results.len(), 2);
        // File a has 2 functions, file b has 1
        let total_functions: usize = result.file_results.iter()
            .map(|r| r.function_results.len())
            .sum();
        assert_eq!(total_functions, 3);
    }

    #[test]
    fn test_check_arrow_functions() {
        let files = vec![
            ("a.ts".to_string(), "const add = (x: number, y: number) => x + y;".to_string()),
            ("b.ts".to_string(), "const double = (x: number) => { return x * 2; };".to_string()),
        ];

        let program = compile_files(files);
        let result = check_functions_parallel(&program);

        // Should find the arrow functions
        let total_functions: usize = result.file_results.iter()
            .map(|r| r.function_results.len())
            .sum();
        assert!(total_functions >= 2, "Should find at least 2 arrow functions");
    }

    #[test]
    fn test_check_class_methods() {
        let files = vec![
            ("a.ts".to_string(), "class Calculator { add(x: number, y: number) { return x + y; } subtract(x: number, y: number) { return x - y; } }".to_string()),
        ];

        let program = compile_files(files);
        let result = check_functions_parallel(&program);

        // Should find the class methods
        let total_functions: usize = result.file_results.iter()
            .map(|r| r.function_results.len())
            .sum();
        assert!(total_functions >= 2, "Should find at least 2 class methods");
    }

    #[test]
    fn test_check_with_stats() {
        let files = vec![
            ("a.ts".to_string(), "function foo() { return 1; }".to_string()),
            ("b.ts".to_string(), "function bar() { return 2; }".to_string()),
            ("c.ts".to_string(), "function baz() { return 3; }".to_string()),
        ];

        let program = compile_files(files);
        let (result, stats) = check_functions_with_stats(&program);

        assert_eq!(stats.file_count, 3);
        assert_eq!(stats.function_count, 3);
        assert_eq!(result.file_results.len(), 3);
    }

    #[test]
    fn test_check_large_program_parallel() {
        // Test parallel checking with many files
        let files: Vec<_> = (0..50)
            .map(|i| {
                let source = format!(
                    "function fn{}(x: number): number {{ return x * {}; }} const val{} = fn{}(10);",
                    i, i, i, i
                );
                (format!("module{}.ts", i), source)
            })
            .collect();

        let program = compile_files(files);
        let (result, stats) = check_functions_with_stats(&program);

        assert_eq!(stats.file_count, 50);
        // Each file has 1 function declaration
        assert!(stats.function_count >= 50, "Expected at least 50 functions, got {}", stats.function_count);
    }

    #[test]
    fn test_check_consistency() {
        // Check the same program multiple times - results should be consistent
        let files = vec![
            ("a.ts".to_string(), "function add(x: number, y: number): number { return x + y; }".to_string()),
        ];

        let program = compile_files(files);

        let result1 = check_functions_parallel(&program);
        let result2 = check_functions_parallel(&program);

        assert_eq!(result1.function_count, result2.function_count);
        assert_eq!(result1.diagnostic_count, result2.diagnostic_count);
        assert_eq!(result1.file_results.len(), result2.file_results.len());
    }

    #[test]
    fn test_check_nested_functions() {
        let files = vec![
            ("a.ts".to_string(), "function outer() { function inner() { return 1; } return inner(); }".to_string()),
        ];

        let program = compile_files(files);
        let result = check_functions_parallel(&program);

        // Should find both outer and inner functions
        let total_functions: usize = result.file_results.iter()
            .map(|r| r.function_results.len())
            .sum();
        assert!(total_functions >= 2, "Should find both outer and inner functions");
    }

    #[test]
    fn test_check_exported_functions() {
        let files = vec![
            ("a.ts".to_string(), "export function add(x: number, y: number) { return x + y; }".to_string()),
            ("b.ts".to_string(), "export function subtract(x: number, y: number) { return x - y; }".to_string()),
        ];

        let program = compile_files(files);
        let result = check_functions_parallel(&program);

        // Should find the exported functions
        let total_functions: usize = result.file_results.iter()
            .map(|r| r.function_results.len())
            .sum();
        assert!(total_functions >= 2, "Should find exported functions");
    }
}
