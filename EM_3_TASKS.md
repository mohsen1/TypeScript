# EM-3 Task List

## Team: Engineering Manager 3
- **Workers**: worker-9, worker-10, worker-11
- **Branch**: em-team-3
- **Worktree**: /tmp/orchestrator-workspace/worktrees/em-3
- **Squad Focus**: Parser/Scanner ONLY (TS1005/TS1109)

## Mission Phase: Phase 8 - Conformance, Convergence, and Hardening

### Current Status
| Metric | Value | Target |
|--------|-------|--------|
| Exact Match | 30.1% | 40% |
| Missing Errors | 60.0% | <50% |
| Parser false positives | 701 | <100 |
| TS2304 extra errors | 343 | <50 |

### Critical Problem: "Any" Poisoning
The compiler defaults to `Any` when it encounters unresolved symbols, silencing downstream errors. This masks real progress.

---

## Squad Assignments

### Parser/Scanner Squad (3 workers) - TS1005/TS1109 Focus
**Workers**: worker-9, worker-10, worker-11
**Priority**: HIGH - 701 false positives inflate "Extra Errors" by 14%

**Directive**:
- Audit TS1005 ("expected X") emission - likely over-triggering on valid syntax
- Audit TS1109 ("expression expected") - false positives on edge cases
- Reduce parser false positives to <100

**Key Files**:
- `src/compiler/parser.ts` (540K lines)
- `src/compiler/scanner.ts` (219K lines)
- `TS1005_REDUCTION_RESULTS.md` - reference for patterns already fixed
- `TS1109_ANALYSIS.md` - reference for TS1109 analysis

**Success Metrics**:
- TS1005: 439 → <100
- TS1109: 262 → <50
- Total parser false positives: 701 → <100

### Worker Breakdown

| Worker | Primary Focus | Secondary Focus |
|--------|---------------|-----------------|
| **worker-9** | TS1005 patterns 6-10 (object/array literals) | TS1109 statement parsing |
| **worker-10** | TS1109 expression expected errors | TS1005 type parameters |
| **worker-11** | TS1005 patterns 11-15 (edge cases) | TS1109 class member parsing |

---

## Team Workflow

### 1. Task Assignment
- EM-3 maintains this file
- Updates each `WORKER_<id>_TASK_LIST.md` when assigning tasks
- Workers work on individual branches

### 2. Integration Process
- Workers push to their feature branches
- EM-3 merges locally to em-team-3
- Run validation (`npm test` or conformance tests)
- Escalate to Director only when stable

### 3. Validation Commands
```bash
# Run conformance tests
npm run test:conformance

# Run unit tests
npm test

# Build compiler
npm run build

# Check specific error patterns
npm run test -- --grep "TS1005"
npm run test -- --grep "TS2304"
```

---

## Current Sprint Goals

1. **Worker 9**: Fix TS1005 patterns 6-10 (object literals, arrays, edge cases)
2. **Worker 10**: Fix TS1109 false positives (expression expected errors)
3. **Worker 11**: Fix TS1005 patterns 11-15 (edge cases: type parameters, return types, statements)

---

## Next Actions

- [x] Create WORKER_9_TASK_LIST.md
- [x] Create WORKER_10_TASK_LIST.md
- [x] Update WORKER_11_TASK_LIST.md (Parser focus)
- [x] Remove WORKER_12_TASK_LIST.md (reassigned)
- [ ] Run baseline conformance tests
- [ ] Assign updated tasks to workers 9-11
