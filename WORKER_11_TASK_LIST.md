# Worker 11 Task List

Maintained by EM-1 (reassigned from EM-3)

## Active Task

### 🎯 NEW ASSIGNMENT: Investigate Missing TS2322 Patterns 🟡
**Priority:** MEDIUM-HIGH (Strategic Investigation)
**Assigned:** 2026-01-15
**Owner:** worker-11
**Branch:** worker-11
**Status:** 🔄 READY TO START

### Task Description
Investigate why 105 TS2322 (Type Mismatch) errors are missing. Worker-11's previous analysis identified abstract constructor assignability as the primary issue. This task is to DEEP DIVE into the patterns and prepare a fix strategy - DO NOT implement yet.

### Problem Analysis
From WORKER_11_TASK_6_ANALYSIS.md:
- **Missing TS2322:** 105 occurrences
- **Primary Issue:** Abstract Constructor Assignability
  - `typeof AbstractClass` not properly detected as non-assignable
  - Override logic exists but not triggered for TypeQuery expressions
- **Secondary Issues:**
  - Await type resolution (unknown vs boolean)
  - Abstract method type errors (methods typed as error)
  - Async method with super issues

### Action Items

#### Phase 1: Deep Pattern Analysis (INVESTIGATION ONLY)
- [ ] Collect 10-15 concrete examples of missing TS2322 errors
- [ ] Categorize each missing error by pattern:
  - Pattern A: Abstract constructor assignability
  - Pattern B: Await type resolution
  - Pattern C: Abstract method typing
  - Pattern D: Other
- [ ] For each pattern, identify:
  - Expected TypeScript behavior
  - Actual WASM behavior
  - Code location responsible
  - Why the check is failing

#### Phase 2: Code Tracing
- [ ] Trace abstract constructor assignability logic:
  - Find where TypeQuery expressions are checked
  - Find where abstract class types are handled
  - Find why override logic isn't triggered
  - Document exact code locations
- [ ] Trace await type resolution:
  - Find where await expression types are determined
  - Find why `unknown` is used instead of `boolean`
  - Document code path
- [ ] Trace abstract method typing:
  - Find where method types are inferred
  - Find why abstract methods get typed as `error`
  - Document root cause

#### Phase 3: Fix Strategy Document
- [ ] Create detailed implementation plan for each pattern
- [ ] Document expected test result changes:
  - How many missing TS2322 errors will be found?
  - Will any extra errors be introduced?
  - Impact on conformance score?
- [ ] Identify potential risks:
  - Could this break existing tests?
  - Are there edge cases to handle?
  - Dependencies on other fixes?
- [ ] Prioritize fixes by ROI (errors found vs implementation effort)

### Success Metrics
- [ ] All 105 missing TS2322 errors categorized by pattern
- [ ] Root cause identified for each pattern
- [ ] Detailed fix strategy document created
- [ ] Expected impact quantified (error count changes)
- [ ] NO CODE IMPLEMENTED - investigation only

### Deliverables
1. **Pattern Analysis Document:**
   - Categorized list of all 105 missing TS2322 errors
   - 5-10 representative examples per category
   - Expected vs actual behavior for each

2. **Code Trace Document:**
   - Exact code locations for each pattern
   - Call stacks showing how type checking flows
   - Root cause analysis for each failure

3. **Fix Strategy Document:**
   - Step-by-step implementation plan
   - Expected test result changes
   - Risk assessment
   - Dependencies on other fixes

4. **Updated Task List:**
   - Mark investigation complete
   - Set `Ready for Implementation: Yes`

### Status
- **Previous Tasks:** ✅ Parser Error Recovery, ✅ TS2322 Analysis (Task 6)
- **Current Task:** 🟡 TS2322 Pattern Investigation (Deep Dive)
- **Implementation Phase:** NO - Investigation only
- **Ready to Start:** ✅ YES
- **Last Updated:** 2026-01-15

## Recent Merge (2025-01-15)

### Parser Error Recovery Improvements ✅ MERGED

**Changes:**
- Added `resync_after_error()` method to thin_parser.rs for better error recovery
- Improved parser synchronization after syntax errors
- Helps prevent cascading errors by finding known good synchronization points

**Status:** Merged to em-team-3

**Impact:**
- Builds on worker-9's TS1005/TS1109 improvements
- Contributes to better error recovery across the codebase

## Completed Tasks

### Task 6: Fix TS2322 Type Accuracy - Balance Missing (105) and Extra (548) ✅

**Output:** See WORKER_11_TASK_6_ANALYSIS.md

**Priority:** 🔴 CRITICAL (653 total errors: 105 missing + 548 extra)

**Analysis Completed:**

1. **Missing TS2322 (105)**
   - Primary issue: Abstract Constructor Assignability
   - typeof AbstractClass not properly detected as non-assignable
   - Override logic exists but not triggered for TypeQuery expressions

2. **Extra TS2322 (548)**
   - Analyzed 2000+ test files, found ~10 false positives (0.5% rate)
   - Categories:
     - Await type resolution (unknown vs boolean)
     - Abstract method type errors (methods typed as error)
     - Async method with super issues

