# EM-1 Task List

## Team: em-team-1
- **Branch:** em-team-1
- **Workers:** worker-1, worker-2, worker-3, worker-4
- **Worktree:** /tmp/orchestrator-workspace/worktrees/em-1

## Mission Summary
Keep em-team-1 in sync with rust. Own task assignment for workers 1-4. Merge worker branches locally, run validations, and escalate to Director only when stable.

## EM Responsibilities

### Ongoing Tasks
- [ ] Sync em-team-1 with rust before any merge operations
- [ ] Review each worker's branch after they mark "Ready for Merge: Yes"
- [ ] Run conformance tests after merging each worker branch
- [ ] Escalate to Director (merge to rust) only when stable

### My Workers' Assignments

#### worker-1 → Parser Noise (TS1005 & TS1109)
- **Priority:** 🔴 CRITICAL
- **Task:** Fix error resynchronization in ThinParser
- **Target:** Reduce TS1005/TS1109 from ~700 to <40
- **Task List:** `WORKER_1_TASK_LIST.md`

#### worker-2 → Global Scope Fix (TS2304)
- **Priority:** 🔴 CRITICAL
- **Task:** Fix lib.d.ts injection and global merging
- **Target:** Reduce Extra TS2304 from 343 to <10
- **Task List:** `WORKER_2_TASK_LIST.md`

#### worker-3 → Class Property Initialization (TS2564)
- **Priority:** 🟡 TACTICAL
- **Task:** Implement strictPropertyInitialization check
- **Target:** Reduce Missing TS2564 from 413 to <20
- **Task List:** `WORKER_3_TASK_LIST.md`

#### worker-4 → Recursion Guards
- **Priority:** 🟢 STABILITY
- **Task:** Add recursion depth counters to prevent stack overflow
- **Target:** Zero crashes on recursiveTypes test
- **Task List:** `WORKER_4_TASK_LIST.md`

## Merge Status

### Worker Branches
| Worker | Ready? | Last Sync | Notes |
|--------|--------|-----------|-------|
| worker-1 | ⏳ Pending | - | Awaiting assignment |
| worker-2 | ⏳ Pending | - | Awaiting assignment |
| worker-3 | ⏳ Pending | - | Awaiting assignment |
| worker-4 | ⏳ Pending | - | Awaiting assignment |

### em-team-1 → rust
| Status | Notes |
|--------|-------|
| ⏳ Pending | Awaiting stable worker branches |

## Workflow

1. **Daily Sync:** `git fetch origin && git merge origin/rust --no-edit`
2. **Worker Review:** Check worker branches for "Ready for Merge: Yes"
3. **Local Merge:** `git merge worker-X` (after review)
4. **Validation:** `./wasm/differential-test/run-conformance.sh --all`
5. **Escalate:** If stable, notify Director for rust merge

## Key Metrics

### Baseline (from PROJECT_DIRECTION.md)
- **Exact Match:** 30.1%
- **TS1005/TS1109 (Parser):** ~700 extra errors
- **TS2304 (Binder):** 343 extra, 116 missing
- **TS2564 (CFA):** 413 missing errors
- **Crashes:** 2 (stack overflow)

### Success Targets
- **TS1005/TS1109:** <40
- **TS2304 (Extra):** <10
- **TS2564 (Missing):** <20
- **Exact Match:** 80%+
- **Crashes:** 0

## Status
- **Current Phase:** Task assignment and initialization
- **Last Updated:** 2026-01-14
- **Next Action:** Create WORKER task lists and notify workers
