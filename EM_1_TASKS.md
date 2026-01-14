# EM-1 Task List

**Engineering Manager:** EM-1
**Branch:** `em-team-1`
**Base Branch:** `rust`
**Workers:** worker-1, worker-2, worker-3, worker-4

*Last Updated: 2025-01-14*

---

## Team Mission

Keep `em-team-1` in sync with `rust`. Assign focused work to workers, merge their branches locally, validate, and escalate to the Director only when stable.

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

### Worker-1: Binder Squad (Critical Path)
**Branch:** `worker-1`
**Focus:** Global Scope and Lib Injection
**Target:** Reduce TS2304 extra errors to <50

**Tasks:**
1. Debug why `console.log` fails to resolve (check `lib.dom.d.ts` loading)
2. Verify `lib_loader.rs` correctly merges `lib.d.ts` symbols into root `SymbolTable`
3. Fix module augmentation resolution (merging `interface Window` across files)
4. Debug basic globals (`console`, `Array`, `Promise`) resolution failures

**Key Files:**
- `src/lib_loader.rs`
- `src/thin_binder.rs`

---

### Worker-2: Binder Squad (Critical Path)
**Branch:** `worker-2`
**Focus:** Scope Resolution and Symbol Table
**Target:** Reduce TS2304 missing errors to <50

**Tasks:**
1. Fix `file_locals` population from library context
2. Ensure global symbols are accessible in all files
3. Debug namespace/import resolution edge cases
4. Verify symbol table merging logic

**Key Files:**
- `src/thin_binder.rs`
- `src/symbol_table.rs`

---

### Worker-3: Solver Squad (Strategic)
**Branch:** `worker-3`
**Focus:** Strictness Enforcement
**Target:** Switch from `Any` to `Unknown`/`Error` defaults

**Tasks:**
1. Change `lower_type` to return `Error` instead of `Any` on resolution failure
2. Implement "Lawyer" layer from `specs/SOLVER.md` for TypeScript quirks
3. Harden `solve_subtype` logic (function bivariance, void return exceptions)
4. Convert "Missing TS2322" into either "Exact Match" or "Extra TS2322"

**Key Files:**
- `src/solver/mod.rs`
- `specs/SOLVER.md`

---

### Worker-4: Parser Squad (High Impact)
**Branch:** `worker-4`
**Focus:** Reduce False Positives
**Target:** Reduce TS1005/TS1109 false positives to <100

**Tasks:**
1. Audit TS1005 ("expected X") emission - likely over-triggering
2. Audit TS1109 ("expression expected") - false positives on edge cases
3. Implement better error recovery ("resynchronization")
4. Ensure parser continues after minor syntax errors

**Key Files:**
- `src/thin_parser.rs`

---

## EM-1 Responsibilities

### Daily Operations
1. **Sync:** `git pull origin rust` and merge into `em-team-1`
2. **Monitor:** Check worker branches for progress
3. **Validate:** Run conformance tests on merged worker branches
4. **Escalate:** Only push to Director when all worker branches merge cleanly

### Validation Checklist Before Escalation
- [ ] All 4 worker branches merge into `em-team-1` without conflicts
- [ ] Conformance tests run: `npm run test:conformance`
- [ ] No regressions in Exact Match score
- [ ] TS2304 extra errors < 50 (Binder squad target)
- [ ] Parser false positives < 100 (Parser squad target)

### Merge Workflow
```bash
# Sync with base
git checkout em-team-1
git pull origin rust
git merge worker-1 --no-ff -m "Merge worker-1: Binder fixes"
git merge worker-2 --no-ff -m "Merge worker-2: Scope resolution"
git merge worker-3 --no-ff -m "Merge worker-3: Solver strictness"
git merge worker-4 --no-ff -m "Merge worker-4: Parser fixes"

# Run validation
npm run test:conformance

# If stable, push to remote
git push origin em-team-1
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
