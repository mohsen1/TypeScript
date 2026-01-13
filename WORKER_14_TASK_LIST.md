# Worker 14 Task List - Solver Squad

## Current Task
- [ ] **SOLV-43: Add index access type evaluation**
  - Implement `T[K]` type resolution
  - Handle literal key access: `Person["age"]`
  - Handle union key access: `Person["age" | "name"]`
  - Distribute over union types: `(A | B)[K]` -> `A[K] | B[K]`

## Queue
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
- [x] **SOLV-42: Implement tuple type checking** (already implemented)
  - TypeKey::Tuple exists with TupleElement support
  - check_tuple_subtype handles length/type mismatches
  - evaluate_tuple_index handles numeric literal access
  - Rest elements and optional elements supported
  - Fixed build errors and test API compatibility

## Notes
- Focus on improving exactness from 30.8% toward 40% target
- Prioritize features that affect the most conformance tests
- Follow the "Lawyer vs Judge" pattern from specs/SOLVER.md
