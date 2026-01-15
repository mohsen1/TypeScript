# Worker 2 Task List

Maintained by EM-1 (reassigned from EM-2)

## Active Task

### 🎯 NEW ASSIGNMENT: Parser Noise Cleanup Support 🟢
**Priority:** LOW-SUPPORT (Worker-1 Backup)
**Assigned:** 2026-01-15
**Owner:** worker-2
**Branch:** worker-2
**Status:** ⏸️ ON HOLD - Wait for Worker-1 Progress

### Task Description
Support Worker-1's Parser Noise (TS1005/TS1109) cleanup effort by tackling remaining edge cases after initial implementation. This is a support role - WAIT for worker-1 to make initial progress before starting.

### Problem Analysis
From PROJECT_DIRECTION.md:
- **Parser Noise:** ~700 extra errors (TS1005: 439, TS1109: 262)
- **Root Cause:** ThinParser bailing out on valid syntax, ASI issues
- **Primary Owner:** Worker-1

### Action Items (ON HOLD - Do NOT start yet)

#### Phase 1: Wait for Worker-1 Progress
- [ ] Monitor worker-1's progress reports
- [ ] Review worker-1's initial fixes when available
- [ ] Identify remaining edge cases from worker-1's results

#### Phase 2: Tackle Remaining Edge Cases
- [ ] Investigate ASI (Automatic Semicolon Insertion) edge cases
- [ ] Fix parser error resynchronization in specific contexts:
  - Object literals
  - Array literals
  - Function parameters
  - Type annotations
- [ ] Run conformance tests to verify improvements

#### Phase 3: Validation
- [ ] Run `./wasm/test.sh` (Docker-only!)
- [ ] Run conformance tests: `./wasm/differential-test/run-conformance.sh --all`
- [ ] Verify TS1005/TS1109 reduced to <40 combined
- [ ] Check for regressions

### Success Metrics
- **TS1005/TS1109 Combined:** Reduce from ~700 to <40
- **No Regressions:** Don't break existing working tests
- **Support Worker-1:** Complement their work, not duplicate

### Deliverables
1. Parser edge case fixes
2. Conformance test report showing improvement
3. Updated task list with "Complete" status

### Status
- **Previous Tasks:** ✅ Module Resolution (TS2792), ✅ Global Scope Resolution
- **Current Task:** 🟢 Parser Noise Support (ON HOLD)
- **Ready to Start:** ⏸️ NO - Wait for Worker-1 progress
- **Last Updated:** 2026-01-15

---

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
| Missing TS2792 (3000 samples) | 15 | 10 | 33% reduction |
| TS2307/TS2792 mismatches | 4 | 0 | 100% fixed |
| Extra TS2792 errors | 0 | 0 | No regressions |

**Sample Test Cases Fixed:**
- `export * as ns from './nonexistent'` - Now emits TS2792 ✅
- `import { x } from './module'` - Now emits TS2792 (not TS2307) ✅
- All relative import errors now use TS2792 ✅

**Remaining Issues (10 missing):**
- Module resolution edge cases (ES5 target, package imports)
- File extension handling (`./foo.ts`, `./example.json`)
- Package subpath resolution (`lodash-ts/add.ts`)
- #imports syntax

These are module resolution logic issues, not TS2792 emission issues.

**Files Modified:**
- `wasm/src/thin_checker.rs`: Added export module specifier check (+49 lines)
- `wasm/src/cli/driver.rs`: Fixed error code to always use TS2792 (-6 lines)

**Testing:**
- Manual verification with `export * as from './nonexistent'` ✅
- Conformance tests with find-ts2792.mjs (100-3000 samples) ✅
- No extra TS2792 errors introduced ✅

**Success Criteria Met:**
- ✅ Reduced TS2307/TS2792 mismatches to 0
- ✅ Reduced missing TS2792 from 15 to 10 (33% improvement)
- ✅ No false positives introduced
- ✅ Export declarations now properly checked

**Next Steps:**
- Remaining 10 missing errors require module resolution enhancements
- Consider adding more sophisticated module resolution logic
- Could add support for:
  - ES5 target module kind handling
  - File extension resolution (.ts, .json, etc.)
  - Package subpath resolution
  - #imports syntax

---

## Notes
- Work in: /tmp/orchestrator-workspace/worktrees/worker-2
- Push to worker-2 branch when complete
- Do not touch other teams' directories
