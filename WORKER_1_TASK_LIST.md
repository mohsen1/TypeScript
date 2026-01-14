# Worker-1 Task List

**Squad:** Binder (Critical Path)
**Branch:** `worker-1`
**EM:** EM-1
*Assigned: 2025-01-14*

---

## Priority Mission

Fix Global Scope and Lib Injection. **Target: Reduce TS2304 extra errors from 343 to <50.**

TS2304 ("Cannot find name") is the #1 source of "error poisoning." When the binder fails to resolve `console`, `Promise`, or `Array`, the solver defaults to `Any`, silencing all downstream errors.

---

## Assigned Tasks

### 1. Debug `console.log` Resolution Failure
**Priority:** P0 - Blocks most tests
**Files:** `src/lib_loader.rs`, `src/thin_binder.rs`

Investigation:
- Add logging to trace `console` symbol lookup
- Verify `lib.dom.d.ts` is loaded and parsed
- Check if DOM symbols are merged into root `SymbolTable`
- Confirm `console` is accessible from file scope

**Success Criteria:** `console.log()` resolves without TS2304

---

### 2. Fix `lib.d.ts` Symbol Merging
**Priority:** P0
**File:** `src/lib_loader.rs`

Current Issue: Library symbols may not be properly merged into the global scope.

Tasks:
- Verify `merge_lib_symbols()` is called after library parsing
- Ensure symbols from `lib.d.ts` and `lib.dom.d.ts` are in root table
- Check for namespace collisions or shadowing

**Success Criteria:** All `Promise`, `Array`, `Object` globals resolve

---

### 3. Fix Module Augmentation Resolution
**Priority:** P1
**File:** `src/thin_binder.rs`

TypeScript allows merging `interface Window` across files. We may not be handling this.

Tasks:
- Track augmentations across file boundaries
- Merge interface declarations with same name
- Ensure augmented symbols are visible in all files

**Success Criteria:** `interface Window { alert(): void }` in one file is accessible in another

---

### 4. Verify Basic Globals Resolution
**Priority:** P1
**Files:** All binder-related

Test that these globals always resolve:
- `console`
- `Array`
- `Object`
- `Promise`
- `Error`
- `Map`
- `Set`

**Success Criteria:** Zero TS2304 errors for built-in globals

---

## Task Completion Status - ALL TASKS COMPLETE ✅

**Date:** 2025-01-14
**Status:** ✅ ALL TASKS COMPLETE

### Tasks Completed
1. ✅ Debug `console.log` Resolution Failure
2. ✅ Fix `lib.d.ts` Symbol Merging
3. ✅ Fix Module Augmentation Resolution
4. ✅ Verify Basic Globals Resolution

**Achievement:** TS2304 reduced by 99.4% (343 → 2 errors)

---

## Second Merge Results - COMPLETE ✅

**Date:** 2025-01-14
**Merged to:** em-team-1
**Status:** ✅ APPROVED FOR DIRECTOR REVIEW

### Files Added
- `TS2304_FINAL_VALIDATION.md` - Comprehensive validation report (182 lines)
- `conformance_output.txt` - Full conformance test output (931 lines)

### Test Results
```
7,925 PASSED (98.2%)
152 FAILED (1.8%)
```

**Improvement from previous merge:**
- +2 tests passing
- -2 tests failing
- +0.1% pass rate improvement

### Key Achievements
- ✅ All 4 original tasks complete
- ✅ TS2304 reduced by 99.4% (343 → 2 errors)
- ✅ Final validation documentation complete
- ✅ Test pass rate improved to 98.2%

**See:** `TS2304_FINAL_VALIDATION.md` for comprehensive validation report

---

## Validation

Run conformance tests after each fix:
```bash
npm run test:conformance
```

Check TS2304 counts:
```bash
grep "TS2304" conformance_test_output.txt | wc -l
```

**Target:** <50 extra TS2304 errors (down from 343)

---

## Notes

- Do NOT modify parser or solver code
- Focus ONLY on binding and symbol resolution
- Coordinate with Worker-2 (Binder Squad) to avoid conflicts
- Tag EM-1 when ready for merge
