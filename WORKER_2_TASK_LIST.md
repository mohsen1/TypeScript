# Worker 2 Task List

Maintained by EM-1

## ⚡ CURRENT STATUS: READY FOR NEW TASK

**Last Updated:** 2026-01-15
**Status:** ✅ All previous tasks completed
**Ready for:** New task assignment from EM-1

**Completed Tasks Summary:**
- ✅ Task 1: Global Scope Symbol Resolution
- ✅ Task 2: Fix TS2792 Module Import Errors
- ✅ Task 3: Verify lib.d.ts Global Scope Injection (TS2304)
- ✅ Task 4: Investigate TS1005/TS1109 Parser Noise (found already completed by worker-5)

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

### Remaining High-Priority Tasks:

1. **Class Property Initialization (TS2564)** - P4 - 413 missing errors
2. **Solver Strictness Improvements** - P3 - 2961 missing errors

Awaiting EM-1 direction on which task to assign next.

---

## Notes
- Work in: /tmp/orchestrator-workspace/worktrees/worker-2
- Push to worker-2 branch when complete
- Do not touch other teams' directories
