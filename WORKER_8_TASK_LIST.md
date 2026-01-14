# Worker 8 Task List

**Maintained by:** EM-2
**Branch:** worker-8 → rust
**Worktree:** /tmp/orchestrator-workspace/worktrees/worker-8

---

## Completed Tasks

### ✅ Task 1: TS2454 - Variable Use Before Assignment Detection

**Status:** COMPLETED (2026-01-14)
**Commit:** fd6567c22

Fixed critical bug where TS2454 errors were not reported for variables used as arguments in call expressions when the callee returned ANY/ERROR type.

### ✅ Task 2: TS2564 - Property Initialization Detection

**Status:** ALREADY IMPLEMENTED
**All 12 unit tests pass**

The TS2564 functionality was already fully implemented in the codebase with comprehensive coverage for all edge cases.

### ✅ Task 4: TS2339 - Reduce False Positives

**Status:** ALREADY IMPLEMENTED
**All 22 TS2339 tests + apparent members test pass**

The apparent members functionality for primitives was already fully implemented.

---

## Current Task (IN PROGRESS)

### Task 5: Run Conformance Tests and Identify Remaining Issues

**Priority:** HIGH
**Goal:** Analyze conformance test results to identify specific gaps in type checking

#### Requirements
1. Build WASM module: `cd wasm && ./build-wasm`
2. Run conformance tests: `./differential-test/run-conformance.sh --max=10000`
3. Analyze "missing errors" and "extra errors" categories
4. Identify top 5 error codes with the most discrepancies
5. Document specific test cases that should pass/fail

#### Acceptance Criteria
- [ ] Conformance test baseline established
- [ ] Top 5 missing error codes identified
- [ ] Top 5 extra error (false positive) codes identified
- [ ] Specific test cases documented for each

#### Expected Deliverable
Create `WORKER_8_CONFORMANCE_ANALYSIS.md` with:
- Overall conformance test results
- Top error codes with discrepancies
- Specific failing test examples
- Recommendations for next tasks

---

## Queue (Future Tasks)

### Task 6: Fix Top Missing Error from Conformance Analysis

**Priority:** HIGH
**Approach:** Based on Task 5 findings, fix the most impactful missing error

### Task 7: Fix Top False Positive from Conformance Analysis

**Priority:** MEDIUM
**Approach:** Based on Task 5 findings, reduce the most common false positive

### Task 8: Improve Enum Type Checking

**Priority:** MEDIUM
**Error Codes:** TS2322, TS2339 related to enums
**Focus:** Enum member access, implicit enum widening, const enum behavior

### Task 9: Enhance Generic Constraint Checking

**Priority:** MEDIUM
**Error Codes:** TS2344, TS2345 related to generics
**Focus:** Generic constraint satisfaction, generic defaults

---

## Anti-Priorities (DO NOT WORK ON)
- New emitter transforms (ES3, obscure module formats)
- LSP features (semantic tokens, code actions)
- CLI argument parsing
- Performance micro-optimizations

---

## Status Updates
- **Created:** 2026-01-14
- **Last Updated:** 2026-01-14
- **Current Focus:** Conformance test analysis
- **Progress:** 4/9 tasks complete (44%)

---

## Previous Summary

### Work Completed (2026-01-14)
1. **Task 1 (TS2454)**: Fixed critical bug - definite assignment now checked for call arguments
2. **Task 2 (TS2564)**: Verified already fully implemented
3. **Task 3 (TS2322)**: Investigated - requires architectural decision
4. **Task 4 (TS2339)**: Verified apparent members already fully implemented

### Key Achievement
**Fixed 1 critical bug (TS2454)** that prevented variable use-before-assignment errors from being reported in call arguments.