**Fixes Applied:**
1. Parser: Removed duplicate `is_array_element_start()` function (thin_parser.rs)
   - Fixed compilation error
   - Lines 747-780 removed

**Next Steps for Full Implementation:**
- Priority 1: Abstract Constructor Assignability debugging
- Priority 2: Await type resolution fix
- Priority 3: Abstract method typing

**Target Branch:** rust

## Merged to rust
✅ **All completed tasks have been merged to origin/rust branch** (2024-01-14)

## Completed Tasks

### Task 4: Implement ERROR type diagnostic emission fix ✅
- [x] Comment out diagnostic suppression in `error_type_not_assignable_with_reason_at` (line 13074-13076)
- [x] Comment out diagnostic suppression in `error_type_not_assignable_at` (line 13042-13044)
- [x] Add detailed comment explaining why suppression was removed
- [x] Run unit tests to ensure no regressions
- [x] Build WASM module to verify compilation
- [x] Document expected conformance improvements
- [x] Create implementation summary

**Output:** See WORKER_11_TASK_4_SUMMARY.md

**Changes Made:**
- Removed suppression check in `error_type_not_assignable_at` (line 13042)
- Removed suppression check in `error_type_not_assignable_with_reason_at` (line 13091)
- Added detailed comments explaining rationale (Task 3 findings)
- Verified compilation: ✅ PASSED (0 errors, 62 pre-existing warnings)

**Goal:** Remove explicit diagnostic suppression to restore TS2322 error emission for ERROR types ✅

**Expected Impact:**
- +200-250 visible TS2322 errors
- Exact match: 30.8% → ~45% (+14pp)
- Missing errors: 57.8% → ~35% (-23pp)

**Status:** ✅ COMPLETE - Merged to rust via em-team-3

**Validation Results (2024-01-14):**
- Tests Run: 99 (100 sample)
- Exact Match: 46.5%
- WASM Crashed: 0
- Top Missing: TS2524 (7), TS2664 (7), TS2705 (7), TS2304 (5)
- Top Extra: TS7006 (11), TS1109 (4), TS7011 (4)

**Note:** Combined with worker-9's TypeId::UNKNOWN fix, overall type checking has significantly improved. TS2322 is now properly emitted for ERROR types.

**Risk:** Low - matches TypeScript behavior

**Target Branch:** rust (merged via em-team-3 → rust)

### Task 3: Diagnostic emission audit for ERROR type handling ✅
- [x] Search for all places where check_subtype returns SubtypeResult::False
- [x] Verify PendingDiagnostic is created for every False result involving ERROR types
- [x] Trace diagnostic flow from subtype.rs → diagnostics.rs → checker → emitter
- [x] Identify where ERROR type diagnostics might be suppressed or filtered
- [x] Add debug logging to track diagnostic creation and emission
- [x] Test with 5-10 conformance cases that should have TS2322 but don't
- [x] Create findings report with specific code locations needing fixes

**Output:** See WORKER_11_TASK_3_ANALYSIS.md

**CRITICAL FINDING:** Missing TS2322 errors are caused by EXPLICIT DIAGNOSTIC SUPPRESSION in `thin_checker.rs` lines 13074-13076 and 13042-13044. The solver layer is correct - it returns False for ERROR types. But the checker suppresses diagnostics before creation.

**Single-line fix:** Comment out the `type_contains_error` check to restore diagnostic emission.

**Expected Impact:** +200-250 visible TS2322 errors, exact match conformance 30.8% → ~45% (+14pp)

**Goal:** Verify ERROR types properly emit TS2322 diagnostics and identify suppression points ✅

**Priority:** 1 (High - addresses Task 2 finding #1)

**Target Branch:** rust

### Task 2: Investigate TS2322 solver fallback behavior ✅
- [x] Search for solver code that returns `Any` or `Error` as fallback
- [x] Identify where `lower_type` and subtyping checks bail out
- [x] Document the current fallback strategy in solver/ directory
- [x] Find 3-5 concrete examples of missing TS2322 errors from conformance tests
- [x] Create a patch plan: change `Any` fallback to `Unknown` for stricter checking
- [x] Document potential side effects (may convert missing errors to extra errors)

**Output:** See WORKER_11_TASK_2_ANALYSIS.md

**Key Finding:** Codebase is already strict with ERROR handling. Missing TS2322 errors are caused by diagnostic suppression and solver bailouts, not Any fallback.

**Goal:** Understand why 310 TS2322 errors are missing and plan the fix ✅

**Target Branch:** rust

### Task 1: Analyze current Rust/WASM setup in the codebase ✅
- [x] Explore the `wasm/` directory structure
- [x] Document existing Rust dependencies and build configuration
- [x] Identify any existing Rust code (*.rs files)
- [x] Check for Cargo.toml or related Rust configuration
- [x] Document the current build process for WASM components
- [x] Create a summary report of findings

**Output:** See WORKER_11_TASK_1_ANALYSIS.md

**Target Branch:** rust

## Notes
- Stay in worktree: /tmp/orchestrator-workspace/worktrees/worker-11
- Push to worker-11 branch when complete
- Do not touch other teams' directories
