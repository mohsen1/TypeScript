# Worker 6 Task List - Binder Squad

## Current Task
- [ ] **BIND-6: Fix symbol lookup order**
  - Ensure correct scope chain: local -> module -> global
  - Add shadowing tests for variable declarations
  - Handle `import { x }` vs `let x` correctly

## Queue
- [ ] **BIND-9: Write conformance tests for Binder**
  - Create test file: `tests/conformance/binder_tests.ts`
  - Add cases for global symbols, module augmentation, ambient contexts
  - Verify TS2304 counts match tsc output
  - Goal: Eliminate TS2304 false positives
- [ ] **BIND-12: Test namespace and enum resolution**
  - Verify namespace member resolution works correctly
  - Test enum accessibility across imports
  - Ensure `namespace.subsymbol` resolution works

## Completed
- [x] **BIND-3: Trace TS2304 false positives**
  - Found 10+ specific cases where valid code produces TS2304
  - Identified root causes:
    - Missing lib symbol injection (~700 errors)
    - Scope lookup order issues
    - Module resolution problems
  - Created minimal repro test cases
  - Documented in `docs/ts2304_analysis.md`
