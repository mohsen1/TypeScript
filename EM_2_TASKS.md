# EM-2 Task List

## Team: em-team-2
- **Branch:** em-team-2
- **Workers:** worker-5, worker-6, worker-7, worker-8
- **Worktree:** /tmp/orchestrator-workspace/worktrees/em-2
- **Status:** Active (rust branch diverged, has worker commits)

---

## Mission Summary

Lead the **Semantics & Core Type System** squad. Own the most critical issues blocking TypeScript parity:
- Solver strictness (stop hiding errors behind `Any`)
- Global scope poisoning (fix `lib.d.ts` injection)
- Class property initialization (implement missing CFA)
- Support parser noise fixes (worker-5 backup/validation)

Your team has **high priority** because solver/binder issues poison ALL downstream error reporting.

---

## EM Responsibilities

### Ongoing Tasks
- [ ] Sync em-team-2 with rust before any merge operations: `git fetch origin && git merge origin/rust --no-edit`
- [ ] Review each worker's branch after they mark "Ready for Merge: Yes"
- [ ] Run conformance tests after merging each worker branch: `./wasm/differential-test/run-conformance.sh --all`
- [ ] Escalate to Director (merge to rust) only when stable

### Critical: Team Size Management
**Current team size:** 4 workers (at limit)
**Re-size after every merge** to rust. If adding workers, redistribute tasks.

---

## My Workers' Assignments

### worker-5 → Parser Noise (TS1005 & TS1109)
- **Priority:** 🔴 CRITICAL (P1)
- **Status:** ✅ COMPLETED (TS1005/TS1109 suppression merged)
- **Task:** Enhanced error resynchronization in ThinParser
- **Results:** Error suppression implemented, see WORKER_5_TASK_LIST.md
- **Next Action:** Validate conformance improvement; reassign if stable

### worker-6 → Global Scope Fix (TS2304)
- **Priority:** 🔴 CRITICAL (P2)
- **Status:** 🟡 STARTED (completed TS2589, started TS2564)
- **Task:** Fix lib.d.ts injection and global merging
- **Target:** Reduce Extra TS2304 from 343 to <10
- **Problem:** `console`, `Promise`, `Array` undefined → Solver treats as `Any` → suppresses errors
- **Action Items:**
  1. Ensure `lib.d.ts` is correctly merged into root `SymbolTable`
  2. Fix global merging for `interface Window` and similar
  3. Verify loaded BEFORE test files run
- **Files:** `wasm/src/binder/symbol_table.rs`, `wasm/src/binder/mod.rs`
- **Next Action:** Continue on original TS2304 task (not TS2564/TS2589)

### worker-7 → Invert Solver Defaults
- **Priority:** 🟠 STRATEGIC (P3)
- **Status:** ✅ COMPLETED (solver defaults inverted to ERROR)
- **Task:** Return `TypeId::ERROR` instead of `TypeId::ANY` for unresolved references
- **Results:** Successfully inverted defaults, see WORKER_7_TASK_LIST.md
- **Impact:** Expected regression spike (intentional)—exposes hidden errors
- **Next Action:** Validate conformance; document regression; reassign if stable

### worker-8 → Class Property Initialization (TS2564)
- **Priority:** 🟡 TACTICAL (P4)
- **Status:** ✅ COMPLETED (implemented TS2589 recursion guards)
- **Original Task:** Implement `strictPropertyInitialization` check
- **Current Task Completed:** Added recursion guards (TS2589)
- **Target:** Reduce Missing TS2564 from 413 to <20
- **Action Items:**
  1. Implement CFA to verify class properties initialized in constructor
  2. Add check to `wasm/src/checker/thin_checker.rs`
  3. Respect definite assignment assertions (`!`)
- **Files:** `wasm/src/checker/thin_checker.rs`, `wasm/src/checker/class_checker.rs`
- **Next Action:** Return to TS2564 task after TS2589 validation

---

## Merge Status

### Worker Branches → em-team-2
| Worker | Ready? | Last Status | Notes |
|--------|--------|-------------|-------|
| worker-5 | ✅ Merged | a72330bf5 | TS1005/TS1109 suppression |
| worker-6 | ⏳ In Progress | Started | On TS2589/TS2564 (reassign to TS2304) |
| worker-7 | ✅ Merged | aeda8a6d6 | Solver defaults inverted |
| worker-8 | ✅ Merged | dc6d8767d | TS2589 recursion guards |

### em-team-2 → rust
| Status | Notes |
|--------|-------|
| 🟡 Diverged | Has worker commits, awaiting validation |

---

## Workflow

1. **Daily Sync:** `git fetch origin && git merge origin/rust --no-edit`
2. **Worker Review:** Check worker branches for "Ready for Merge: Yes"
3. **Local Merge:** `git merge worker-X` (after review)
4. **Validation:** `./wasm/differential-test/run-conformance.sh --all`
5. **Escalate:** If stable, notify Director for rust merge

---

## Key Metrics

### Baseline (from PROJECT_DIRECTION.md)
- **Exact Match:** 60.8%
- **TS2304 (Extra):** 343 (missing globals)
- **Missing Errors Total:** 2961 (60% gap)
- **TS2564 (Missing):** 413 (class property init)

### Success Targets (for your team)
- **TS2304 (Extra):** <10 (fix lib injection)
- **Solver Defaults:** Return ERROR not ANY (exposes real bugs)
- **TS2564 (Missing):** <20 (implement CFA check)
- **Exact Match:** 80%+ (cumulative across all teams)

---

## Priority Actions

### Immediate (This Cycle)
1. **Reassign worker-6** to original TS2304 task (global scope fix)
2. **Reassign worker-8** to TS2564 (class property initialization)
3. **Validate** worker-5 and worker-7 conformance results
4. **Run full conformance** to measure current em-team-2 state

### Next Cycle (After rust merge)
1. Resize team if needed (max 4 workers)
2. Reassign completed workers to new priority tasks
3. Escalate stable em-team-2 to Director

---

## Status
- **Current Phase:** Worker task reassignment and validation
- **Last Updated:** 2026-01-14
- **Next Action:** Run conformance tests, reassign workers 6 and 8 to original tasks
