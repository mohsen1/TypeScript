# Task 8: Fix TS2322 Type Compatibility Errors

**Status:** 🔄 IN PROGRESS
**Started:** 2025-01-15
**Priority:** HIGH (103 missing + 593 extra = 696 total TS2322 errors)

## Baseline Results (2025-01-15)

**Conformance Test Results:**
- **TS2322 Missing:** 103 occurrences (down from 179 - 42% improvement from Worker 11 Task 3)
- **TS2322 Extra:** 593 occurrences (we're too strict in some cases)
- **Combined:** 696 TS2322 errors

**Most Common Missing Error Codes:**
1. TS2792: 161 occurrences (import/export module resolution)
2. TS2304: 114 occurrences (cannot find name)
3. **TS2322: 103 occurrences** ← **Focus of this task**
4. TS1005: 96 occurrences (token expected)
5. TS2339: 79 occurrences (property does not exist)

**Most Common Extra Error Codes:**
1. **TS2322: 593 occurrences** ← **Also focus of this task**
2. TS7005: 425 occurrences (implicit 'any')
3. TS7008: 336 occurrences (implicit 'any' in module)
4. TS2304: 313 occurrences
5. TS2571: 262 occurrences (object is 'any')

## Problem Statement

The Rust implementation has 103 missing TS2322 errors where TypeScript emits them but we don't, and 593 extra TS2322 errors where we emit them but TypeScript doesn't. This indicates both:
1. **Incomplete type checking** - we're not checking assignability in some cases where TypeScript does
2. **Over-strict type checking** - we're checking assignability when TypeScript doesn't, or using stricter rules

## Previous Work

**Worker 11 Task 3** (2024-01-14) already made significant progress by:
- Removing diagnostic suppression for TS2322 errors
- Fixed ~310 missing TS2322 errors
- Improved conformance by ~14pp

The remaining 103 missing errors are due to other causes beyond diagnostic suppression.

## Current State Analysis

### TS2322 Emission Points Found

The following functions properly check assignability and emit TS2322 errors:

1. **`check_assignment_expression`** (line 6777)
   - Checks assignment `left = right`
   - Calls `error_type_not_assignable_with_reason_at()` when types don't match
   - ✅ Correctly implemented

2. **`check_return_statement`** (line 15704)
   - Checks return statement value type
   - Calls `error_type_not_assignable_with_reason_at()` when return type doesn't match
   - ✅ Correctly implemented

3. **`error_type_not_assignable_at`** (line 13089)
   - Reports basic TS2322 error
   - Uses solver's diagnostic builder for detailed error messages
   - ✅ Diagnostic suppression already removed by Worker 11

4. **`error_type_not_assignable_with_reason_at`** (line 13131)
   - Reports TS2322 with detailed elaboration
   - Uses solver's "explain" API for better error messages
   - ✅ Diagnostic suppression already removed by Worker 11

### Assignability Check Functions

- **`is_assignable_to`** (line 11423) - Main assignability check function
- **`is_subtype_of`** (line 11502) - Stricter subtype check
- **`should_skip_weak_union_error`** (line 11478) - Skips weak type violations (TypeScript behavior)

## Root Cause Analysis

### Missing TS2322 Errors (103)

Possible causes:
1. **Missing assignability checks** in specific contexts:
   - Function call argument type checking
   - Array element type checking
   - Generic type parameter constraint checking
   - Spread operator type checking
   - Destructuring assignment type checking

2. **Incorrect type inference** causing wrong types to be checked

3. **Incorrect assignability rules** - using `is_subtype_of` instead of `is_assignable_to` in some cases

### Extra TS2322 Errors (593)

Possible causes:
1. **Over-strict assignability rules** - not respecting TypeScript's special cases:
   - Weak type detection (already implemented via `should_skip_weak_union_error`)
   - Dual function types
   - Bivariant parameter types
   - Enum assignability

2. **Assignability checks in wrong contexts**:
   - Checking when TypeScript doesn't (e.g., in type positions)
   - Not respecting contextual typing

3. **Type inference issues** causing inferred types to be more specific than TypeScript's

## Action Plan

### Phase 1: Identify Specific Failing Tests

1. Extract specific test cases showing missing TS2322 errors
2. Extract specific test cases showing extra TS2322 errors
3. Categorize by pattern (function calls, arrays, generics, etc.)

### Phase 2: Fix Missing TS2322 Errors

1. **Add missing assignability checks** in contexts where TypeScript checks but we don't:
   - Function call arguments
   - Array literal elements
   - Spread operands
   - Destructuring assignments
   - Generic type arguments

2. **Fix type inference** issues causing wrong types

### Phase 3: Fix Extra TS2322 Errors

1. **Implement missing special cases** in assignability:
   - Better weak type detection
   - Dual function type handling
   - Bivariant parameter handling
   - Enum assignability overrides

2. **Fix over-strict checking**:
   - Skip checks where TypeScript doesn't check
   - Respect contextual typing better

### Phase 4: Validate and Test

1. Run full conformance test suite
2. Verify no regression in passing tests
3. Measure improvement in TS2322 metrics
4. Target: <20 missing, <300 extra (50% improvement)

## Files to Work On

- `wasm/src/thin_checker.rs`
  - Assignability check functions (already correct)
  - Need to add checks in missing contexts
  - Need to fix over-strict checks

- `wasm/src/solver/compat.rs`
  - Type compatibility checking
  - Subtype checking

- `wasm/src/solver/subtype.rs`
  - Subtype relationship checking

## Related Work

- Builds on Worker 11 Task 3 (diagnostic suppression removal)
- Coordinates with Worker 1 (parser error suppression)
- Coordinates with Worker 5 (expression end detection)

## Success Criteria

- Reduce Missing TS2322 from 103 to <20 (80% improvement)
- Reduce Extra TS2322 from 593 to <300 (50% improvement)
- Combined improvement: 696 → <320 errors (54% improvement)
- Overall conformance improvement: 31.4% → 35%+

## Testing

- Run `./wasm/differential-test/run-conformance.sh --all` after changes
- Focus on TS2322-specific test failures
- Measure improvement in exact match percentage
- Ensure no regression in other error codes
