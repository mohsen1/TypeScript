# Worker 8 Task List - Solver Squad

## Current Task
- [ ] **SOLV-2: Implement "Lawyer" layer for Any propagation**
  - Read `specs/SOLVER.md` for Lawyer spec
  - Create `src/solver/lawyer.rs` with `AnyPropagationRules`
  - Implement logic: Any should NOT silence structural mismatches
  - Add `is_any_allowed_to_suppress` helper function

## Queue
- [ ] **SOLV-5: Fix generic type parameter checking**
  - Harden `solve_generic_instantiation` against unknown type params
  - Don't bail to `Any` when constraints can't be verified
  - Emit errors for unsatisfied generic constraints
  - Test: `function f<T>(x: T): number { return x; } f<string>("hi")`
- [ ] **SOLV-8: Handle implicit any parameters (TS7006)**
  - Check function parameters with no annotation
  - When inferred type is implicitly any, emit TS7006
  - Test: `function f(x) { return x; }` should error with TS7006
  - Goal: Fix 357 missing TS7006 errors

## Completed
(none yet)
