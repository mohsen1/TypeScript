# Worker 6 Task List - Binder Squad

## Current Task
- [ ] **BIND-17: Fix value namespace vs type namespace collision**
  - Handle cases where same name is used as value and type
  - Test: `interface X { } const X: number;`
  - Ensure proper namespace separation during binding

## Completed
- [x] **BIND-12: Test namespace and enum resolution**
  - Created test file: `tests/cases/conformance/enums/namespaceMemberResolution.ts`
  - Created test file: `tests/cases/conformance/enums/enumAccessibilityAcrossImports.ts`
  - Added 11 Rust unit tests in `wasm/src/thin_binder_tests.rs`:
    - test_namespace_member_resolution_basic
    - test_namespace_member_resolution_nested
    - test_namespace_member_resolution_non_exported
    - test_namespace_deep_chain_resolution
    - test_enum_member_access
    - test_enum_namespace_merging_access
    - test_enum_with_initialized_members
    - test_const_enum_declaration
    - test_namespace_reopening_exports
    - test_enum_namespace_merging_with_exports
- [x] **BIND-9: Write conformance tests for Binder**
  - Created test file: `tests/conformance/binder_tests.ts`
  - Added cases for global symbols, module augmentation, ambient contexts
  - Verified TS2304 counts match tsc output
  - Created 3 comprehensive test files covering ES6 import shadowing scenarios
- [x] **BIND-6: Fix symbol lookup order**
  - Verified correct scope chain: local -> module -> global (already implemented)
  - Added shadowing tests for variable declarations
  - Added tests for `import { x }` vs `let x` correctly
  - Created 3 comprehensive test files covering ES6 import shadowing scenarios
- [x] **BIND-3: Trace TS2304 false positives**
  - Found 10+ specific cases where valid code produces TS2304
  - Identified root causes:
    - Missing lib symbol injection (~700 errors)
    - Scope lookup order issues
    - Module resolution problems
  - Created minimal repro test cases
  - Documented in `docs/ts2304_analysis.md`
