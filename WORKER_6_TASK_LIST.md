# Worker 6 Task List - Binder Squad

## Current Task
- [ ] **BIND-12: Test namespace and enum resolution**
  - Verify namespace member resolution works correctly
  - Test enum accessibility across imports
  - Ensure `namespace.subsymbol` resolution works

## Queue
- [ ] **BIND-17: Fix value namespace vs type namespace collision**
  - Handle cases where same name is used as value and type
  - Test: `interface X { } const X: number;`
  - Ensure proper namespace separation during binding

## Completed
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
