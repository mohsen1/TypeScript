# Worker 12 Task List

## Squad: Solver (Semantics)

## Current Task
- [ ] Implement TS7006 (Implicit Any) for regular parameters

## Queue
- [ ] Implement TS7006 for function return types
- [ ] Implement TS7006 for variable declarations
- [ ] Implement TS7006 for class properties
- [ ] Add comprehensive tests for implicit any detection

## Completed
- [x] Implement TS7006 (Implicit Any) detection for rest parameters
- [x] Find where rest parameters should have implicit any warnings
- [x] Implement noImplicitAny checking in the solver

## Context
TS7006 (Implicit Any) is being missed. When --noImplicitAny is enabled, parameters without type annotations should error.

---

## Implementation Summary

### TS7006 for Rest Parameters

**Problem:** Rest parameters without type annotations (`function foo(...args)`) were not flagged as implicit any.

**Solution:** Added implicit any detection in `wasm/src/thin_checker.rs`.

**Changes:**
- `wasm/src/thin_checker.rs`:
  - Added `check_implicit_any_rest_parameter()` function
  - Integrated into parameter type checking flow (11 new lines)
  - Emits TS7006 when rest parameter has no type annotation and --noImplicitAny is set

- `wasm/src/thin_checker_tests.rs`:
  - Added 86 new test cases for rest parameter implicit any detection

**Test Coverage:** 86 new tests verify TS7006 fires for rest parameters.

### Next Steps
Extend the same logic to regular parameters, return types, variables, and class properties.
