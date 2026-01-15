# EM-3 Task List

## Team: em-team-3
- **Branch:** em-team-3
- **Workers:** worker-9, worker-10, worker-11, worker-12
- **Worktree:** /tmp/orchestrator-workspace/worktrees/em-3

## Mission Summary
Keep em-team-3 in sync with rust. Own task assignment for workers 9-12. Merge worker branches locally, run validations, and escalate to Director only when stable.

## EM Responsibilities

### Ongoing Tasks
- [ ] Sync em-team-3 with rust before any merge operations
- [ ] Review each worker's branch after they mark "Ready for Merge: Yes"
- [ ] Run conformance tests after merging each worker branch
- [ ] Escalate to Director (merge to rust) only when stable

### My Workers' Assignments

#### worker-9 -> Parser Error Recovery (TS1005 & TS1109)
- **Priority:** @ CRITICAL
- **Task:** Implement robust error resynchronization in ThinParser
- **Target:** Reduce TS1005/TS1109 from ~700 to <40
- **Task List:** `/tmp/orchestrator-workspace/worktrees/worker-9/WORKER_9_TASK_LIST.md`
- **Branch:** worker-9

#### worker-10 -> Global Scope & Lib Injection (TS2304)
- **Priority:** @ CRITICAL
- **Task:** Fix lib.d.ts injection and global symbol merging
- **Target:** Reduce Extra TS2304 from 343 to <10
- **Task List:** `/tmp/orchestrator-workspace/worktrees/worker-10/WORKER_10_TASK_LIST.md`
- **Branch:** worker-10

#### worker-11 -> Solver Strictness (Missing Errors)
- **Priority:** @ STRATEGIC
- **Task:** Invert solver defaults from ANY to UNKNOWN/ERROR
- **Target:** Reduce missing errors from 2961 to <500
- **Task List:** `/tmp/orchestrator-workspace/worktrees/worker-11/WORKER_11_TASK_LIST.md`
- **Branch:** worker-11

#### worker-12 -> Class Property Initialization (TS2564)
- **Priority:** @ TACTICAL
- **Task:** Implement strictPropertyInitialization check
- **Target:** Reduce Missing TS2564 from 413 to <20
- **Task List:** `/tmp/orchestrator-workspace/worktrees/worker-12/WORKER_12_TASK_LIST.md`
- **Branch:** worker-12

## Merge Status

### Worker Branches
| Worker | Ready? | Last Sync | Notes |
|--------|--------|-----------|-------|
| worker-9 | ✅ Merged | 2025-01-14 | ASI fixes and parser improvements |
| worker-10 | ✅ Merged | - | (No new commits - fully synced) |
| worker-11 | ✅ Merged | 2025-01-14 | ERROR type diagnostic analysis (Tasks 2-3) |
| worker-12 | ✅ Merged | 2025-01-14 | WASM compilation fix + TS2564 verification |

### em-team-3 -> rust
| Status | Notes |
|--------|-------|
| @ Ready for Review | All workers merged, validated, and ready for rust merge |

### Validation Results (2025-01-14)
- **WASM Build:** ✅ Success (61 warnings, 0 errors)
- **Conformance Tests:**
  - Tests Run: 487
  - Exact Match: 158 (32.4%)
  - Same Error Count: 184 (37.8%)
  - **Total Parity: 70.2%** (exact + same count)
  - **WASM Crashed: 0** (down from 2) ✅
- **Key Improvements:**
  - TS2564 no longer in missing errors (was 413 missing)
  - TS2322 reduced to 13 missing (was 310+)
  - WASM compilation fixed (syntax error in thin_parser.rs)

## Workflow

1. **Daily Sync:** `git fetch origin && git merge origin/rust --no-edit`
2. **Worker Review:** Check worker branches for "Ready for Merge: Yes"
3. **Local Merge:** `git merge worker-X` (after review)
4. **Validation:** `./wasm/differential-test/run-conformance.sh --all`
5. **Escalate:** If stable, notify Director for rust merge

## Key Metrics

### Baseline (from PROJECT_DIRECTION.md)
- **Exact Match:** 60.8%
- **TS1005/TS1109 (Parser):** ~700 extra errors
- **TS2304 (Binder):** 343 extra, 116 missing
- **Missing Errors:** 2961 total (60%)
- **TS2564 (CFA):** 413 missing errors
- **Crashes:** 2 (stack overflow)

### Success Targets
- **TS1005/TS1109:** <40
- **TS2304 (Extra):** <10
- **Missing Errors Total:** <500
- **TS2564 (Missing):** <20
- **Exact Match:** 80%+
- **Crashes:** 0

## Status
- **Current Phase:** Task assignment and initialization
- **Last Updated:** 2026-01-14
- **Next Action:** Create WORKER task lists and notify workers

## Strategy for EM-3

Based on PROJECT_DIRECTION.md priority order:

1. **Phase 1 - Parser (worker-9):** Fix noise first. Without a clean AST, semantic errors are unreliable.
2. **Phase 2 - Binder (worker-10):** Fix global scope. Stop error poisoning from undefined symbols.
3. **Phase 3 - Solver (worker-11):** Invert defaults. This WILL increase extra errors temporarily - this is expected and good.
4. **Phase 4 - CFA (worker-12):** Add strictPropertyInitialization check.

**Important:** Do NOT parallelize all phases. Worker-9 must make significant progress before worker-10's changes will be meaningful. Worker-11's changes will cause a temporary regression in metrics - this is intentional and exposes hidden bugs.
