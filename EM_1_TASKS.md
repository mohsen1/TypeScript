# EM_1 Tasks - Hybrid Squad (Parser + Binder)

**Branch:** `em-team-1`
**Priority:** 🔴 HIGHEST
**Assigned Workers:** workers 1-4
**Last Updated:** 2026-01-14 (Director merge)

---

## Director's Note

**This is a HYBRID squad due to worker progress made before formal squad structure.**

- Workers 1-3: Parser work (TS1005/TS1109) - will transfer to EM_2 after validation
- Worker 4: Binder work (TS2304) - core EM_1 responsibility

**After current validation cycle, workers 1-3 will formally transfer to EM_2.**

---

## Overall Mission: Phase 8 - Stop the "Any" Poisoning

### Current Status (from PROJECT_DIRECTION.md)
| Metric | Current | Target |
|--------|---------|--------|
| **Exact Match** | 30.1% | **40%** |
| **Missing Errors** | 60.0% | **<50%** |
| **Parser false positives** | 701 | **<100** |
| **TS2304 extra errors** | 343 | **<50** |

### Squad Allocation (4 workers)
| Squad | Workers | Focus Area | Status |
|-------|---------|------------|--------|
| **Parser (Syntax)** | worker-1, worker-2, worker-3 | TS1005/TS1109 false positives | 🟠 Active |
| **Binder (CRITICAL)** | worker-4 | TS2304 error poisoning | 🔴 Critical Path |

---

## EM-1 Responsibilities

### 1. Branch Hygiene
- [x] Sync em-team-1 with rust (merged via origin/rust)
- [ ] Merge worker branches locally only after validation
- [ ] Run full conformance suite before any merge to rust
- [ ] Escalate to Director only when metrics show stable improvement

### 2. Task Assignment Strategy

#### Priority 1: Fix TS2304 (Binder) - Error Poisoning Root Cause
- Worker 4 is on the critical path
- TS2304 causes `Any` fallback which silences all downstream errors
- Must fix before solver work can be accurately validated

#### Priority 2: Fix Parser False Positives (TS1005/TS1109)
- Workers 1-3 working in parallel on different patterns
- 701 parser errors inflate "Extra Errors" by 14%
- Cascading errors compound the problem

### 3. Validation Protocol
Before merging any worker branch:
1. Worker must run conformance tests and report metrics
2. Verify no regressions in other error codes
3. Ensure build passes (`cargo build --release`)
4. Check that the specific metric improved (e.g., TS1005 count decreased)

---

## Current Worker Assignments

### Worker 1 (Parser - TS1005)
**Status:** ✅ MERGED - Patterns 1-5 reduce TS1005 by 29% (439→312)
**Results:** Exact Match +1.1%, No regressions, Build passes
**Next:** Coordinate with Worker 3 on comma inference recovery

### Worker 2 (Parser - TS1109)
**Status:** ✅ VALIDATED - Definite assignment fix reduces TS1109 by 24% (262→198)
**Results:** Exact Match +0.4%, No regressions, Build passes
**Decision:** MERGE APPROVED - Ready to merge worker-2 into em-team-1
**Next:** After merge, prioritize new.target context validation (~52 cases)

### Worker 3 (Parser - Cascading Errors)
**Status:** ✅ VALIDATED - Cascading error fix reduces parser FP by 21% (701→551)
**Results:** Biggest single-worker impact! Exact Match +1.8%, No regressions, Build passes
**Synergy:** Amplifies Worker 1 & 2 results - combined: 701→389 (-44%, 312 errors)
**Decision:** MERGE APPROVED - Ready to merge worker-3 into em-team-1
**Lesson:** Should have been FIRST - cascading errors masked individual fix impact

### Worker 4 (Binder - CRITICAL)
**Status:** Fixed lib.d.ts symbol merging, working on ambient modules
**Next:** Complete ambient module fix, then module augmentation
**Blocker:** None - this is the critical path

---

## Next Actions for EM-1

1. **IMMEDIATE:** Have all workers run conformance tests to establish baselines
2. **TODAY:** Review Worker 4's ambient module fix (critical path)
3. **THIS WEEK:** Merge validated fixes in order: Worker 4 → Worker 3 → Worker 1 → Worker 2
4. **CONTINUOUS:** Monitor metrics - target is <100 parser false positives and <50 TS2304 errors
5. **AFTER VALIDATION:** Transfer workers 1-3 to EM_2 (Parser Squad)

---

## Success Criteria

- Exact Match increases from 30.1% to 40%
- Missing Errors decreases from 60% to <50%
- Parser false positives (TS1005 + TS1109) reduced from 701 to <100
- TS2304 extra errors reduced from 343 to <50
- All worker branches validated and merged locally
- Ready to escalate to Director with stable metrics improvement
- Workers 1-3 transferred to EM_2 after validation cycle

---

## Key Files for Reference

| Focus Area | File |
|------------|------|
| **Parser** | `src/compiler/parser.ts` |
| **Binder** | `src/lib_loader.rs`, `src/thin_binder.rs`, `src/symbol_table.rs` |
