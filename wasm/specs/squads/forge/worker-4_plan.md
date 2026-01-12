# Worker 4 Plan - Squad Forge

## Mission
Fix TS2339: Property Does Not Exist

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Available for next task**

### Completed Work
- [x] TS2339 - Fixed WASM discrepancy in private property access (pushed to origin/worker/forge-4)
- [x] TS2454 - Extended definite assignment to var variables (pushed to origin/worker/forge-4)
- [x] TS2705 - Promise<void> return types (merged to squad/forge)
- [x] TS2355 - Async functions false positives (pushed to origin/worker/forge-4)

### TS2355 Fix Details

**Issue**: 6 extra TS2355 errors for async functions without return values

**Root Cause**: TS2355 check didn't account for async functions. Async functions
without return statements implicitly return Promise<void>, so they should not
emit "function must return a value" errors.

**Fix**: Added `!is_async` check to skip TS2355 for async functions in:
- Arrow functions and function expressions
- Function declarations
- Method declarations
- Accessor/getter declarations

**Test Results**:
- Before: 6 extra TS2355 errors for async functions
- After: 0 extra TS2355 errors (eliminated from top 10 extra errors list)

### TS2454 Fix Details

**Root Cause**: `should_check_definite_assignment` only checked `BLOCK_SCOPED_VARIABLE` flag (let/const), excluding `FUNCTION_SCOPED_VARIABLE` (var).

**Fix**: Modified check to include both block-scoped and function-scoped variables:
```rust
// Before: only let/const
if (symbol.flags & symbol_flags::BLOCK_SCOPED_VARIABLE) == 0 {
    return false;
}

// After: both let/const and var
if (symbol.flags & symbol_flags::BLOCK_SCOPED_VARIABLE) == 0
    && (symbol.flags & symbol_flags::FUNCTION_SCOPED_VARIABLE) == 0
{
    return false;
}
```

**Test Results**:
- Before: 29 missing TS2454 errors
- After: 0 missing TS2454 errors
- Introduced: 4 extra TS2454 errors (edge cases)
- Net improvement: 25 errors fixed

**Known Issues**: 4 false positives for variables in nested scopes/closures

### Conformance Analysis (300 tests after TS2454 fix)
- Exact Match: 92 (31.7%)
- Same Error Count: 103 (35.5%)
- Tests with missing errors: 129 (44.5%)
- Tests with extra errors: 122 (42.1%)

### Top Missing Error Codes (Priority Order)
1. TS1109: 16 occurrences - Expression expected
2. TS2524: 15 occurrences - Name not defined
3. TS2304: 11 occurrences - Cannot find name
4. TS1359: 10 occurrences - Identifier expected
5. TS7006: 9 occurrences - Parameter implicitly has 'any' type
6. TS2507: 8 occurrences - Cannot find type
7. TS2664: 7 occurrences - Invalid module name
8. TS2372: 6 occurrences - Index signature missing
9. TS2515: 6 occurrences - Object literal type
10. TS7022: 6 occurrences - Cannot invoke non-function

### Top Extra Error Codes
1. TS2705: 72 occurrences - Async function in ES5/ES3
2. TS7011: 15 occurrences - Arrow function implicit any return
3. TS1005: 13 occurrences - Identifier expected
4. TS2355: 6 occurrences - Property does not exist
5. TS2654: 6 occurrences - Identifier expected
6. TS2300: 4 occurrences - Duplicate identifier
7. TS2454: 4 occurrences - Variable used before assigned (false positives)

### Recent Work
- [x] TS7010 - Implicit any return type (merged to squad/forge)
- [x] TS7006 - Parameter 'any' type (merged to squad/forge)
- [x] TS2322 - Constructor return statement (merged to squad/forge)
- [x] TS2339 - WASM discrepancy fix (merged to squad/forge)
- [x] TS2454 - Var variable definite assignment (merged to squad/forge)
- [x] TS2705 - Promise<void> return types (merged to squad/forge)
- [x] TS2355 - Async functions false positives (pushed to origin/worker/forge-4)

### TS2705 Fix Status

**Issue**: 72 extra TS2705 errors for async arrow functions with Promise<void> return types

**Root Cause**: `type_ref_is_promise_like` didn't handle `TypeKey::Object` types. Promise from lib files is stored as an Object type (interface) rather than a Ref.

**Fix**: Added Object type handling that conservatively returns true for Object types from lib files.

**Status**: ✅ Merged to squad/forge

### Conformance Analysis (300 tests - Current)
- Exact Match: 92 (31.7%)
- Same Error Count: 102 (35.2%)
- Tests with missing errors: 129 (44.5%)
- Tests with extra errors: 120 (41.4%)

## Notes
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
- Focus on errors with highest occurrence counts for maximum impact
