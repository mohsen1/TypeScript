# Worker 10 Task List - Solver Squad

## Current Task
- [ ] **SOLV-26: Add readonly modifier handling**
  - Implement readonly property type checking
  - Ensure readonly is covariant (can assign readonly to mutable)
  - Test: `readonly x: number` vs `x: number` assignability
  - Coordinate with Worker 9 on intersection types

## Queue
  - Implement readonly property type checking
  - Ensure readonly is covariant (can assign readonly to mutable)
  - Test: `readonly x: number` vs `x: number` assignability
  - Coordinate with Worker 9 on intersection types
- [ ] **SOLV-37: Implement discriminated union narrowing**
  - Narrow union types based on discriminant property
  - Test: `type Shape = { kind: "circle", radius: number } | { kind: "square", side: number }`
  - Verify `if (shape.kind === "circle")` narrows to circle type
  - Handle user-defined type guards with discriminants
- [ ] **SOLV-38: Add optional chaining type checking**
  - Implement `obj?.prop` type checking
  - Handle `obj?.method()` return types
  - Test optional chaining with null/undefined
  - Verify `obj?.[key]` indexed access types
- [ ] **SOLV-39: Coordinate with CFA and Binder squads on cross-cutting issues**
  - Ensure solver strictness works with CFA improvements
  - Verify Unknown fallback doesn't conflict with fixed global scope
  - Test end-to-end type checking with all improvements integrated

## Completed
- [x] **SOLV-25: Test discriminated union type checking**
  - Ensured discriminated unions narrow correctly
  - Tested: `type Shape = { kind: "circle", radius: number } | { kind: "square", side: number }`
  - Verified type narrowing on discriminant property
  - Added tests to solver_tests.ts
- [x] **SOLV-24: Write conformance tests for Solver**
  - Created test file: `tests/conformance/solver_tests.ts`
  - Added cases for TS2322 (type not assignable) errors
  - Added cases for TS7006 (implicit any) errors
  - Verified error counts match tsc output
  - Enhanced LSP hover, completions, and project modules
  - Refactored test files for better organization
- [x] **SOLV-23: Add strictness mode toggle**
  - Implemented `--strict` compiler flag to enable all strict checks
  - Ensured Unknown fallback is used in strict mode
  - Enabled implicit any parameter detection (TS7006)
  - Tested strict vs non-strict mode differences
- [x] **SOLV-17: Add structural type checking tests**
  - Created test file: `tests/cases/conformance/solver/structural_type_tests.ts`
  - Wrote tests for structural type compatibility
  - Tested object types with same properties
  - Tested interface compatibility
- [x] **SOLV-13: Add conformance test reports**
- [x] **SOLV-10: Write comprehensive tests**
