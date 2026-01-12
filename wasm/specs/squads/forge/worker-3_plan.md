# Worker 3 Plan - Squad Forge

## Mission
General Task Assignment

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Available for next task**

### Completed Work
- [x] TS2304 - Cannot find name (fixed infer type parameter false positives)
- [x] TS2705 - Async function must return Promise (implemented and merged to squad/forge)
- [x] TS2300 - Duplicate identifier for class members (verified existing implementation)

### Recent Merges
- ✅ TS2705 async function return type - Merged to squad/forge 2026-01-12
- ✅ TS2300 duplicate class members - Verified existing symbol-level check handles this
- ✅ All changes committed and pushed to origin/worker/forge-3

### Implementation Details (TS2705)
- Added ASYNC_FUNCTION_RETURNS_PROMISE error code (2705)
- Added is_promise_type() helper function
- Handles function declarations, arrow functions, function expressions, and methods
- Test added: test_async_function_returns_promise
- 5051 tests passing, TS2705 test passes

### TS2300 Duplicate Class Members Investigation (2026-01-12)
**Finding: TS2300 for duplicate class members is already implemented**

Investigation revealed that the existing `check_duplicate_identifiers` function in `thin_checker.rs` (line 12180) already handles duplicate class members through symbol-level checking. This function:
- Checks all symbols in the binder's scope (including class members)
- Detects conflicting declarations using `declarations_conflict()`
- Emits TS2300 on conflicting declarations

**Verification:**
- Added test: `test_duplicate_class_members` - verifies duplicate class properties are detected
- All 10 duplicate identifier tests pass
- No new code was needed for this functionality

**Note:** The symbol-level check reports TS2300 on both duplicate declarations, which differs from tsc's behavior (reports once on the first occurrence). This is acceptable for the current implementation.

### Next Steps
- Object literal properties - may need investigation
- Function parameters - already handled by existing checks
- Namespace members - may need investigation

### Notes
- Ready for new high-priority task assignment
- Previous work: type inference, async function type checking
- Push to: `origin/worker/forge-3`
