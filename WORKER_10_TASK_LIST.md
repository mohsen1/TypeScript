# Worker 10 Task List - Solver Squad

## Current Task
- [ ] **SOLV-25: Test discriminated union type checking**
  - Ensure discriminated unions narrow correctly
  - Test: `type Shape = { kind: "circle", radius: number } | { kind: "square", side: number }`
  - Verify type narrowing on discriminant property

## Queue
- [ ] **SOLV-26: Add readonly modifier handling**
  - Implement readonly property type checking
  - Ensure readonly is covariant (can assign readonly to mutable)
  - Test: `readonly x: number` vs `x: number` assignability
  - Coordinate with Worker 9 on intersection types

## Completed
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
