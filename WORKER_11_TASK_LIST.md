# Worker 11 Task List

Maintained by EM-3

## Active Task

### Task 3: Diagnostic emission audit for ERROR type handling
- [ ] Search for all places where check_subtype returns SubtypeResult::False
- [ ] Verify PendingDiagnostic is created for every False result involving ERROR types
- [ ] Trace diagnostic flow from subtype.rs → diagnostics.rs → checker → emitter
- [ ] Identify where ERROR type diagnostics might be suppressed or filtered
- [ ] Add debug logging to track diagnostic creation and emission
- [ ] Test with 5-10 conformance cases that should have TS2322 but don't
- [ ] Create findings report with specific code locations needing fixes

**Goal:** Verify ERROR types properly emit TS2322 diagnostics and identify suppression points

**Priority:** 1 (High - addresses Task 2 finding #1)

**Target Branch:** rust

## Completed Tasks

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
