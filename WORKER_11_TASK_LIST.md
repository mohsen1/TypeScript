# WORKER-11 TASK LIST

## Worker: worker-11
## EM: EM-3
## Base Branch: rust
## Team Branch: em-team-3
## Worker Branch: worker-11

---

## Assignment: TS2571 Over-reporting Fix (Priority 2)

### Mission
Fix TS2571 ("Object is of type 'unknown'") false positives that should be TS2683 ("'this' implicitly has type 'any'") instead.

---

## Current Status: 🟡 Assigned 2026-01-15

### Task 1: Investigate TS2571 Emissions
**Status:** ✅ Complete
**Priority:** HIGH

**Objective:** Understand where and why TS2571 is being emitted incorrectly.

**Action Items:**
1. Search for all TS2571 error emissions in wasm/src/
   - Focus: `thin_checker.rs`, `type_checker.rs`
   - Pattern: `add_diagnostic(2571, ...)` or similar
2. Document the conditions that trigger TS2571
3. Identify which should be TS2683 instead
4. Create test cases showing the difference

**Key Context:**
- TS2571: "Object is of type 'unknown'" - for when `this` is typed as unknown
- TS2683: "'this' implicitly has type 'any'" - for when `this` has no type in non-method functions
- Worker 1 already fixed `current_this_type()` for TS2683 in regular functions
- There may be more contexts where TS2683 should be emitted instead of TS2571

**Expected Deliverable:** Document with:
- List of all TS2571 emission points
- Classification: correct vs should be TS2683
- Test file showing 5-10 examples of each case

---

### Task 2: Fix TS2571 → TS2683 Conversion
**Status:** ✅ Complete
**Priority:** HIGH

**Objective:** Modify code to emit TS2683 instead of TS2571 in appropriate contexts.

**Action Items:**
1. Update diagnostic emissions based on Task 1 findings
2. Ensure type inference for `this` in:
   - Arrow functions used as methods
   - Event handlers and callbacks
   - Bound functions
   - Object literal methods
3. Run conformance tests to verify

**Success Criteria:**
- TS2571 extra errors < 50 (down from unknown)
- TS2683 missing errors filled
- No regression in Worker 1's TS2683 fix

---

### Task 3: Validate and Test
**Status:** 🔄 In Progress
**Priority:** MEDIUM

**Action Items:**
1. Run `./wasm/differential-test/run-conformance.sh --max=500`
2. Compare error counts before/after
3. Check for new TS2683 emissions
4. Verify TS2571 count decreased
5. Document any edge cases

**Target Metrics:**
| Error Code | Before | Target |
|------------|--------|--------|
| TS2571 extra | Unknown | < 50 |
| TS2683 missing | Unknown | Fill gaps |

---

## Testing Scenarios to Cover

Create tests for these contexts:
1. `this` in regular function (already fixed by Worker 1)
2. `this` in arrow function assigned to object property
3. `this` in callback passed to another function
4. `this` in nested functions
5. `this` in object literal methods
6. `this` in class methods with @ decorators
7. `this` in bound functions (`.bind(this)`)

---

## Key Files to Modify

Based on investigation:
- `wasm/src/thin_checker.rs` - Worker 1's fix location, may need more changes
- `wasm/src/checker/type_checker.rs` - if it exists and has TS2571 emissions
- Any other files with TS2571 diagnostic code

---

## Commits

When committing, use format:
```
[wasm] checker: <description>

Example:
[wasm] checker: fix TS2571 over-reporting for arrow functions

Changes:
- Detect arrow function contexts in this_type_inference()
- Emit TS2683 instead of TS2571 for arrow functions
- Add test for arrow function this typing

Conformance: TS2571 -X, TS2683 +Y
```

---

## EM-3 Review Criteria

When ready for merge:
1. All tasks completed
2. TS2571 extra errors < 50
3. TS2683 gaps filled
4. No regression in Worker 1 or Worker 2 fixes
5. Test cases added for fixed scenarios
6. Documentation updated

---

## Workflow Reminder

1. Read this file first (always!)
2. Work on CURRENT task only (marked 🔄)
3. Commit frequently with descriptive messages
4. `git add -A && git commit -m "..." && git push origin worker-11`
5. Update status in this file
6. When all tasks complete, notify EM-3 and STOP

**DO NOT:**
- Work on multiple tasks simultaneously
- Skip ahead to future tasks
- Modify TypeScript source files (src/compiler/)
- Push to rust branch directly

---

## Status Log

| Date | Task | Status | Notes |
|------|------|--------|-------|
| 2026-01-15 | Task 1 | ✅ Complete | Found root cause at thin_checker.rs:9812-9817, created ts2571_investigation.md |
| 2026-01-15 | Task 2 | ✅ Complete | Implemented fix: arrow functions inherit outer `this`, regular functions use ANY instead of UNKNOWN |
| 2026-01-15 | Task 3 | 🔄 In Progress | Awaiting EM-3 merge to run conformance tests |
