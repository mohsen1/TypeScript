# Worker 2 Task List

Maintained by EM-1

## ⚡ CURRENT STATUS: WORKING ON CLASS PROPERTY TYPE INFERENCE

**Last Updated:** 2026-01-15
**Status:** 🔄 Working on Class Property Type Inference & Validation
**Task:** Shorthand method parameter type inference fix

**Current Task: Class Property Type Inference & Validation**
- **Priority:** 🟡 MEDIUM
- **Problem:** Shorthand methods with tuple parameter types produce TS2304 errors (16 occurrences)
- **Files:** `wasm/src/checker/thin_checker.rs`

**Completed Tasks Summary:**
- ✅ Task 1: Global Scope Symbol Resolution
- ✅ Task 2: Fix TS2792 Module Import Errors
- ✅ Task 3: Verify lib.d.ts Global Scope Injection (TS2304) - Already fixed
- ✅ Task 4: Investigate TS1005/TS1109 Parser Noise - Completed by worker-5
- ✅ Task 5: Investigate Recursion Guards - Already implemented

**Investigation Findings:**
All high-priority tasks from PROJECT_DIRECTION.md have been completed by other workers or were already implemented:
- Parser Noise (TS1005/TS1109) → Completed by worker-5 ✅
- Global Scope Fix (TS2304) → Already fixed in previous commits ✅
- Invert Solver Defaults → Completed by worker-3 ✅
- Class Property Initialization (TS2564) → Completed by worker-3 ✅
- Recursion Guards → Already implemented ✅

**Branch Status:** Clean, synced with rust, ready for new work.

## Completed Tasks

### Task 1: Global Scope Symbol Resolution ✅
- Added comprehensive global symbol resolution tests
- Validated TS2304 fixes from previous work
- Status: Completed and merged to rust

---

### Task 2: Fix TS2792 Module Import Errors ✅

**Status:** @ COMPLETED (2025-01-15)
**Commit:** f5d8d96c05b

**Problem Investigation:**
The "161 missing TS2792 errors" figure was outdated. Current baseline showed only 15 missing errors with 4 TS2307/TS2792 mismatches.

**Root Causes Identified:**

1. **Missing Error Code for Export Declarations:**
   - `export * as ns from './nonexistent'` was not checked
   - Export declarations with module specifiers were not validated
   - Missing `check_export_module_specifier()` function

2. **Wrong Error Code for Relative Imports:**
   - Relative imports (`./module`) emitted TS2307 instead of TS2792
   - Code incorrectly used `MODULE_NOT_FOUND` (2307) for relative paths
   - TypeScript uses TS2792 for ALL unresolved module imports

**Changes Made:**

1. **Added `check_export_module_specifier()` function** (thin_checker.rs:15816-15852)
   - Validates module specifiers in export declarations
   - Checks against resolved modules set
   - Emits TS2792 for unresolved export module specifiers

2. **Updated EXPORT_DECLARATION handling** (thin_checker.rs:14712-14724)
   - Added call to `check_export_module_specifier()`
   - Now checks both export clause AND module specifier

3. **Fixed error code selection** (thin_checker.rs:15808, driver.rs:2472-2477)
   - Changed from: `if relative { MODULE_NOT_FOUND } else { CANNOT_FIND_MODULE }`
   - Changed to: Always use `CANNOT_FIND_MODULE` (TS2792)
   - Matches TypeScript's exact behavior

**Test Results:**

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Missing TS2307/TS2792 mismatches | 4 | 0 | 100% fixed |
| Extra TS2792 errors | 0 | 0 | No regressions |

**Files Modified:**
- `wasm/src/thin_checker.rs`: Added export module specifier check (+49 lines)
- `wasm/src/cli/driver.rs`: Fixed error code to always use TS2792 (-6 lines)

---

### Task 3: Verify lib.d.ts Global Scope Injection (TS2304 Extra Errors)

**Status:** @ INVESTIGATION COMPLETE (2026-01-15)
**Priority:** 🔴 CRITICAL (P2)
**Assigned from:** EM_1_TASKS.md

**Problem Description (from task assignment):**
343 EXTRA TS2304 errors were reportedly emitted because global symbols like `console`, `Promise`, `Array`, etc. were undefined.

**Investigation Results:**

1. **lib.d.ts Loading Mechanism Verified:**
   - `wasm/src/lib_loader.rs` contains `load_default_lib_dts()` and `merge_lib_symbols()`
   - Tests in lib_loader.rs verify global symbols are merged correctly
   - `thin_binder.rs` has `merge_lib_symbols()` and `inject_lib_symbols()` methods

2. **WASM API Verified:**
   - `lib.rs` exposes `addLibFile()` via WASM bindgen
   - `bind_source_file()` calls `bind_source_file_with_libs()` to merge lib symbols
   - `check_source_file()` sets lib_contexts on the checker

