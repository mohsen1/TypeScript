# Worker 10 Task List - Solver Squad

## Current Task
- [ ] **SOLV-24: Write conformance tests for Solver**
  - Create test file: `tests/conformance/solver_tests.ts`
  - Add cases for TS2322 (type not assignable) errors
  - Add cases for TS7006 (implicit any) errors
  - Verify error counts match tsc output
  - Goal: Reduce missing TS2322/TS7006 errors by 80%

## Queue
- [ ] **SOLV-25: Test discriminated union type checking**
  - Ensure discriminated unions narrow correctly
  - Test: `type Shape = { kind: "circle", radius: number } | { kind: "square", side: number }`
  - Verify type narrowing on discriminant property
- [ ] **SOLV-26: Add readonly modifier handling**
  - Implement readonly property type checking
  - Ensure readonly is covariant (can assign readonly to mutable)
  - Test: `readonly x: number` vs `x: number` assignability
  - Coordinate with Worker 9 on intersection types

## Completed
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
  - No files deleted
- [x] **SOLV-13: Add conformance test reports**
- [x] **SOLV-10: Write comprehensive tests**
