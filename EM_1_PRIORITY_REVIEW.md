# EM-1 Team Priority Review & Task Assignments
**Date:** 2026-01-15
**Reviewer:** EM-1

## Current Team State

### Workers With Active Tasks:
- **Worker-1:** Parser Noise (TS1005/TS1109) - IN PROGRESS (assigned but not ready for merge)
- **Worker-3:** Global Scope Fix (TS2304) - IN PROGRESS (assigned 2026-01-15)
- **Worker-12:** Class Property Type Inference - IN PROGRESS (assigned but no commits yet)

### Workers Available for Assignment:
- **Worker-4:** ✅ Flow Recording complete, ✅ Application Expansion complete - **READY**
- **Worker-11:** TS2322 Type Accuracy complete - **READY**
- **Worker-2:** Module Resolution complete (EM-3 team) - **READY**

---

## Priority Analysis (from PROJECT_DIRECTION.md)

### CRITICAL PRIORITIES:
1. **Parser Noise (TS1005/TS1109)** - ~700 extra errors
2. **Global Scope (TS2304)** - 343 extra, 116 missing (Error Poisoning)
3. **Solver Defaults** - 2961 missing errors (Invert ANY to UNKNOWN)

### HIGH IMPACT:
4. **Class Property Initialization (TS2564)** - #1 missing error (413 occurrences)

### COMPLETED:
- ✅ Recursion Guards (verified by worker-3, zero crashes)
- ✅ Module Resolution (worker-2, worker-10)
- ✅ Flow Recording (worker-4)

---

## Task Assignments

### Assignment 1: Fix Class Property Initialization (TS2564)
**Assigned to:** Worker-4
**Priority:** 🟠 HIGH (Strategic - #1 missing error category)
**Reasoning:** Worker-4 just completed two major tasks (Flow Recording, Application Expansion) with excellent results. TS2564 is the single biggest missing error category (413 occurrences).

**Task:**
- Implement/fix `strictPropertyInitialization` check in `wasm/src/checker/thin_checker.rs`
- Target: Reduce 413 missing errors to <20
- High ROI task that directly impacts conformance metrics

### Assignment 2: Investigate Missing TS2322 Patterns
**Assigned to:** Worker-11
**Priority:** 🟡 MEDIUM-HIGH (Strategic Analysis)
**Reasoning:** Worker-11 just completed TS2322 analysis and has deep domain knowledge. Before implementing fixes, we need more investigation into the 105 missing TS2322 errors.

**Task:**
- Investigate why 105 TS2322 errors are missing
- Focus on abstract constructor assignability (primary issue from worker-11's analysis)
- Document patterns and prepare fix strategy
- DO NOT implement yet - investigation phase only

### Assignment 3: Remaining Parser Noise Cleanup
**Assigned to:** Worker-2
**Priority:** 🟢 SUPPORT (Worker-1 needs backup)
**Reasoning:** Worker-1 is working on Parser Noise but it's a large task (701 errors). Worker-2 has parser experience and can help tackle remaining issues.

**Task:**
- Investigate remaining TS1005/TS1109 edge cases after worker-1's initial work
- Focus on ASI (Automatic Semicolon Insertion) edge cases
- Wait for worker-1 to complete initial pass before starting

---

## Workers on Hold

### Worker-12: Class Property Type Inference
**Status:** ⏸️ PAUSED
**Reason:** TS2564 is higher priority and in the same domain (class properties). After worker-4 completes TS2564, worker-12 can resume with better context.

### Worker-1: Parser Noise
**Status:** 🔄 CONTINUE (make progress report in 24 hours)

### Worker-3: Global Scope Fix
**Status:** 🔄 CRITICAL PATH - Continue investigation

---

## Success Metrics for Next Review

### Target (from PROJECT_DIRECTION.md):
- **TS1005/TS1109 (Parser):** Reduce from ~700 to <40
- **TS2304 (Binder):** Reduce Extra errors from 343 to <10
- **TS2564 (CFA):** Reduce Missing errors from 413 to <20
- **Exact Match:** Increase from 60.8% to **80%+**

### Worker-4 (TS2564):
- [ ] Implement strictPropertyInitialization check
- [ ] Run conformance tests
- [ ] Reduce missing TS2564 from 413 to <20
- [ ] Document any regressions

### Worker-11 (TS2322 Investigation):
- [ ] Categorize all 105 missing TS2322 patterns
- [ ] Identify abstract constructor assignability fix
- [ ] Document implementation strategy
- [ ] Prepare detailed plan for EM review

### Worker-2 (Parser Support):
- [ ] Wait for worker-1 progress report
- [ ] Identify remaining ASI edge cases
- [ ] Prepare to tackle parser noise leftovers

---

## Risk Assessment

### High Risk:
- **Worker-1 Parser Task:** Large scope (701 errors), may need additional support
- **Worker-3 Global Scope:** Critical path task, blocks other improvements

### Mitigation:
- Worker-2 on standby to support worker-1 if needed
- Daily check-ins on worker-3 progress
- EM-1 available to unblock critical issues

---

## Next Actions

1. ✅ Update WORKER_4_TASK_LIST.md with TS2564 assignment
2. ✅ Update WORKER_11_TASK_LIST.md with TS2322 investigation assignment
3. ✅ Update WORKER_2_TASK_LIST.md with parser support assignment
4. ⏸️ Update WORKER_12_TASK_LIST.md with PAUSED status
5. 📧 Notify workers of new assignments
6. 📅 Schedule 24-hour progress review

---

**Prepared by:** EM-1
**Date:** 2026-01-15
**Status:** Ready for director review