3. **Conformance Test Runner Verified:**
   - `conformance-child.mjs` calls `parser.addLibFile(DEFAULT_LIB_NAME, DEFAULT_LIB_SOURCE)`
   - `analyze-extra-ts2304.mjs` loads lib files from TypeScript package

4. **Test Results:**
   - Ran `analyze-extra-ts2304.mjs` on 1000+ test files
   - **Result: 0 extra TS2304 errors found**
   - The issue appears to have been fixed in previous commits

**Git History Analysis:**
Multiple commits show TS2304 was fixed:
- `57245e40429 chore(em-1): document Worker 2 merge - TS2304 fix completed`
- `4ac9c86e467 fix: merge lib symbols BEFORE binding to fix TS2304 "error poisoning"`
- `7b60c2306b8 [wasm] checker: fix lib.d.ts global type resolution`
- `9d8e83e18db fix(wasm): load lib.d.ts files for global symbol resolution`

**Conclusion:**
The TS2304 global scope injection issue (343 extra errors) has been **RESOLVED**. The lib.d.ts loading and symbol merging infrastructure is working correctly. The data in the original task description appears to be outdated.

**Recommendation:**
This task should be marked as complete. No further action needed for TS2304 global scope injection.

---

### Task 4: Investigate Recursion Guards (Stack Overflow Prevention)

**Status:** @ INVESTIGATION COMPLETE (2026-01-15)
**Priority:** 🟢 STABILITY
**Assigned from:** Available high-priority tasks

**Problem Description (from PROJECT_DIRECTION.md):**
2 crashes (stack overflow) in `recursiveTypes` test were blocking all validation work. Task required adding recursion depth counters to prevent unbounded recursion.

**Investigation Results:**

1. **Recursion Guards: ✅ ALREADY FULLY IMPLEMENTED**

   **Location:** `wasm/src/solver/subtype.rs:313-330`

   ```rust
   // Depth Check (stack overflow prevention)
   if self.depth > 100 {
       // Recursion too deep - mark as exceeded and return false to prevent stack overflow
       self.depth_exceeded = true;
       return SubtypeResult::False;
   }

   // Cycle detection (coinduction)
   let pair = (source, target);
   if self.in_progress.contains(&pair) {
       // We're in a cycle - return provisional true
       return SubtypeResult::Provisional;
   }
   ```

2. **TS2589 Error Emission: ✅ ALREADY IMPLEMENTED**

   **Location:** `wasm/src/thin_checker.rs:11513-11517`

   ```rust
   // Emit TS2589 if recursion depth was exceeded
   if depth_exceeded.1 {
       self.error_at_current_node(
           diagnostic_messages::TYPE_INSTANTIATION_EXCESSIVELY_DEEP,
           diagnostic_codes::TYPE_INSTANTIATION_EXCESSIVELY_DEEP,
       );
   }
   ```

3. **Verification:**
   - Depth counter with MAX_DEPTH = 100 ✅
   - Cycle detection using coinductive semantics (GFP) ✅
   - TS2589 error emission when depth exceeded ✅
   - **Zero crashes** in all test scenarios ✅

4. **Previous Investigation:**
   - Worker-3 investigated this task (commit `120fe36f5dc`)
   - Created `RECURSION_GUARDS_FINDINGS.md` documenting implementation
   - Worker-4 was assigned but task was reassigned when found complete
   - Commit `2269cd7d66b` marked task as complete

**Conclusion:**
The Recursion Guards task has been **FULLY IMPLEMENTED** and is working correctly. No crashes were found in testing. The implementation includes:
- Recursion depth limiting (MAX_DEPTH = 100)
- Cycle detection for legitimate recursive types
- Proper TS2589 error emission

**Recommendation:**
No action needed. Task is complete.

**Reference:** See `RECURSION_GUARDS_FINDINGS.md` for full investigation details.

---

## Next Task: TBD

**Status:** AWAITING ASSIGNMENT

### Investigation Notes:

**Parser Noise (TS1005 & TS1109) Task Status:**

**Status:** ✅ ALREADY COMPLETED by Worker-5

Investigation (2026-01-15) revealed that the Parser Noise task has been **fully completed** by Worker-5 (EM-2 team). The improvements have been merged to the rust branch.

**Worker-5 Results:**
- **Goal:** Reduce 701 combined extra errors to <40 (94% reduction)
- **Achieved:** ~29 errors (96% reduction) ✅ GOAL EXCEEDED
- **TS1005:** 97% reduction (439 → ~13 errors)
- **TS1109:** 94% reduction (262 → ~16 errors)

**Merged Commits:**
- `b690646d692 Merge worker-5: Parser noise reduction - Goal achieved`
- `76aaec21bdc feat(parser): advanced error suppression mechanisms`
- `8db5e2f82ff docs: mark Parser Noise Fix task as complete`

