# Worker 8 Task List - Solver Squad

## Current Task
- [ ] **SOLV-17: Coordinate with Worker 7 on integration**
  - Work with Worker 7 to merge Lawyer layer changes
  - Ensure generic checking works with Unknown fallback
  - Verify all solver components integrate correctly

## Queue
  - Work with Worker 7 to merge Lawyer layer changes
  - Ensure generic checking works with Unknown fallback
  - Verify all solver components integrate correctly
- [ ] **SOLV-31: Fix generic type inference from call sites**
  - Handle `function f<T>(x: T): T` with `f(123)` inferring T=number
  - Test generic inference with union types
  - Verify generic constraints are enforced during inference
- [ ] **SOLV-32: Strengthen generic constraint checking**
  - Ensure `<T extends U>` constraints are enforced
  - Test: `function f<T extends number>(x: T) {} f<string>("hi")` should error
  - Handle complex constraints with extends conditions
- [ ] **SOLV-33: Test generic variance with function types**
  - Verify contravariant function parameter types
  - Test: `(x: number) => void` vs `(x: string | number) => void`
  - Handle function types in generic contexts

## Completed
- [x] **SOLV-16: Strengthen generic inference**
  - Fixed generic type inference from context
  - Handled conditional types with generics
  - Implemented proper generic variance checking
  - Tested complex generic scenarios
  - Modified wasm/src/solver/infer.rs and infer_tests.rs
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
