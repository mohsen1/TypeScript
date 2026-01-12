# Worker 9 Task List

## Current Task
- [ ] **SOLVER-5: Fix missing TS2322 (Type not assignable) errors**
  - Audit 310 missing TS2322 cases from conformance report
  - Categorize by pattern (generics, unions, literals, etc.)
  - Fix the top 5 most common patterns
  - Add regression tests for each fix

## Queue
- [ ] **SOLVER-6: Fix missing TS7006 (Implicit Any) errors**
  - Audit 357 missing TS7006 cases from conformance report
  - Find where parameters implicitly get `Any` without error
  - Add `check_implicit_any` function in checker
  - Emit error when parameter type cannot be inferred

## Completed
(none yet)
