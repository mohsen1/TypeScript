# Worker 1 Task List

Maintained by EM-1

## Current Tasks

*No active tasks*

---

## Completed Tasks

### [COMPLETED] Task 2: Verify TS2322 Tuple Type Assignability

**Status:** Complete

**Description:**
Verified that TS2322 (Type 'X' is not assignable to type 'Y') errors for tuple type assignability are correctly emitted by the thin checker.

**Investigation Findings:**
- The `missing-ts2322-by-category.txt` file does NOT contain a "Tuple" category
- This is because tuple-related TS2322 errors are NOT missing - they are already being emitted correctly
- The thin checker properly handles tuple type assignability through `check_tuple_subtype` in `wasm/src/solver/subtype.rs`

**Test Coverage Verified:**
- `test_tuple_array_assignability_in_checker` - Confirms array-to-tuple assignment errors
- All 7 tuple subtyping integration tests pass:
  - `test_named_tuple_elements`
  - `test_tuple_length_mismatch_fails`
  - `test_tuple_element_variance`
  - `test_tuple_with_optional_elements`
  - `test_tuple_to_array_assignability`
  - `test_tuple_with_rest_element`
  - `test_tuple_covariant_subtyping_same_length`
- 200+ tuple-related tests exist across the codebase

**Examples of Correctly Emitted TS2322 Errors:**
```typescript
type Tup = [string, number];
const tup: Tup = ["a", 1];           // OK
const arr: (string | number)[] = tup;  // OK - tuple to array
const bad: Tup = arr;                  // TS2322 - array to tuple not assignable
```

**Validation:**
- No code changes required - functionality already working correctly
- Extensive test coverage confirms tuple TS2322 errors are properly emitted

---

### [COMPLETED] Task 1: Implement TS2683 for Implicit `this` in Functions

**Priority:** High (Tier 2 - Type Checker Accuracy)

**Status:** Complete

**Commit:** c958fc9cb - "fix: implement TS2683 for implicit this in functions"

**Description:**
Fixed the type checker to correctly emit TS2683 ("'this' implicitly has type 'any'") when `this` is used inside a regular function (not a method). Previously, it was incorrectly typed as `unknown` and emitted TS2571 on property access.

**Error Example Fixed:**
```typescript
function foo() {
    this.x = 1;  // Now emits: TS2683: 'this' implicitly has type 'any'
                 // Previously emitted: TS2571: Object is of type 'unknown'
}
```

**Changes Made:**
- Modified `wasm/src/thin_checker.rs` - `current_this_type()` handling
- Added proper detection for `this` usage in non-method functions
- Ensured TS2683 is emitted instead of TS2571 for this case

**Validation:**
- Code committed to worker-1 branch
- Merged into em-team-1
- Merged into rust branch (c958fc9cb)

---

## Notes

- **Worktree:** `/tmp/orchestrator-workspace/worktrees/worker-1`
- **Branch:** `worker-1`
- **Target branch:** `rust`
- **Squad:** Type Squad

---

## Previous Activity Log

### 2026-01-15
- Created WORKER_1_TASK_LIST.md template
- Completed TS2683 fix (c958fc9cb)
- Merged to em-team-1
- Assigned Task 2: Fix TS2322 Tuple category
- **Task 2 Completed**: Verified tuple TS2322 errors are correctly emitted (no missing errors)
