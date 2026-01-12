# Worker 9 Task List - Solver Squad

## Current Task
- [ ] **SOLV-6: Fix intersection type checking**
  - Implement `solve_intersection_subtype` correctly
  - Intersection A is subtype of B if ANY member of A is subtype of B
  - Test: `type A = { x: number } & { y: string };`

## Queue
- [ ] **SOLV-9: Test generic constraint violations**
  - Create tests for bounded generics: `<T extends number>`
  - Ensure `f<string>(123)` errors when string doesn't extend number
  - Test default type parameter inference behavior

## Completed
- [x] **SOLV-3: Strengthen union type checking**
  - Audited `solve_union_subtype` implementation
  - Fixed union subtyping: Union A is subtype of Union B only if all A's members are in B
  - Removed `any` as universal subtype in unions
  - Added union type tests in `union_tests.rs`
