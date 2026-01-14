# EM-1 Task List

**Engineering Manager:** EM-1
**Branch:** `em-team-1`
**Base Branch:** `rust`
**Workers:** worker-1, worker-2, worker-3, worker-4

*Last Updated: 2026-01-14*
*Director Review: Outstanding results - build on momentum*

---

## Director's Top Priority for EM-1

**UNBLOCK WORKER-2 IMMEDIATELY** - Uncommitted work is blocking progress. Either:
- Escalate commit request today, OR
- Document specific blocker and request reassignment

Your team has delivered excellent results (99.4% TS2304 reduction, ERROR type enforcement). Build on this momentum.

---

## Current Project Status (Phase 8)

| Metric | Current | Target | Owner |
|--------|---------|--------|-------|
| **TS2304 Extra Errors** | ~50 (post-Worker-1) | <10 | Binder |
| **TS2564 Missing Errors** | 413 | <20 | CFA |
| **Parser False Positives (TS1005/TS1109)** | 701 | <40 | EM-3 |
| **Exact Match** | 30.1% | 80%+ | All |

**Critical Issue:** Parser noise (701 errors) blocks semantic accuracy. EM-3 owns this, but your team's Binder/Solver work is foundational.

---

## Updated Squad Assignments (per Director Review)

### Worker-1: Binder Squad - OUTSTANDING
**Branch:** `worker-1`
**Status:** COMPLETED (99.4% TS2304 reduction)
**Impact:** Fixed lib.d.ts loading - root cause of global symbol poisoning
**Next Steps:**
- Validate lib loading across all test scenarios
- Ensure module augmentation edge cases don't regress
- **Escalate ready commits for merge**
- Focus on integration validation, not new features

---

### Worker-2: Binder Squad - BLOCKED (CRITICAL PATH)
**Branch:** `worker-2`
**Status:** BLOCKED - Uncommitted work
**Priority:** YOUR TOP CONCERN
**Immediate Action Required:**
1. Review current changes in worker-2 worktree
2. Test against conformance suite
3. **Escalate commit request OR document blocker**
4. If blocked >1 day, request reassignment to Stability (Recursion Guards)

**Director Note:** Uncommitted work wastes capacity. Resolve today.

---

### Worker-3: Solver Squad - EXCELLENT
**Branch:** `worker-3`
**Status:** COMPLETED (ERROR type enforcement)
**Impact:** Implemented strictness by returning ERROR instead of ANY
**Next Steps:**
- Verify ERROR type propagation across all solver paths
- Document any remaining `TypeId::ANY` fallbacks
- Run conformance to measure Missing Error reduction
- Focus on validation, not new solver features

---

### Worker-4: CFA Squad - PROGRESSING
**Branch:** `worker-4`
**Status:** IN PROGRESS (DECLARATION flow nodes implemented)
**Priority Goal:** Knock out #1 missing error (TS2564 - 413 occurrences)
**Action Items:**
- Complete `strictPropertyInitialization` check in `thin_checker.rs`
- Verify constructor initialization analysis
- Coordinate with EM-2 Worker-8 to avoid duplication
- Target: Reduce TS2564 missing errors from 413 to <20

**Key Files:**
- `src/checker/thin_checker.rs`
- `src/checker/control_flow.rs`

---

## EM-1 Responsibilities (Updated)

### Daily Operations
1. **UNBLOCK WORKER-2** - This is your #1 priority today
2. **Sync:** `git pull origin rust` and merge into `em-team-1`
3. **Coordinate:** Ensure Worker-4 coordinates CFA work with EM-2 Worker-8
4. **Validate:** Run conformance tests on merged worker branches
5. **Escalate:** Push to Director when ready

### Validation Checklist Before Escalation
- [ ] Worker-2 uncommitted work resolved (committed OR reassigned)
- [ ] All 4 worker branches merge into `em-team-1` without conflicts
- [ ] Conformance tests run: `npm run test:conformance`
- [ ] No regressions in Exact Match score
- [ ] TS2304 extra errors remain <50 (maintain Worker-1 gains)
- [ ] TS2564 missing errors show reduction (Worker-4 progress)

### Merge Workflow
```bash
# Sync with base
git checkout em-team-1
git pull origin rust

# Merge workers in order of completion
git merge worker-1 --no-ff -m "Merge worker-1: Lib loading validation"
git merge worker-3 --no-ff -m "Merge worker-3: Solver strictness validation"
git merge worker-4 --no-ff -m "Merge worker-4: CFA TS2564 implementation"
git merge worker-2 --no-ff -m "Merge worker-2: [resolved blocker]"

# Run validation
npm run test:conformance

# If stable, push to remote
git push origin em-team-1
```

---

## Success Metrics (per PROJECT_DIRECTION.md)
- **TS2304 Extra Errors:** Maintain <50 (Worker-1's fix)
- **TS2564 Missing Errors:** 413 → <20 (Worker-4's target)
- **Overall Exact Match:** Contribute to 80%+ target
- **Team Velocity:** Zero blocked workers

---

## Director Notes

**Your team is the lead for Phase 8 core fixes.** You've delivered excellent results. Keep the momentum by unblocking Worker-2 and coordinating CFA work.

**Anti-Priorities:**
- Do NOT assign parser work (EM-3 owns TS1005/TS1109)
- Do NOT start new features until Worker-2 is resolved
- Do NOT duplicate EM-2's CFA work (coordinate with Worker-8)

**Report blockers immediately** - don't let tasks stall.
