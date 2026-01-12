# Worker 7 Task List - Solver Squad

## Current Task
- [ ] **SOLV-1: Audit current solve_subtype implementation**
  - Read `src/solver/subtype.rs` completely
  - Identify all locations that return `Any` or `True` as fallback
  - Document fallback behavior in `docs/solver_fallback_analysis.md`
  - Find where error poisoning suppresses downstream errors

## Queue
- [ ] **SOLV-4: Change fallback from Any to Unknown**
  - Modify `solve_subtype` to return `Unknown` on safe failures
  - Update `Type::Unknown` to be more strict than `Any`
  - Ensure unknown types still propagate errors (not silence them)
  - Test: `let x: number = "string"` should error even with unknown in scope
- [ ] **SOLV-7: Test TS2322 fixes**
  - Create test file for assignment type mismatches
  - Verify errors are emitted for: `let x: string = 123`, generics with wrong types
  - Goal: Convert 310 missing TS2322 errors to actual errors or extra errors

## Completed
(none yet)
