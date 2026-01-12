# Worker 8 Task List

## Current Task
- [ ] **SOLVER-3: Implement "Lawyer" layer for Any propagation**
  - Read `specs/SOLVER.md` for the Lawyer layer spec
  - Create `src/solver/lawyer.rs` module
  - Implement logic: `Any` should NOT silence structural mismatches
  - Only allow `Any` to pass through when explicitly required

## Queue
- [ ] **SOLVER-4: Fix generic type inference strictness**
  - Find where generic inference bails out to `Any`
  - Implement proper constraint solving for generics
  - Fix tests with missing TS2322 errors on generic code
  - Add tests: `function foo<T>(x: T): string` should error on `foo(42)`

## Completed
(none yet)
