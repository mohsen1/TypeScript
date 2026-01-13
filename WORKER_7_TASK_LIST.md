# Worker 7 Task List - Solver Squad

## Current Task
- [ ] **SOLV-15: Implement strict subtyping for generic types**
  - Don't bail to Any when generic parameters are complex
  - Add proper constraint checking in solve_subtype
  - Handle generic instantiation with unknown type arguments
  - Coordinate with Worker 8 on generic parameter checking

## Queue
- [ ] **SOLV-18: Fix tuple type subtyping**
  - Implement covariant tuple subtyping
  - Handle tuple length differences correctly
  - Test: `[number, string]` vs `[number, string, boolean]`
- [ ] **SOLV-19: Strengthen function type variance**
  - Implement proper contravariance for parameter types
  - Handle function type assignability correctly
  - Test: `(x: number) => void` vs `(x: string | number) => void`
- [ ] **SOLV-20: Add solver integration tests**
  - Create comprehensive test suite for solver strictness
  - Verify TS2322 and TS7006 error counts improve
  - Goal: Convert missing TS2322 errors to exact matches

## Completed
- [x] **SOLV-14: Integrate Lawyer layer into CompatChecker**
  - Added AnyPropagationRules field to CompatChecker struct
  - Integrated Lawyer layer into both constructors (new, with_resolver)
  - Added set_strict_any_propagation() method for strict mode
  - Added lawyer() and lawyer_mut() accessors
  - No file deletions - clean merge with latest rust branch
- [x] **SOLV-7: Test TS2322 fixes**
- [x] **SOLV-1: Audit current solve_subtype implementation**
- [x] **SOLV-4: Change fallback from Any to Unknown**
- [x] **SOLV-11: Add comprehensive error messages**
- [x] **SOLV-12: Lawyer layer integration (attempted - had conflicts, completed in SOLV-14)**