**Improvements Implemented:**
1. ASI (Automatic Semicolon Insertion) for restricted productions
2. Error budget system (reduced from 10 to 2/3 errors per statement)
3. Expression boundary detection (`is_at_expression_end()`)
4. Object/array literal recovery
5. Enhanced statement-level error recovery

**Documentation:** See `WORKER_5_PARSER_IMPROVEMENTS.md` for full details.

---

### Available Tasks Identified:

Based on investigation of other workers' task lists, the following tasks are **AVAILABLE FOR REASSIGNMENT**:

#### 🔴 HIGH PRIORITY: Class Property Type Inference & Validation
**Status:** ⚠️ ASSIGNED TO WORKER-12 BUT NOT STARTED
**Worker-12 Status:** Synchronized but appears unavailable (no implementation commits)
**Priority:** 🟡 MEDIUM
**Problem:**
- Shorthand methods with tuple parameter types produce TS2304 errors (16 occurrences)
- Type checker fails to infer types for shorthand method parameters
- Object literal property type inference gaps
**Success Criteria:**
- Fix shorthand method type inference (16 TS2304 errors)
- Improve object literal type inference
- Reduce class-related errors by 50%
**Files:** `wasm/src/checker/thin_checker.rs`
**Reference:** `/tmp/orchestrator-workspace/worktrees/em-3/WORKER_12_TASK_LIST.md`

#### 🔴 CRITICAL: Fix TS2322 Type Accuracy Balance
**Status:** 🔄 ASSIGNED TO WORKER-11 (IN PROGRESS)
**Priority:** 🔴 CRITICAL (167 total errors: 48 missing + 119 extra)
**Current Baseline:**
- Missing TS2322: 48 occurrences
- Extra TS2322: 119 occurrences
**Note:** This task is active with worker-11, may need coordination
**Reference:** `/tmp/orchestrator-workspace/worktrees/em-3/WORKER_11_TASK_LIST.md`

#### 🟡 MEDIUM: Module Resolution Validation (Task 3)
**Status:** ⚠️ ASSIGNED TO WORKER-10 BUT NOT STARTED
**Worker-10 Status:** Appears unavailable
**Priority:** 🔴 HIGH
**Context:** Worker-10 completed Module Resolution (Tasks 1-2) but Task 3 validation not started
**Reference:** `/tmp/orchestrator-workspace/worktrees/em-3/WORKER_10_TASK_LIST.md`

---

Awaiting EM-1 direction on which task to assign next.

**Recommendation:** Consider assigning **Class Property Type Inference & Validation** as it's:
1. Clearly defined scope
2. Medium priority (good balance of impact/complexity)
3. Worker-12 appears unavailable
4. Has clear success criteria and file locations

---

## Notes
- Work in: /tmp/orchestrator-workspace/worktrees/worker-2
- Push to worker-2 branch when complete
- Do not touch other teams' directories

---

## Worker-2 Merge Summary (January 15, 2026)

### EM-1 Merge Details:
**Merge Commit:** `4ed30d05ac8`
**Branch:** worker-2 → em-team-1
**Status:** ✅ Successfully merged (investigation documentation)

### Commits Merged:
1. **Comprehensive Available Tasks Investigation Report** (434519fc27a)
   - Documented all available tasks for potential reassignment
   - Identified Class Property Type Inference & Validation as available
   - Noted TS2322 Type Accuracy Balance (worker-11 in progress)
   - Documented Module Resolution Validation (worker-10 not started)

2. **Recursion Guards Investigation Findings** (d4264e9e192)
   - Confirmed task already implemented with depth counter (MAX_DEPTH = 100)
   - Verified cycle detection using coinductive semantics (GFP)
   - Confirmed TS2589 error emission when depth exceeded
   - Zero crashes found in all test scenarios

3. **Task Status Update** (ed4c331e75d)
   - Updated worker-2 status: ready for new assignment
   - All assigned tasks and investigations completed
   - Awaiting EM-1 direction on next task

4. **TS1005/TS1109 Parser Noise Investigation** (82095fce371)
   - Confirmed task completed by worker-5
   - 96% reduction achieved (701 → ~29 errors)
   - Goal exceeded (target was <40 errors)

5. **TS2304 Global Scope Fix Verification** (ba6690b32b7)
   - Verified lib.d.ts loading mechanism working correctly
   - Confirmed 0 extra TS2304 errors in testing
   - Issue fixed in previous commits

### Files Modified:
- `WORKER_2_TASK_LIST.md` - Comprehensive investigation documentation

### Summary:
Worker-2 completed all investigation tasks and is ready for new work. All high-priority tasks from PROJECT_DIRECTION.md have been completed by other workers or were already implemented.
