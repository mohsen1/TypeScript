# Worker 11 Task List

Maintained by EM-3

## Active Task
- None awaiting assignment

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
