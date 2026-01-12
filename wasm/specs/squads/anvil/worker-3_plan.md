# Worker 3 Plan - Squad Anvil

## Mission
LSP Features Implementation

Status: Complete
Priority: P1 (High)

## Current Assignment
**[COMPLETED] Implement LSP Code Actions**

### Background
Code Actions provide quick fixes and refactorings to improve code quality and fix errors.

### Implementation Summary

The LSP Code Actions feature is **FULLY IMPLEMENTED** with comprehensive test coverage.

**Implementation Location:** `src/lsp/code_actions.rs` (2441 lines)

**Features Implemented:**
1. ✅ Extract Variable (selection-based refactoring)
2. ✅ Organize Imports (sort-only, respects blank line groups)
3. ✅ Remove Unused Import (diagnostic-based quick fix)
4. ✅ Add Missing Property (diagnostic-based quick fix, local declarations)
5. ✅ Add Missing Import (diagnostic-based quick fix, project-aware)

**Integration:**
- `Project::get_code_actions()` in `src/lsp/project.rs` provides the LSP API
- Supports different code action kinds: QuickFix, Refactor, RefactorExtract, SourceOrganizeImports

### Test Results

**Unit Tests:** 42/42 PASS (in `src/lsp/code_actions_tests.rs`)

Extract Variable (17 tests):
- test_extract_variable_array_literal_span
- test_extract_variable_avoids_name_collision
- test_extract_variable_blocks_tdz_for_loop_initializer
- test_extract_variable_blocks_tdz_in_jsx_attribute
- test_extract_variable_blocks_tdz_in_jsx_child
- test_extract_variable_blocks_tdz_in_jsx_tag
- test_extract_variable_empty_range
- test_extract_variable_call_expression_span
- test_extract_variable_no_action_cross_scope
- test_extract_variable_jsx_child_wraps_expression
- test_extract_variable_no_action_for_simple_literal
- test_extract_variable_object_literal_span
- test_extract_variable_parenthesizes_comma_expression_with_parens
- test_extract_variable_parenthesizes_comma_expression
- test_extract_variable_preserves_parenthesized_conditional_replacement
- test_extract_variable_preserves_parenthesized_replacement
- test_extract_variable_property_access

Organize Imports (1 test):
- test_organize_imports_sort_only

Remove Unused Import (3 tests):
- test_quickfix_remove_unused_default_import
- test_quickfix_remove_unused_named_import
- test_quickfix_remove_unused_named_import_entire_decl
- test_quickfix_preserves_type_only_named_import

Add Missing Property (5 tests):
- test_quickfix_add_missing_property_object_literal_element_access
- test_quickfix_add_missing_property_object_literal_multiline
- test_quickfix_add_missing_property_object_literal_single_line
- test_quickfix_add_missing_property_object_literal_single_line_trailing_comma
- test_quickfix_add_missing_property_to_class
- test_quickfix_add_missing_property_to_class_element_access

Add Missing Import (15 tests):
- test_quickfix_add_missing_import_after_existing_import
- test_quickfix_add_missing_import_default
- test_quickfix_add_missing_import_named
- test_quickfix_add_missing_import_namespace
- test_quickfix_add_missing_import_merge_default_with_named
- test_quickfix_add_missing_import_merge_default_with_namespace
- test_quickfix_add_missing_import_merge_named_multiline
- test_quickfix_add_missing_import_merge_named_same_module
- test_quickfix_add_missing_import_merge_named_with_default
- test_quickfix_add_missing_import_class_extends_uses_value_import
- test_quickfix_add_missing_import_class_implements_uses_import_type
- test_quickfix_add_missing_import_type_position_uses_import_type
- test_quickfix_add_missing_import_value_skips_type_only_candidate
- test_quickfix_add_missing_import_type_query_uses_value_import

**Integration Tests:** 5/5 PASS (in `src/lsp/project_tests.rs`)
- test_project_code_actions_missing_import_default_export
- test_project_code_actions_missing_import_default_reexport
- test_project_code_actions_missing_import_named
- test_project_code_actions_missing_import_reexport
- test_project_code_actions_missing_import_tsx

**Total Code Actions Tests:** 47/47 PASS

**Previous LSP Features:**
- Go-To-Definition: 22/22 PASS (in `src/lsp/definition.rs`)
- Find References: 23/23 PASS (in `src/lsp/references.rs`)
- Integration Tests: 3/3 PASS (in `src/lsp/tests.rs`)

**Total LSP Tests:** 95/95 PASS

### Key Code Locations
- `src/lsp/code_actions.rs` - code actions implementation (2441 lines, 42 tests)
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
- [x] LSP Code Actions (47 tests passing) - Already complete

## Ready for Merge
All LSP features were already implemented. All 95 tests passing.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Document code actions implementation status`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
