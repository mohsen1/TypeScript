# Worker 11 Task List

Maintained by EM-3

## Active Task

### Task 6: Fix TS2322 Type Accuracy - Balance Missing (105) and Extra (548)

**Priority:** 🔴 CRITICAL (653 total errors: 105 missing + 548 extra)

**Problem:**
TS2322 type errors have significant accuracy issues:
- **105 missing:** Type incompatibilities not detected
- **548 extra:** False positives on valid code
- This indicates type checking is both too permissive and too strict in different areas

**Root Causes (from Task 2 & 3 analysis):**

1. **Missing Errors (105):**
   - Solver bails out on complex types (generics, conditional types, mapped types)
   - Control Flow Analysis gaps (not tracking definite assignments)
   - Type parameter defaults (still using ANY in some places)
   - Intersection type handling
   - Union type compatibility checks

2. **Extra Errors (548):**
   - Over-strict type narrowing
   - Incorrect generic constraint checking
   - Discriminated union type failures
   - Type predicate issues
   - Literal type widening

**Action Items:**

1. **Analyze the 105 Missing TS2322 Errors**
   - Extract failing test cases from conformance results
   - Categorize by root cause:
     - Generic type resolution failures
     - Conditional type evaluation
     - Mapped type handling
     - Union/intersection compatibility
     - CFA-related (properties not known to be assigned)
   - Use `wasm/differential-test/find-missing-ts2322.mjs` to get samples

2. **Analyze the 548 Extra TS2322 Errors**
   - Extract false positive test cases
   - Categorize by pattern:
     - Type narrowing too aggressive
     - Generic constraints over-checked
     - Literal types not widening when they should
     - Discriminant property checks failing
     - Method signature compatibility

3. **Fix Missing Errors (Priority P0)**
   - Fix solver bailouts on complex types
   - Add CFA tracking for:
     - Property assignments in all code paths
     - Variable declarations in closures
     - Array/object destructuring
   - Remove remaining ANY defaults
   - Improve intersection/union type checking

4. **Fix Extra Errors (Priority P1)**
   - Refine type narrowing logic
   - Fix generic constraint checking
   - Properly handle literal type widening
   - Fix discriminant union type checking
   - Improve method signature compatibility

5. **Testing**
   - Run `wasm/differential-test/find-extra-ts2322.mjs`
   - Run `wasm/differential-test/find-missing-ts2322.mjs`
   - Target: Reduce both to <100 each
   - Verify no regressions in passing tests

**Success Criteria:**
- Reduce Missing TS2322 from 105 to <30
- Reduce Extra TS2322 from 548 to <100
- Net improvement: 653 → 130 errors (80% reduction)
- Overall type parity: 28% → 45%+

**Files to Work On:**
- `wasm/src/solver/` - Type resolution and subtyping
- `wasm/src/checker/control_flow.rs` - Definite assignment analysis
- `wasm/src/thin_checker.rs` - Type checking logic
- `wasm/src/checker/narrowing.rs` - Type narrowing (if exists)

**Related Work:**
- Builds on Tasks 2 (solver investigation)
- Builds on Tasks 3 (diagnostic audit)
- Builds on Tasks 4 (ERROR type fix)
- Coordinates with Worker 9 (solver defaults changes)

**Target Branch:** rust

**Testing:**
- Run `./wasm/differential-test/run-conformance.sh --all` after changes
- Focus on `types/*` test category
- Use TS2322 finder scripts to measure improvement

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
