# WORKER-2 TASK LIST

## Team Assignment
- **EM:** EM-1
- **Branch:** worker-2
- **Parent:** em-team-1

## Priority: Fix Global Scope (TS2304)

### Mission
Fix the root cause of "error poisoning" - missing global symbols like `console`, `Promise`, `Array` cause cascading failures where undefined symbols are treated as `Any`, suppressing legitimate type errors.

### Current Data
- **Extra TS2304:** 343 errors ("Cannot find name 'X'")
- **Missing TS2304:** 116 errors
- **Root Cause:** lib.d.ts not loaded correctly in test runner
- **Target:** <10 extra errors

---

## Previous Work Completed ✅

### 1. Fix `file_locals` Population from Library Context
**Status:** ✅ COMPLETE
**File:** `src/thin_binder.rs`
**Commit:** `f0103f305`

Lib symbols are now preserved across binding process with user symbol precedence.

### 2. Ensure Global Symbols Are Accessible in All Files
**Status:** ✅ COMPLETE (addressed by Task 1)

---

## Current Tasks

### Task 1: Verify Lib.d.ts Loading in Test Runner
**Priority:** CRITICAL
**Files:** `wasm/src/integration/`, test runner

Even though lib symbol preservation is working, we need to verify:
1. Where and how lib.d.ts is loaded in the test runner
2. Whether it's being parsed correctly
3. Whether global symbols are registered before type checking
4. Check for any merge conflicts or duplicate symbol issues

**Acceptance Criteria:**
- `console`, `Promise`, `Array`, `Object` available in all tests
- Extra TS2304 errors drop below 10
- Test runner shows lib.d.ts loaded in compilation context

### Task 2: Fix Global Merging Across Files
**Priority:** CRITICAL
**File:** `wasm/src/binder/`

Ensure global interfaces merge correctly:
1. `interface Window` from lib.d.ts
2. `interface Window` from user code
3. Module globals vs. script globals
4. Augmentation across multiple files

**Acceptance Criteria:**
- Global interfaces merge without conflicts
- Augmented globals (e.g., `Window`) properly extend base types
- No symbol shadowing issues

### Task 3: Investigate Missing TS2304 Errors
**Priority:** HIGH
**Analysis Required:** We have 116 MISSING TS2304 errors

These are cases where tsc emits "Cannot find name" but we don't. Possible causes:
1. We're falling back to `Any` instead of emitting error
2. Our symbol resolution is more lenient
3. We have different scoping rules

**Acceptance Criteria:**
- Document why we're missing these errors
- Fix if it's a legitimate bug
- Document if it's intentional behavior difference

---

## Deliverables
1. Verified lib.d.ts loading in test runner
2. Corrected global merging logic
3. Analysis of missing TS2304 errors
4. Conformance test results showing TS2304 reduction

## Success Metric
Reduce extra TS2304 errors from **343 to <10**.

## Merge Status

### 2026-01-14 - Merge Attempt
**Status:** No commits to merge
**Result:** worker-2 branch is at same commit as em-team-1 (17b30883e)
**Action:** No code changes found on branch

### Tasks Completed
- Previous work (commit f0103f305) already in history - lib symbol preservation
- No new code commits detected

### Test Results
- No tests run - no changes to validate

### Next Steps
- Worker-2 needs to commit global scope fixes for merge
- OR awaiting Director reassignment

---

## Notes
- Global scope issues cause cascading failures - fix this first
- Coordinate with worker-3 (solver strictness) to validate downstream effects
- Reference TypeScript's lib loading logic
- Previous work on lib symbol preservation is merged (commit 95153ca2a)
