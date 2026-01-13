# Worker 14 Task List - Solver Squad

## Current Task
- [ ] **SOLV-45: Add function bivariance configuration**
  - Implement the "Lawyer" layer for function parameter checking
  - Support `strictFunctionTypes` compiler option
  - Methods should remain bivariant, standalone functions contravariant
  - Test: callback assignment compatibility

## Completed
- [x] **SOLV-44: Implement mapped type evaluation** (already implemented)
  - evaluate_mapped handles `{ [K in Keys]: Transform<K> }` lowering
  - readonly/optional modifiers and key remapping (as clause) supported
  - 136 mapped type tests passing

- [x] **SOLV-43: Add index access type evaluation** (already implemented)
  - evaluate_index_access handles T[K] type resolution
  - evaluate_object_index handles literal key access
  - Union key access returns union of property types
  - Union distribution (A | B)[K] -> A[K] | B[K] implemented
  - 63 index access tests passing

- [x] **SOLV-42: Implement tuple type checking** (already implemented)
  - TypeKey::Tuple exists with TupleElement support
  - check_tuple_subtype handles length/type mismatches
  - evaluate_tuple_index handles numeric literal access
  - Rest elements and optional elements supported

## Notes
- Focus on improving exactness from 30.8% toward 40% target
- Prioritize features that affect the most conformance tests
- Follow the "Lawyer vs Judge" pattern from specs/SOLVER.md
