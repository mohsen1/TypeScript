# Worker 7 Task List - Solver Squad

## Current Task
- [ ] **SOLV-20: Add solver integration tests**
  - Create comprehensive test suite for solver strictness
  - Verify TS2322 and TS7006 error counts improve
  - Goal: Convert missing TS2322 errors to exact matches

## Queue
- [ ] **SOLV-27: Verify solver strictness improvements**
  - Run full conformance test suite with solver changes
  - Measure actual reduction in missing TS2322/TS7006 errors
  - Document remaining gaps and edge cases

## Completed
- [x] **SOLV-19: Strengthen function type variance**
  - Implemented proper contravariance for parameter types
  - Handled function type assignability correctly
  - Tested `(x: number) => void` vs `(x: string | number) => void`
- [x] **SOLV-18: Fix tuple type subtyping**
  - Implemented covariant tuple subtyping
  - Handled tuple length differences correctly
  - Tested `[number, string]` vs `[number, string, boolean]`
- [x] **SOLV-15: Implement strict subtyping for generic types**
  - Modified erase_placeholders_for_inference to use constraint when available
  - No longer bail to Any when generic parameters have constraints
  - Proper constraint checking during generic type instantiation
  - Maintained existing behavior for unconstrained type parameters
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
