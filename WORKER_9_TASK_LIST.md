# Worker 9 Task List - Solver Squad

## Current Task
- [ ] **SOLV-21: Add conditional type handling**
  - Implement conditional type evaluation in solver
  - Handle `T extends U ? X : Y` correctly
  - Test conditional type distributivity

## Queue
- [ ] **SOLV-22: Fix template literal type checking**
  - Implement template literal type inference
  - Handle string literal unions in template literals
  - Test: `` `hello-${T}` `` type resolution

## Completed
- [x] **SOLV-9: Test generic constraint violations**
  - Created tests for bounded generics: `<T extends number>`
  - Ensured `f<string>(123)` errors when string doesn't extend number
  - Tested default type parameter inference behavior
  - Added test cases to `instantiate_tests.rs`, `intern_tests.rs`, `lower_tests.rs`
- [x] **SOLV-6: Fix intersection type checking**
  - Implemented `solve_intersection_subtype` correctly
  - Intersection A is subtype of B if ANY member of A is subtype of B
  - Tested `type A = { x: number } & { y: string };`
- [x] **SOLV-3: Strengthen union type checking**
  - Audited `solve_union_subtype` implementation
  - Fixed union subtyping: Union A is subtype of Union B only if all A's members are in B
  - Removed `any` as universal subtype in unions
  - Added union type tests in `union_tests.rs`
