# Worker-1 Task List

**Squad:** Binder (Critical Path)
**Branch:** `worker-1`
**EM:** EM-1
*Assigned: 2025-01-14*

---

## Priority Mission - COMPLETED ✅

Fix Global Scope and Lib Injection. **Target: Reduce TS2304 extra errors from 343 to <50.**
**ACHIEVED: Reduced to 2 errors (99.4% reduction)**

---

## Phase 2 Tasks

### 5. Accept and Verify New Test Baselines
**Priority:** P0 - Blocking test suite
**Files:** `tests/baselines/local/`

The conformance tests created new baselines after our TS2304 fixes. These need to be reviewed and committed.

Tasks:
- Review the ~15 new baseline files created
- Verify the baselines reflect the correct (improved) behavior
- Commit the baselines to complete the fix

**Success Criteria:** All baselines committed, tests pass without "New baseline created" errors

---

### 6. Investigate Remaining TS2304 Sources
**Priority:** P1
**Files:** `wasm/src/`

We have 2 remaining TS2304 errors (intentional). Investigate if there are other TS2304 sources we haven't addressed.

Tasks:
- Search for any remaining TS2304 patterns in test output
- Check if module-specific symbols need special handling
- Verify `declare global` augmentations work correctly

**Success Criteria:** Document any remaining TS2304 sources and their mitigation

---

### 7. Optimize Lib Symbol Loading Performance
**Priority:** P2
**Files:** `wasm/src/parallel.rs`, `wasm/src/lib_loader.rs`

Current implementation loads lib.d.ts for each file binding. This may be inefficient.

Tasks:
- Profile lib loading performance
- Consider caching lib binders across files
- Benchmark with and without caching

**Success Criteria:** Lib loading is not a performance bottleneck

---

### 8. Verify Cross-File Symbol Merging Edge Cases
**Priority:** P2
**Files:** `wasm/src/parallel.rs`

The `merge_bind_results` function handles cross-file interface merging. Verify edge cases work correctly.

Tasks:
- Test multiple interface augmentations across 3+ files
- Verify namespace merging works
- Test interface + class merging
- Add test cases for edge cases

**Success Criteria:** All cross-file merging scenarios work correctly

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
npm test
```

Check for remaining TS2304 counts:
```bash
grep "TS2304" tests/baselines/local/*.errors.txt | grep -v "^tests/baselines/local/binder_integration" | wc -l
```

---

## Notes

- Do NOT modify parser or solver code (those are other squads)
- Focus ONLY on binding and symbol resolution
- Coordinate with Worker-2 (Binder Squad) to avoid conflicts
- Tag EM-1 when ready for merge
