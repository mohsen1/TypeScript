# Worker 3 Plan - Squad Anvil

## Mission
LSP Go-To-Definition Implementation

Status: Complete
Priority: P1 (High)

## Current Assignment
**[COMPLETED] Implement LSP Go-To-Definition**

### Background
Go-to-definition allows users to navigate to where symbols are defined. Critical IDE feature.

### Implementation Summary

The LSP Go-To-Definition feature is **FULLY IMPLEMENTED** with comprehensive test coverage.

**Implementation Location:** `src/lsp/definition.rs`

**Features Implemented:**
1. ✅ Find symbol at cursor position (via `find_node_at_offset`)
2. ✅ Look up symbol declaration in binder (via `ScopeWalker::resolve_node`)
3. ✅ Return location (file, line, column) for symbol declarations
4. ✅ Handle different symbol types:
   - Variables and functions
   - Class members
   - Import/export declarations
   - Type aliases
   - Binding patterns
   - Parameter bindings
   - Decorator references
   - Nested scopes (arrow functions, blocks, etc.)

**Integration:**
- `Project::get_definition()` in `src/lsp/project.rs` provides the LSP API
- Uses scope caching for performance
- Returns `Vec<Location>` for multiple declarations (overloads, merged declarations)

### Test Results

**Unit Tests:** 22/22 PASS (in `src/lsp/definition.rs`)
- test_goto_definition_simple_variable
- test_goto_definition_type_reference
- test_goto_definition_binding_pattern
- test_goto_definition_parameter_binding_pattern
- test_goto_definition_class_method_local
- test_goto_definition_class_method_name
- test_goto_definition_class_member_not_in_scope
- test_goto_definition_class_self_reference
- test_goto_definition_class_expression_name
- test_goto_definition_nested_arrow_in_conditional
- test_goto_definition_nested_arrow_in_if_condition
- test_goto_definition_nested_arrow_in_while_condition
- test_goto_definition_nested_arrow_in_for_of_expression
- test_goto_definition_export_default_expression
- test_goto_definition_labeled_statement_local
- test_goto_definition_with_statement_local
- test_goto_definition_var_hoisted_in_nested_block
- test_goto_definition_decorator_reference
- test_goto_definition_decorator_argument_local
- test_goto_definition_nested_arrow_in_object_literal
- test_goto_definition_class_static_block_local
- test_goto_definition_not_found

**Integration Tests:** 3/3 PASS (in `src/lsp/tests.rs`)
- test_lsp_with_function
- test_lsp_diagnostic_conversion
- test_lsp_workflow_simple
- test_project_multi_file_definition

### Key Code Locations
- `src/lsp/definition.rs` - go-to-definition implementation (733 lines, 22 tests)
- `src/lsp/project.rs` - LSP server integration, `Project::get_definition()` method
- `src/lsp/resolver.rs` - `ScopeWalker` for symbol resolution
- `src/binder/` - symbol declaration tracking

## Task Queue
- [ ] Help with find-references or document symbols if needed

## Completed
- [x] ES5 Private Accessor Emission (7 tests passing) - MERGED to squad/anvil
- [x] Parser error recovery: function keyword in class - MERGED to squad/anvil
- [x] LSP Go-To-Definition (25 tests passing) - Already complete

## Ready for Merge
Go-To-Definition was already implemented. All tests passing.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Document go-to-definition implementation status`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
