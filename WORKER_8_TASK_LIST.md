# Worker 8 Task List - Solver Squad

## Current Task
- [ ] **SOLV-16: Strengthen generic inference**
  - Fix generic type inference from context
  - Handle conditional types with generics
  - Implement proper generic variance checking
  - Test complex generic scenarios

## Queue
- [ ] **SOLV-17: Coordinate with Worker 7 on integration**
  - Work with Worker 7 to merge Lawyer layer changes
  - Ensure generic checking works with Unknown fallback
  - Verify all solver components integrate correctly

## Completed
- [x] **SOLV-8: Handle implicit any parameters (TS7006)**
  - Checked function parameters with no annotation
  - When inferred type is implicitly any, emit TS7006
  - Tested `function f(x) { return x; }` errors with TS7006
  - Created `wasm/src/solver/operations.rs` and `operations_tests.rs`
- [x] **SOLV-5: Fix generic type parameter checking**
  - Hardened `solve_generic_instantiation` against unknown type params
  - No longer bail to `Any` when constraints can't be verified
  - Emit errors for unsatisfied generic constraints
  - Tested `function f<T>(x: T): number { return x; } f<string>("hi")`
- [x] **SOLV-2: Implement "Lawyer" layer for Any propagation**
  - Read `specs/SOLVER.md` for Lawyer spec
  - Created `src/solver/lawyer.rs` with `AnyPropagationRules`
  - Implemented logic: Any should NOT silence structural mismatches
  - Added `is_any_allowed_to_suppress` helper function
  - Integrated Lawyer layer into `CompatChecker`
