# Worker 3 Plan - Squad Anvil

## Mission
LSP Go-To-Definition and Find References Implementation

Status: Complete
Priority: P1 (High)

## Current Assignment
**[COMPLETED] Implement LSP Find References**

### Background
Find References allows users to find all usages of a symbol. Critical IDE feature for refactoring.

### Implementation Summary

The LSP Find References feature is **FULLY IMPLEMENTED** with comprehensive test coverage.

**Implementation Location:** `src/lsp/references.rs` (963 lines)

**Features Implemented:**
1. ✅ Find symbol at cursor position (via `find_node_at_offset`)
2. ✅ Look up symbol declaration in binder (via `ScopeWalker::resolve_node`)
3. ✅ Find all references to symbol in the AST (via `ScopeWalker::find_references`)
4. ✅ Return locations (file, line, column) for all references
5. ✅ Include both declarations and usages
6. ✅ Support for "usages only" (excluding declarations)
7. ✅ Handle different symbol types:
   - Variables and functions
   - Class members and self-references
   - Import/export declarations
   - Binding patterns (object/array destructuring)
   - Parameter bindings
   - Decorator references
   - Template expressions
   - JSX expressions
   - Tagged templates
   - Await expressions
   - As-expressions
   - Nested scopes (arrow functions, blocks, etc.)

**Integration:**
- `Project::get_references()` in `src/lsp/project.rs` provides the LSP API
- Uses scope caching for performance
- Returns `Vec<Location>` for all references (declarations + usages)

### Test Results

**Unit Tests:** 23/23 PASS (in `src/lsp/references.rs`)
- test_find_references_simple
- test_find_references_for_symbol
- test_find_references_not_found
- test_find_references_template_expression
- test_find_references_jsx_expression
- test_find_references_await_expression
- test_find_references_tagged_template_expression
- test_find_references_as_expression
- test_find_references_binding_pattern
- test_find_references_binding_pattern_initializer
- test_find_references_parameter_binding_pattern
- test_find_references_parameter_array_binding
- test_find_references_nested_arrow_in_switch_case
- test_find_references_nested_arrow_in_if_condition
- test_find_references_export_default_expression
- test_find_references_labeled_statement_local
- test_find_references_with_statement_local
- test_find_references_var_hoisted_in_nested_block
- test_find_references_decorator_reference
- test_find_references_class_method_local
- test_find_references_class_self_reference
- test_find_references_class_expression_name
- test_find_references_class_static_block_local

**Previous LSP Features:**
- Go-To-Definition: 22/22 PASS (in `src/lsp/definition.rs`)
- Integration Tests: 3/3 PASS (in `src/lsp/tests.rs`)

**Total LSP Tests:** 48/48 PASS

### Key Code Locations
- `src/lsp/references.rs` - find references implementation (963 lines, 23 tests)
- `src/lsp/definition.rs` - go-to-definition implementation (733 lines, 22 tests)
- `src/lsp/project.rs` - LSP server integration
- `src/lsp/resolver.rs` - `ScopeWalker` for symbol resolution
- `src/binder/` - symbol declaration tracking

## Task Queue
- [ ] Document symbols if needed
- [ ] Other LSP features (completions, hover, rename, etc.)

## Completed
- [x] ES5 Private Accessor Emission (7 tests passing) - MERGED to squad/anvil
- [x] Parser error recovery: function keyword in class - MERGED to squad/anvil
- [x] LSP Go-To-Definition (22 tests passing) - Already complete
- [x] LSP Find References (23 tests passing) - Already complete

## Ready for Merge
Both LSP features were already implemented. All 48 tests passing.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Resolve merge conflict and confirm find-references complete`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
