# Worker 14 Task List - Solver Squad

## Current Task
- [ ] **SOLV-42: Implement tuple type checking**
  - Add tuple type representation in TypeKey
  - Implement tuple length checking (TS2322 for length mismatches)
  - Handle tuple element access with numeric literals
  - Support rest elements in tuples
  - Test: `[string, number]` vs `[string, number, boolean]`

## Queue
- [ ] **SOLV-43: Add index access type evaluation**
  - Implement `T[K]` type resolution
  - Handle literal key access: `Person["age"]`
  - Handle union key access: `Person["age" | "name"]`
  - Distribute over union types: `(A | B)[K]` -> `A[K] | B[K]`

- [ ] **SOLV-44: Implement mapped type evaluation**
  - Add `{ [K in Keys]: Transform<K> }` lowering
  - Handle readonly/optional modifiers
  - Support key remapping with `as` clause
  - Test with `Partial<T>`, `Required<T>`, `Readonly<T>`

- [ ] **SOLV-45: Add function bivariance configuration**
  - Implement the "Lawyer" layer for function parameter checking
  - Support `strictFunctionTypes` compiler option
  - Methods should remain bivariant, standalone functions contravariant
  - Test: callback assignment compatibility

## Completed
(none yet)

## Notes
- Focus on improving exactness from 30.8% toward 40% target
- Prioritize features that affect the most conformance tests
- Follow the "Lawyer vs Judge" pattern from specs/SOLVER.md
