# Worker 11 Task List

Maintained by EM-3

## Active Task

### Task 7: Fix TS2322 Type Accuracy - Balance Missing and Extra Errors

**Priority:** 🔴 CRITICAL (167 total errors: 48 missing + 119 extra)

**Current Baseline (from 2025-01-15 conformance tests):**
- **Missing TS2322:** 48 occurrences (2.6% of 1822 tests)
- **Extra TS2322:** 119 occurrences (6.5% of 1822 tests)
- **Net imbalance:** 71 extra errors

**Problem:**
TS2322 (Type 'X' is not assignable to type 'Y') has accuracy issues in both directions:

1. **Missing Errors (48):** Type incompatibilities not detected
   - Most likely causes:
     - Generic type resolution failures
     - Conditional type evaluation gaps
     - Union/intersection type compatibility
     - Solver bailouts on complex types
     - Control Flow Analysis (CFA) definite assignment tracking

2. **Extra Errors (119):** False positives on valid code
   - Most likely causes:
     - Over-strict type narrowing
     - Incorrect generic constraint checking
     - Discriminated union type failures
     - Literal type widening issues
     - Method signature compatibility problems

**Action Items:**

1. **Analyze the 48 Missing TS2322 Errors**
   - Use `wasm/differential-test/find-missing-ts2322.mjs` to extract failing test cases
   - Categorize by root cause:
     - Generic type resolution failures
     - Conditional type evaluation
     - Mapped type handling
     - Union/intersection compatibility
     - CFA-related (properties not known to be assigned)
   - Create prioritized list by frequency

2. **Analyze the 119 Extra TS2322 Errors**
   - Use `wasm/differential-test/find-extra-ts2322.mjs` to extract false positives
   - Categorize by pattern:
     - Type narrowing too aggressive
     - Generic constraints over-checked
     - Literal types not widening when they should
     - Discriminant property checks failing
     - Method signature compatibility
   - Create prioritized list by frequency

3. **Fix Missing Errors (Priority P0)**
   - Focus on highest-frequency categories first
   - Fix solver bailouts on complex types
   - Add CFA tracking where needed:
     - Property assignments in all code paths
     - Variable declarations in closures
     - Array/object destructuring
   - Improve intersection/union type checking
   - Test with extracted failing cases

4. **Fix Extra Errors (Priority P1)**
   - Refine type narrowing logic
   - Fix generic constraint checking
   - Properly handle literal type widening
   - Fix discriminant union type checking
   - Improve method signature compatibility
   - Test with extracted false positive cases

5. **Testing and Validation**
   - Run `wasm/differential-test/find-missing-ts2322.mjs` before/after
   - Run `wasm/differential-test/find-extra-ts2322.mjs` before/after
   - Target: Reduce both to <30 each
   - Verify no regressions in passing tests
   - Run full conformance suite: `./wasm/differential-test/run-conformance.sh --max=2000`

**Success Criteria:**
- Reduce Missing TS2322 from 48 to <15 (70% reduction)
- Reduce Extra TS2322 from 119 to <30 (75% reduction)
- Net improvement: 167 errors → 45 errors (73% reduction)
- Overall conformance improvement: +5-10 percentage points

**Files to Work On:**
- `wasm/src/solver/` - Type resolution and subtyping logic
- `wasm/src/checker/control_flow.rs` - Definite assignment analysis
- `wasm/src/thin_checker.rs` - Type checking and diagnostic emission
- `wasm/src/checker/narrowing.rs` - Type narrowing (if exists)

**Related Work:**
- Builds on Tasks 2-6 (previous TS2322 investigations)
- Coordinates with Worker 9 (parser fixes)
- Coordinates with Worker 2 (module import fixes)

**Target Branch:** rust

**Testing:**
- Run `./wasm/differential-test/run-conformance.sh --max=2000` after changes
- Focus on type-related test categories (classes, es6, async)
- Use TS2322 finder scripts to measure improvement
- Target: <30 missing, <30 extra TS2322 errors

---

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
