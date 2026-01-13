# Worker 9 Task List - Solver Squad

## Current Task
- [ ] **SOLV-22: Fix template literal type checking**
  - Implement template literal type inference
  - Handle string literal unions in template literals
  - Test: `` `hello-${T}` `` type resolution

## Queue
  - Implement template literal type inference
  - Handle string literal unions in template literals
  - Test: `` `hello-${T}` `` type resolution
- [ ] **SOLV-34: Implement conditional type evaluation**
  - Evaluate `T extends U ? X : Y` correctly
  - Handle distributive conditional types over unions
  - Test conditional type inference
- [ ] **SOLV-35: Add mapped type handling**
  - Implement `[K in keyof T]: U` mapped types
  - Handle readonly and optional modifiers in mapped types
  - Test: `{ readonly [P in keyof T]: T[P] }`
- [ ] **SOLV-36: Fix infer keyword in conditional types**
  - Handle `infer R` in conditional type constraints
  - Test type inference with `infer` in return types
  - Verify `ReturnType<T>` utility type works correctly

## Completed
- [x] **SOLV-21: Add conditional type handling**
  - Implemented conditional type evaluation in solver
  - Handle `T extends U ? X : Y` correctly
  - Added conditional type support to evaluate.rs
  - Test conditional type distributivity
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
