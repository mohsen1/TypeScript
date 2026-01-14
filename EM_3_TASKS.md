# EM-3 Task List

**Engineering Manager:** EM-3
**Branch:** `em-team-3`
**Base Branch:** `rust`
**Workers:** worker-9, worker-10, worker-11, worker-12

*Last Updated: 2025-01-14*

---

## Team Mission

Keep `em-team-3` in sync with `rust`. Assign focused work to workers, merge their branches locally, validate, and escalate to the Director only when stable.

---

## Current Project Status (Phase 8)

| Metric | Value | Target |
|--------|-------|--------|
| **Exact Match** | 30.1% | 40% |
| **Missing Errors** | 60.0% | <50% |
| **TS2304 Extra Errors** | 343 | <50 |
| **Parser False Positives** | 701 | <100 |

**Critical Issue:** "Error Poisoning" from `Any` type defaults. The compiler is too permissive—when binding fails, it defaults to `Any`, silencing downstream errors.

---

## Squad Assignments

### Worker-9: Parser Squad (High Impact)
**Branch:** `worker-9`
**Focus:** Reduce False Positives
**Target:** Reduce TS1005/TS1109 false positives to <100

**Status:** Awaiting task assignment
**Tasks:**
1. Audit TS1005 ("expected X") emission - likely over-triggering on valid syntax
2. Audit TS1109 ("expression expected") - false positives on edge cases
3. Implement better error recovery ("resynchronization")
4. Ensure parser continues after minor syntax errors

**Key Files:**
- `src/thin_parser.rs`

---

### Worker-10: CFA Squad (Partial)
**Branch:** `worker-10`
**Focus:** Edge-Case Polish
**Target:** Reduce TS2564 missing errors, fix TS2454 extra errors

**Status:** Awaiting task assignment
**Tasks:**
1. Polish TS2564 (variable used before assignment) edge cases
2. Fix TS2454 (variable before assignment) extra errors (225 occurrences)
3. Verify flow graph side-table integration
4. Test CFA logic with complex control flow patterns

**Key Files:**
- `src/cfa.rs`
- `src/flow_graph.rs`

---

### Worker-11: Binder Squad (Critical Path)
**Branch:** `worker-11`
**Focus:** Scope Resolution and Symbol Table
**Target:** Reduce TS2304 extra errors to <50

**Status:** Awaiting task assignment
**Tasks:**
1. Fix `file_locals` population from library context
2. Ensure global symbols are accessible in all files
3. Debug namespace/import resolution edge cases
4. Verify symbol table merging logic

**Key Files:**
- `src/thin_binder.rs`
- `src/symbol_table.rs`

---

### Worker-12: Solver Squad (Strategic)
**Branch:** `worker-12`
**Focus:** Strictness Enforcement
**Target:** Switch from `Any` to `Unknown`/`Error` defaults

**Status:** Awaiting task assignment
**Tasks:**
1. Change `lower_type` to return `Error` instead of `Any` on resolution failure
2. Implement "Lawyer" layer from `specs/SOLVER.md` for TypeScript quirks
3. Harden `solve_subtype` logic (function bivariance, void return exceptions)
4. Convert "Missing TS2322" into either "Exact Match" or "Extra TS2322"

**Key Files:**
- `src/solver/mod.rs`
- `specs/SOLVER.md`

---

## EM-3 Responsibilities

### Daily Operations
1. **Sync:** `git pull origin rust` and merge into `em-team-3`
2. **Monitor:** Check worker branches for progress
3. **Validate:** Run conformance tests on merged worker branches
4. **Escalate:** Only push to Director when all worker branches merge cleanly

### Validation Checklist Before Escalation
- [ ] All 4 worker branches merge into `em-team-3` without conflicts
- [ ] Conformance tests run: `npm run test:conformance`
- [ ] No regressions in Exact Match score
- [ ] TS2304 extra errors < 50 (Binder squad target)
- [ ] Parser false positives < 100 (Parser squad target)

### Merge Workflow
```bash
# Sync with base
git checkout em-team-3
git pull origin rust
git merge worker-9 --no-ff -m "Merge worker-9: Parser fixes"
git merge worker-10 --no-ff -m "Merge worker-10: CFA polish"
git merge worker-11 --no-ff -m "Merge worker-11: Binder fixes"
git merge worker-12 --no-ff -m "Merge worker-12: Solver strictness"

# Run validation
npm run test:conformance

# If stable, push to remote
git push origin em-team-3
```

---

## Anti-Priorities (Do Not Assign)

- Performance optimization (speed is sufficient at 41.7 tests/sec)
- New Emitter features (downleveling is stable)
- LSP polish (no new code actions until semantics accurate)

---

## Success Metrics

When all squads complete their targets:
- Exact Match: 40%+
- Missing Errors: <50%
- TS2304 Extra: <50
- Parser False Positives: <100

**Only then escalate to Director for integration into `rust`.**

---

## Recent Merge Activity

| Date | Worker | Status | Notes |
|------|--------|--------|-------|
| 2025-01-14 | worker-9 | Merged | Branch at rust HEAD, no conflicts |
| 2025-01-14 | worker-11 | Merged | Branch at rust HEAD, no conflicts |
| 2025-01-14 | worker-12 | Merged | Branch at rust HEAD, no conflicts |
