# Worker 9 Task List

## Current Task
All tasks completed!

## Queue
(none)

## Completed
- [x] Add subtyping conformance tests
  - Create tests from the 310 missing TS2322 cases
  - Verify each now emits an error
  - Check for false positives
- [x] Fix missing TS7006 (Implicit Any) errors - 357 cases
  - Track where type inference defaults to Any without error
  - Ensure missing type annotations trigger errors in strict mode
  - Add tests for implicit any in function parameters
  - Verify error messages match tsc output
- [x] Fix missing TS2322 (Type not assignable) errors - 310 cases
  - Add logging to `solve_subtype` to see where errors are dropped
  - Find cases where structural mismatches are incorrectly accepted
  - Force errors on all type mismatches, even edge cases
  - Goal: Convert "Missing" to "Extra" is better than unsound
