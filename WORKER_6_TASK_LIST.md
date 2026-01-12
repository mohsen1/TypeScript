# Worker 6 Task List - Binder Squad

## Current Task
- [ ] **BIND-3: Trace TS2304 false positives**
  - Find 10 specific cases where valid code produces TS2304
  - Identify root cause (missing lib symbol, scope leak, module resolution)
  - Create minimal repro test cases
  - Document in `docs/ts2304_analysis.md`

## Queue
- [ ] **BIND-6: Fix symbol lookup order**
  - Ensure correct scope chain: local -> module -> global
  - Add shadowing tests for variable declarations
  - Handle `import { x }` vs `let x` correctly
- [ ] **BIND-9: Write conformance tests for Binder**
  - Create test file: `tests/conformance/binder_tests.ts`
  - Add cases for global symbols, module augmentation, ambient contexts
  - Verify TS2304 counts match tsc output
  - Goal: Eliminate TS2304 false positives

## Completed
(none yet)
