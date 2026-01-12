# Worker 7 Task List - Solver Squad (Lead)

## Current Task
- [ ] **SOLV-7: Test TS2322 fixes**
  - Create test file for assignment type mismatches
  - Verify errors are emitted for: `let x: string = 123`, generics with wrong types
  - Goal: Convert 310 missing TS2322 errors to actual errors or extra errors

## Queue
- [ ] **SOLV-11: Coordinate Solver Squad integration**
  - Review Workers 8-9 implementations for consistency
  - Ensure Lawyer layer integration is correct across all solver components
  - Merge completed solver components into main branch
- [ ] **SOLV-15: Verify overall Solver improvements**
  - Run full conformance suite after all solver fixes
  - Measure TS2322 and TS7006 missing error reduction
  - Goal: Convert 310 TS2322 and 357 TS7006 missing errors
  - Document final solver conformance metrics

## Completed
- [x] **SOLV-1: Audit current solve_subtype implementation**
  - Read `wasm/src/solver/subtype.rs` completely
  - Identified all locations that return `Any` or `True` as fallback
  - Documented fallback behavior in `docs/solver_fallback_analysis.md`
  - Found where error poisoning suppresses downstream errors
- [x] **SOLV-4: Change fallback from Any to Unknown**
  - Modified `solve_subtype` to return `Unknown` on safe failures
  - Updated `Type::Unknown` to be more strict than `Any`
  - Ensured unknown types still propagate errors (not silence them)
  - Tested: `let x: number = "string"` errors even with unknown in scope
  - Coordinated with Worker 8 on Lawyer layer integration
