# Worker 1 Task List

Maintained by EM-1

## Current Tasks

### [PENDING] Task 2: Fix missing-ts2322-by-category.txt - Tuple Category
- Status: Not Started
- Description: Fix all TS2322 (Type 'X' is not assignable to type 'Y') errors in the **Tuple** category from `missing-ts2322-by-category.txt`
- Details:
  1. Locate and analyze the "Tuple" section in missing-ts2322-by-category.txt
  2. For each test case, understand the expected behavior
  3. Implement fixes in the TypeScript compiler (likely in `wasm/src/thin_checker.rs` or related type checking files)
  4. Add or modify test cases to verify the fixes
  5. Run tests to ensure no regressions

---

## Completed Tasks

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
