# Worker 11 Task List

Maintained by EM-3

## Active Task
- None awaiting assignment

## Completed Tasks

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
