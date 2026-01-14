# EM-2 Task List

**Engineering Manager:** EM-2
**Branch:** `em-team-2`
**Base Branch:** `rust`
**Workers:** worker-5, worker-6, worker-7, worker-8

*Last Updated: 2026-01-14*

---

## Team Mission

Keep `em-team-2` in sync with `rust`. Assign focused work to workers, merge their branches locally, validate, and escalate to the Director only when stable.

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

### Worker-5: Binder Squad (Critical Path)
**Branch:** `worker-5`
**Focus:** Lib Injection and Global Scope
**Target:** Fix lib.d.ts loading and global symbol resolution

**Tasks:**
1. Debug `lib_loader.rs` - verify it correctly merges `lib.d.ts` symbols into root `SymbolTable`
2. Fix module augmentation resolution (merging `interface Window` across files)
3. Ensure DOM globals (`console`, `Promise`, `Array`) resolve in all files
4. Verify library context propagation to `file_locals`

**Key Files:**
- `src/lib_loader.rs`
- `src/thin_binder.rs`
- `src/symbol_table.rs`

---

### Worker-6: Binder Squad (Critical Path)
**Branch:** `worker-6`
**Focus:** File Locals and Scope Chain
**Target:** Fix `file_locals` population and symbol accessibility

**Tasks:**
1. Fix `file_locals` population from library context
2. Ensure global symbols are accessible in all files via scope chain
3. Debug namespace/import resolution edge cases
4. Verify symbol table merging logic for multi-file projects

**Key Files:**
- `src/thin_binder.rs`
- `src/symbol_table.rs`

---

### Worker-7: Solver Squad (Strategic)
**Branch:** `worker-7`
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

### Worker-8: CFA Squad (Partial)
**Branch:** `worker-8`
**Focus:** Control Flow Analysis Polish
**Target:** Reduce TS2454/TS2564 missing errors

**Tasks:**
1. Implement TS2454 - Variable Use Before Assignment Detection
2. Connect Flow Graph to Checker via `check_identifier`
3. Implement TS2564 - Property Initialization Detection
4. Validate with conformance suite

**Key Files:**
- `src/checker/control_flow.rs`
- `src/checker/flow_graph_builder.rs`

---

## EM-2 Responsibilities

### Daily Operations
1. **Sync:** `git pull origin rust` and merge into `em-team-2`
2. **Monitor:** Check worker branches for progress
3. **Validate:** Run conformance tests on merged worker branches
4. **Escalate:** Only push to Director when all worker branches merge cleanly

### Validation Checklist Before Escalation
- [ ] All 4 worker branches merge into `em-team-2` without conflicts
- [ ] Conformance tests run: `npm run test:conformance`
- [ ] No regressions in Exact Match score
- [ ] TS2304 extra errors < 50 (Binder squad target)
- [ ] Missing errors < 50% (overall target)

### Merge Workflow
```bash
# Sync with base
git checkout em-team-2
git pull origin rust
git merge worker-5 --no-ff -m "Merge worker-5: Lib injection fixes"
git merge worker-6 --no-ff -m "Merge worker-6: Scope resolution"
git merge worker-7 --no-ff -m "Merge worker-7: Solver strictness"
git merge worker-8 --no-ff -m "Merge worker-8: CFA implementation"

# Run validation
npm run test:conformance

# If stable, push to remote
git push origin em-team-2
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
