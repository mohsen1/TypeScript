# Worker 8 Task List - Solver Squad

## Current Task
- [ ] **SOLV-5: Fix generic type parameter checking**
  - Harden `solve_generic_instantiation` against unknown type params
  - Don't bail to `Any` when constraints can't be verified
  - Emit errors for unsatisfied generic constraints
  - Test: `function f<T>(x: T): number { return x; } f<string>("hi")`
  - Coordinate with Worker 7 on Unknown fallback changes

## Queue
- [ ] **SOLV-8: Handle implicit any parameters (TS7006)**
  - Check function parameters with no annotation
  - When inferred type is implicitly any, emit TS7006
  - Test: `function f(x) { return x; }` should error with TS7006
  - Goal: Fix 357 missing TS7006 errors
- [ ] **SOLV-16: Strengthen generic inference**
  - Fix generic type inference from context
  - Handle conditional types with generics
  - Implement proper generic variance checking
  - Test complex generic scenarios
- [ ] **SOLV-17: Coordinate with Worker 7 on integration**
  - Work with Worker 7 to merge Lawyer layer changes
  - Ensure generic checking works with Unknown fallback
  - Verify all solver components integrate correctly

## Completed
- [x] **SOLV-2: Implement "Lawyer" layer for Any propagation**
  - Read `specs/SOLVER.md` for Lawyer spec
  - Created `src/solver/lawyer.rs` with `AnyPropagationRules`
  - Implemented logic: Any should NOT silence structural mismatches
  - Added `is_any_allowed_to_suppress` helper function
  - Integrated Lawyer layer into `CompatChecker`
