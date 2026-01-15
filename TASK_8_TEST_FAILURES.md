# Task 8: TS2322 Error Analysis - Specific Test Failures

**Date:** 2025-01-15
**Status:** Analysis Complete

## Summary

Analysis of conformance tests revealed:
- **10 test files with extra TS2322 errors** (false positives)
- **1 test file with missing TS2322 errors** (false negatives)

## False Positive (Extra TS2322 Errors)

### Test Files with Extra Errors:

1. **async/es2017/asyncMethodWithSuperConflict_es6.ts**
   - Errors: `Type '() => void' is not assignable to type 'error'`
   - Pattern: Super call type inference issue
   - Root cause: `error` type appearing instead of proper inferred type

2. **async/es2017/asyncMethodWithSuper_es2017.ts**
   - Same as above

3. **async/es2017/awaitBinaryExpression/awaitBinaryExpression5_es2017.ts**
   - Error: `Type 'unknown' is not assignable to type 'boolean'`
   - Code: `o.a = await p;` where `p: Promise<boolean>` and `o.a: boolean`
   - Root cause: Await expression type inference issue
   - Expected: `await p` should infer `boolean`, not `unknown`

4. **async/es2017/await_incorrectThisType.ts**
   - Error: Complex type not assignable
   - Root cause: Generic type inference issue with Promise-like types

5. **async/es5/asyncMethodWithSuper_es5.ts**
   - Same as #1

6. **async/es5/awaitBinaryExpression/awaitBinaryExpression5_es5.ts**
   - Same as #3

7. **async/es6/asyncMethodWithSuper_es6.ts**
   - Same as #1

8. **async/es6/awaitBinaryExpression/awaitBinaryExpression5_es6.ts**
   - Same as #3

9. **binder_integration.ts**
   - Error: `Type 'error' is not assignable to type 'string'`
   - Root cause: Error propagation when symbols can't be resolved

10. **classes/classDeclarations/classAbstractKeyword/classAbstractUsingAbstractMethod1.ts**
    - Error: `Type 'error' is not assignable to type '{ foo: { (): unknown } }'`
    - Root cause: Abstract method type inference issue

## Key Patterns Identified

### Pattern 1: Type 'error' in Assignability Checks

**Occurrence:** 7 out of 10 files
**Symptom:** `Type 'error' is not assignable to type 'X'`

**Root Cause:**
- When the type checker can't resolve a type, it returns `TypeId::ERROR`
- We then check assignability and emit TS2322 for `error` not being assignable
- TypeScript handles this more gracefully by either:
  - Not emitting the assignability error
  - Emitting a different error (like TS2304) instead

**Example Cases:**
- Super calls in async methods
- Abstract method implementations
- Binder integration tests

**Fix Needed:**
- Check if source or target type is `TypeId::ERROR` before emitting TS2322
- OR emit the resolution error (TS2304) instead of assignability error (TS2322)

### Pattern 2: Await Type Inference Returns `unknown`

**Occurrence:** 3 files (awaitBinaryExpression5)
**Symptom:** `Type 'unknown' is not assignable to type 'boolean'`

**Root Cause:**
- Code: `o.a = await p` where `p: Promise<boolean>`
- Expected: `await p` should infer type `boolean`
- Actual: `await p` infers `unknown`
- Then we check if `unknown` is assignable to `boolean` (it's not)

**Expected TypeScript Behavior:**
- Await expressions should infer the Promise's type argument
- `await Promise<boolean>` should infer `boolean`

**Fix Needed:**
- Improve await expression type inference
- The type of `await p` should be the unwrapping of the Promise type

### Pattern 3: Super Call Type Inference

**Occurrence:** 4 files (asyncMethodWithSuperConflict)
**Symptom:** Method type becomes `error`

**Root Cause:**
- When checking async methods that call `super`
- The method type inference fails
- Results in `error` type instead of proper function type

**Fix Needed:**
- Improve super call type resolution in async methods
- Handle binding of `super` in async contexts properly

## False Negative (Missing TS2322 Errors)

### Test File with Missing Errors:

**classes/classDeclarations/classAbstractKeyword/classAbstractConstructorAssignability.ts**

This test file has 1 missing TS2322 error. More details needed to identify the specific case.

## Next Steps

### Immediate Actions:

1. **Fix Pattern 1 (Type 'error' assignability)**
   - Add check: if source or target is `TypeId::ERROR`, skip TS2322
   - This should fix 7 out of 10 false positives
   - Estimated reduction: 300+ extra TS2322 errors

2. **Fix Pattern 2 (Await type inference)**
   - Investigate await expression type checking
   - Ensure proper Promise type unwrapping
   - Estimated reduction: 100+ extra TS2322 errors

3. **Fix Pattern 3 (Super call inference)**
   - Investigate async method super binding
   - Ensure proper type resolution
   - Estimated reduction: 100+ extra TS2322 errors

### Success Metrics:

Current state:
- Missing: 103 TS2322 errors
- Extra: 593 TS2322 errors

After fixing Pattern 1 (type 'error'):
- Extra: ~300 TS2322 errors (49% improvement)

After fixing all patterns:
- Missing: <20 TS2322 errors (80% improvement)
- Extra: <300 TS2322 errors (50% improvement)

## Files to Modify

1. `wasm/src/thin_checker.rs`
   - Add check for `TypeId::ERROR` before emitting TS2322
   - Improve await expression type inference
   - Fix super call type resolution

2. `wasm/src/solver/operations.rs`
   - May need to improve Promise type unwrapping

3. `wasm/src/solver/subtype.rs`
   - May need to handle ERROR types better in subtype checking
