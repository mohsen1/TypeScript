# Worker 9 Task List - Solver Squad

## Current Task
- [ ] **SOLV-3: Strengthen union type checking**
  - Audit `solve_union_subtype` implementation
  - Fix: Union A is subtype of Union B only if all A's members are in B
  - Don't accept `any` as universal subtype in unions
  - Add tests for: `type A = 1 | 2; type B = 1 | 2 | 3; let a: A = 1 as B;`

## Queue
- [ ] **SOLV-6: Fix intersection type checking**
  - Implement `solve_intersection_subtype` correctly
  - Intersection A is subtype of B if ANY member of A is subtype of B
  - Test: `type A = { x: number } & { y: string };`
- [ ] **SOLV-9: Test generic constraint violations**
  - Create tests for bounded generics: `<T extends number>`
  - Ensure `f<string>(123)` errors when string doesn't extend number
  - Test default type parameter inference behavior

## Completed
(none yet)
