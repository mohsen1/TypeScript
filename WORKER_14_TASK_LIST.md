# Worker 14 Task List

## Squad: CFA (Control Flow Analysis)

## Current Task
- [ ] Fix TS2564 for class property definite assignment in constructors

## Queue
- [ ] Handle async/callback patterns correctly
- [ ] Fix TS2564 for property assignments in conditional branches
- [ ] Fix TS2564 for properties assigned in parent constructors

## Completed
- [x] Fix TS2564 edge cases: class expressions and Symbol computed keys
- [x] Analyze patterns where TS2564 should fire but doesn't
- [x] Fix property initialization tracking in constructors for edge cases

## Context
TS2564 is the #1 missing error with 413 occurrences. The CFA framework is in place but edge cases need work. Focus on class property initialization.

---

## Implementation Summary

### TS2564 Edge Case Fixes

**Problem:** Class properties in certain patterns were not being flagged as definitely assigned when they should be.

**Solution:** Enhanced definite assignment analysis in `wasm/src/thin_checker.rs`.

**Changes:**
- `wasm/src/thin_checker.rs`:
  - Fixed class expression property assignment tracking (33 new lines)
  - Added handling for Symbol-keyed properties
  - Improved constructor analysis for edge cases

- `wasm/src/thin_checker_tests.rs`:
  - Added 170 new test cases for TS2564 edge cases

**Patterns Fixed:**
1. Class expressions: Properties now checked correctly
2. Symbol computed keys: `[Symbol.foo]: type` now handled
3. Complex constructor patterns: Conditional assignments tracked better

**Test Coverage:** 170 new tests verify TS2564 fires correctly for edge cases.

### Remaining Work
Still need to fix:
- Async/await patterns in constructors
- Callback-based initialization
- Complex conditional branches
- Parent constructor property merging
