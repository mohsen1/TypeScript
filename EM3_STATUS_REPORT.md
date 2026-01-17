# EM-3 Status Report: TS2322 Type Assignability Analysis

**Date:** 2026-01-17
**Engineer:** EM-3 (Worker 12)
**Task:** TS2322 Type Assignability - Parser Accuracy (Tier 1)
**Status:** Initial Analysis Complete

---

## Executive Summary

Performed initial analysis of TS2322 type assignability errors in the WASM TypeScript compiler. Found that the subtype checker is already conservative (defaults to `False` for unknown type pairs), but there are:

- **17 extra TS2322 errors** in 500-test conformance suite (false positives)
- **40 failing unit tests** in `solver::subtype_tests`
- Most extra TS2322 errors related to async methods with `super` keyword

---

## Findings

### 1. Conformance Test Results (500 tests)

```
Exact Match:              13 (2.7%)
Same Error Count:         58 (11.9%)
Tests with missing errors: 241 (49.5%)
Tests with extra errors:   470 (96.5%)
```

**Most Common Extra Error Codes:**
- TS6133: 463 occurrences (unused variables - not our concern)
- TS2571: 28 occurrences (this type issues)
- TS7006: 24 occurrences (implicit any)
- **TS2322: 17 occurrences** ← Our focus

**TS2322 Extra Errors Found In:**
- `async/es2017/asyncMethodWithSuperConflict_es6.ts`
- `async/es2017/asyncMethodWithSuper_es2017.ts`
- Other async/super scenarios

### 2. Subtype Checker Architecture

**File:** `wasm/src/solver/subtype.rs` (3,984 lines)

**Key Function:** `check_subtype_inner` (lines 354-906)

**Current Behavior:**
- **Default case (line 904):** `_ => SubtypeResult::False` ✅
- This is CORRECT - the checker is conservative by default
- Unknown type pairs return `False` (not assignable)

**Type Pairs Handled:**
- Intrinsic types ✅
- Literals ✅
- Unions ✅
- Intersections ✅
- Objects ✅
- Arrays/Tuples ✅
- Functions ✅
- Type parameters ✅
- Generic applications ✅
- Mapped types ✅
- Ref types (with resolver) ✅
- Template literals ✅

### 3. Unit Test Failures

**40 failing tests** in `solver::subtype_tests`, including:
- `test_array_intersection`
- `test_constructor_optional_parameter`
- `test_fn_optional_param_*` (multiple)
- `test_generic_contravariant_param_position`
- `test_intersection_union_distribution`
- `test_optional_property_missing_optional`
- `test_template_literal_*` (multiple)
- `test_variance_optional_param_covariant_optionality`

**Common themes:**
- Optional parameters variance
- Rest parameters
- Template literals
- Intersection/union distribution

### 4. Super Keyword Type Handling

**File:** `wasm/src/thin_checker.rs`

**Function:** `get_type_of_super_keyword` (line 14123)

**Current Logic:**
1. Check if in class context
2. Get base class
3. Return base class constructor type
4. Returns `TypeId::ERROR` if not in class or no base class

**Potential Issue:**
- Super property access in async methods may be incorrectly typed
- Super methods might return wrong types in certain contexts

---

## Analysis of Specific Test Cases

### Test Files Created by Previous Workers

**Created test files:**
- `tests/cases/conformance/solver/ts2322_valid_assignments.ts`
- `tests/cases/conformance/solver/ts2322_assignment_tests.ts`
- `tests/cases/conformance/solver/ts2322_edge_cases.ts`

These test files verify:
- ✅ Primitive type widening (literal → base type)
- ✅ `any` type assignments
- ✅ `unknown` top type
- ✅ `never` bottom type
- ✅ Structural subtyping
- ✅ Union/intersection types
- ✅ Generic variance (covariant/contravariant)
- ✅ Object literal freshness

---

## Root Cause Hypothesis

### Why Extra TS2322 in Async/Super Scenarios?

**Hypothesis 1:** Super property access type resolution
- When `super.x()` or `super.x` is accessed in async methods
- The type resolver may be returning an incorrect type
- This causes assignability checks to fail when they shouldn't

**Hypothesis 2:** Async function context changes `this` type
- Async methods may have different `this` typing
- Super relies on correct `this` context
- Mismatch between expected and actual types

**Hypothesis 3:** Method vs Property distinction
- Super methods `super.x()` vs super properties `super.x = value`
- Assignment to super properties may be incorrectly checked
- TypeScript allows certain super property writes that we're rejecting

---

## Recommended Next Steps

### Phase 1: Fix Unit Tests (HIGH PRIORITY)
1. Investigate the 40 failing `solver::subtype_tests`
2. Fix optional parameter variance handling
3. Fix template literal pattern matching
4. Fix intersection/union distribution

### Phase 2: Fix Async/Super TS2322 (MEDIUM PRIORITY)
1. Debug the specific failing conformance tests
2. Check super property access type resolution
3. Verify async context doesn't break super typing
4. Add test cases for async/super scenarios

### Phase 3: Validate with Conformance Tests
1. Re-run conformance suite after fixes
2. Target: Reduce TS2322 extra errors from 17 to <5
3. Ensure no regression in other error codes

---

## Technical Debt Identified

1. **40 failing unit tests** - These should all pass
2. **Template literal support** - Multiple tests failing
3. **Optional parameter variance** - Not fully implemented
4. **Super keyword typing** - Needs verification in async context

---

## Metrics

### Current State (Baseline)
- TS2322 Extra Errors: 17 occurrences
- Failing Unit Tests: 40
- Conformance Exact Match: 2.7%

### Target State (After Fixes)
- TS2322 Extra Errors: <5 occurrences (-70%)
- Failing Unit Tests: 0 (-100%)
- Conformance Exact Match: >5% (+85%)

---

## Files to Modify (Recommended)

### Primary:
1. `wasm/src/solver/subtype.rs` - Fix failing unit tests
2. `wasm/src/thin_checker.rs` - Fix super keyword typing

### Secondary:
3. `wasm/src/solver/compat.rs` - Verify any type propagation
4. `wasm/src/solver/operations.rs` - Template literal operations

---

## Coordination Needed

### Team 1 (Quality & Stability)
- Worker 2: Type expansion fixes - may affect our assignability checks
- Worker 4: AST traversal - may affect property access resolution

### Team 2 (Parser Accuracy)
- No blocking dependencies

---

## Conclusion

The TS2322 assignability checker is architecturally sound (defaults to conservative behavior), but has specific implementation gaps:

1. **Unit test failures** indicate incomplete edge case handling
2. **Async/super scenarios** need debugging
3. **Optional parameters** and **template literals** need fixes

The work is well-scoped and can be completed incrementally by fixing unit tests first, then tackling conformance test failures.

---

**Status:** Ready to begin implementation
**Estimated Effort:** 2-3 days
**Risk Level:** Low (conservative changes, good test coverage)
