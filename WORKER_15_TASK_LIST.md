# Worker 15 Task List

## Squad: CFA (Control Flow Analysis)

## Current Task
- [ ] Fix remaining TS2454 false positive patterns

## Queue
- [ ] Fix TS2454 for try/catch/finally assignment tracking
- [ ] Fix TS2454 for do-while loops
- [ ] Fix TS2454 for never-returning function calls
- [ ] Add comprehensive tests for fixed patterns

## Completed
- [x] Fix TS2454 false positives for any and TypeQuery types
- [x] Analyze patterns where TS2454 fires incorrectly (false positives)
- [x] Fix flow graph to correctly track assignments through control flow
- [x] Implement early return/throw handling (Pattern 4.1, 4.2)

## Context
TS2454 has 225 extra errors (false positives). The CFA is being too strict in some cases. Need to fix flow analysis to match tsc behavior.

---

## Implementation Summary

### TS2454 False Positive Fixes

**Problem:** `check_definite_assignment` was reporting false positives for variables of type `any` and `typeof` expressions.

**Solution:** Enhanced definite assignment analysis to handle these special cases.

**Changes:**
- `wasm/src/thin_checker.rs`:
  - Fixed handling for `any` type variables (37 new lines)
  - Fixed handling for `typeof` type queries
  - Improved early return/throw analysis

- `wasm/differential-test/test-ts2454-extra.mjs`:
  - Added 80 test cases for TS2454 false positive patterns

**Patterns Fixed:**
1. **any type variables**: No longer flagged when unassigned (any allows any usage)
2. **TypeQuery (typeof)**: Properly tracked through control flow
3. **Early return/throw**: BRANCH_LABEL correctly skips unreachable branches
4. **UNREACHABLE paths**: Handled correctly at merge points

**Test Coverage:** 80 new tests verify TS2454 doesn't fire incorrectly.

### Remaining High-Impact Patterns
From the original analysis:
- Pattern 7.1: Finally block assignments (MEDIUM impact)
- Pattern 3.1: Do-while loops (LOW impact)
- Pattern 2.1: Never-returning functions (MEDIUM impact)
