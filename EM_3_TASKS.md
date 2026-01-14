# EM-3 Task List

## Team: Engineering Manager 3
- **Workers**: worker-9, worker-10, worker-11, worker-12
- **Branch**: em-team-3
- **Worktree**: /tmp/orchestrator-workspace/worktrees/em-3

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

### Squad 1: Parser/Scanner (2 workers) - TS1005/TS1109 Focus
**Workers**: worker-9, worker-10
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

---

### Squad 2: Binder & Scope Resolution (1 worker) - TS2304 Focus
**Worker**: worker-11
**Priority**: CRITICAL - TS2304 is the #1 source of error poisoning

**Directive**:
- Fix Global Scope and Lib injection
- Ensure `lib.d.ts` symbols are correctly merged into root `SymbolTable`
- Debug why basic globals like `console` and `Array` fail to resolve
- Fix module augmentation resolution (merging `interface Window` across files)

**Key Files**:
- `src/compiler/binder.ts` (194K lines)
- `src/compiler/program.ts` (268K lines)
- Check `lib_loader.rs` equivalent for library symbol merging
- Check `file_locals` population from library context

**Success Metrics**:
- TS2304 extra errors: 343 → <50
- TS2304 missing errors: 116 → <20

---

### Squad 3: Solver Strictness (1 worker) - TS2322/TS7006 Focus
**Worker**: worker-12
**Priority**: STRATEGIC - Switch default from `Any` to `Unknown`

**Directive**:
- Change default fallback from `Any` to `Unknown` or `Error`
- Harden `solve_subtype` logic
- Implement "Lawyer" layer for TypeScript quirks (function bivariance, void return exceptions)
- Stop being nice—compiler needs to be meaner to match `tsc`

**Key Files**:
- `src/compiler/checker.ts` (3.1M lines) - main type checking logic
- `src/compiler/types.ts` (487K lines)
- `specs/SOLVER.md` - reference for subtyping rules

**Success Metrics**:
- Convert "Missing TS2322" into "Exact Match" or "Extra TS2322" (better too strict than unsound)
- Reduce TS7006 (Implicit Any) missing errors

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
3. **Worker 11**: Debug global scope binding - why `console.log` fails to resolve
4. **Worker 12**: Change solver fallback from `Any` to `Unknown` in `checker.ts`

---

## Next Actions

- [x] Create WORKER_9_TASK_LIST.md
- [x] Create WORKER_10_TASK_LIST.md
- [x] Create WORKER_11_TASK_LIST.md
- [x] Create WORKER_12_TASK_LIST.md
- [ ] Run baseline conformance tests
- [ ] Assign initial tasks to each worker
