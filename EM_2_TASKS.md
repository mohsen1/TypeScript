# EM-2 TASK LIST

## Team: em-team-2
## Manager: EM-2
## Base Branch: rust
## Team Branch: em-team-2

---

## Team Composition

| Worker | Squad | Focus Area | Status |
|--------|-------|------------|--------|
| worker-5 | Syntax Squad | Parser accuracy (TS1005, TS1109) | Active |
| worker-6 | Binder Squad | Global scope, lib injection (TS2304) | Active |
| worker-7 | Semantics Squad | Module resolution, solver defaults | Active |
| worker-8 | LSP Squad | LSP config integration, TS2564 | Active |

---

## Team Priorities (Updated 2026-01-15)

### Priority 1: Parser Accuracy (Tier 1)
**Owner:** worker-5 (Syntax Squad)

**Goal:** Reduce parser noise - eliminate false syntax errors

| Error Code | Current | Target | Owner |
|------------|---------|--------|-------|
| TS1005 extra | ~345 | <50 | worker-5 |
| TS1109 extra | ~701 combined | <100 | worker-5 |

**Action Items:**
- Statement-level error recovery enhancement (in progress)
- ASI edge cases (mostly complete)
- Object/array literal error recovery (complete)

### Priority 2: Module Symbol Resolution (Tier 3)
**Owner:** worker-7 (Semantics Squad)

**Goal:** Fix cross-file symbol resolution

| Error Code | Current | Target | Owner |
|------------|---------|--------|-------|
| TS7005 extra | 489 | <100 | worker-7 |
| TS7008 extra | 336 | <50 | worker-7 |
| TS2792 missing | 161 | <20 | worker-7 |

**Action Items:**
- Build export tables for each module
- Resolve imports against export tables
- Handle re-exports and dynamic imports

### Priority 2.5: Global Scope / Lib Injection
**Owner:** worker-6 (Binder Squad)

**Goal:** Ensure lib.d.ts globals are available

| Error Code | Current | Target | Owner |
|------------|---------|--------|-------|
| TS2304 extra | 337 | <50 | worker-6 |
| TS2454 | Fixed | - | worker-6 (completed) |

**Action Items:**
- Verify lib.d.ts symbol injection works in all contexts
- Fix global interface merging
- Address remaining TS2304 gaps

### Priority 3: Type Checker Accuracy (Tier 2)
**Owner:** worker-7, worker-8

**Goal:** Emit correct semantic errors

| Error Code | Status | Owner |
|------------|--------|-------|
| TS2571 extra | Needs investigation | Unassigned |
| TS2683 missing | Needs investigation | Unassigned |
| TS2564 | Verified complete | worker-8 |

### Priority 4: LSP Enhancements
**Owner:** worker-8 (LSP Squad)

**Goal:** Improve LSP accuracy and user experience

| Task | Status | Owner |
|------|--------|-------|
| LSP TypeScript config integration | Approved, ready to start | worker-8 |
| TS2564 verification | Complete | worker-8 |

---

## Conformance Test Baseline (2026-01-14)

Based on worker-7's latest validation:

| Metric | Result | Goal |
|--------|--------|------|
| Tests Run | 4941 | - |
| Exact Match | 1409 (28.5%) | 95%+ |
| Same Error Count | 1536 (31.1%) | - |
| Combined Parity | 59.6% | 95%+ |
| Missing Errors | 2575 (52.1%) | <5% |
| Extra Errors | 2273 (46.0%) | <5% |

### Top Extra Errors (Over-reporting)
1. TS2322: 548 (Type Mismatch) - *Intentional regression from solver fix*
2. TS7005: 489 (Module symbol reference)
3. TS2304: 340 (Cannot find name)
4. TS7008: 336 (Module export member)
5. TS1005: 345 (Parser X expected)

### Top Missing Errors (Under-reporting)
1. TS2792: 161 (import() type resolution)
2. TS2304: 114 (Cannot find name - different category)
3. TS2322: 105 (Type mismatch - edge cases)
4. TS1005: 90 (Parser error recovery)
5. TS2339: 79 (Property access on unknown)

### Crashes
- 2 stack overflows remain (recursive types)
- TS2589 guards partially applied, need deeper recursion protection

---

## Workflow

### For EM-2:
1. **Daily sync**: `git pull origin rust` → merge to em-team-2
2. **Review worker branches**: Check commits, test results
3. **Merge locally**: `git merge worker-X` into em-team-2
4. **Run validation**: `./wasm/differential-test/run-conformance.sh`
5. **Push to director**: Only when stable and validated

### For Workers:
1. Create branch from em-team-2
2. Work on assigned task ONLY
3. Commit frequently with `[wasm] <component>: <description>`
4. Push to worker-X branch
5. Update task list with status
6. Notify EM-2 when ready for merge

---

## Merge Readiness Status (2026-01-15)

| Worker | Status | Notes |
|--------|--------|-------|
| worker-5 | 🟡 Active | Working on statement-level error recovery |
| worker-6 | 🟢 Ready | TS2454 fix complete, needs review |
| worker-7 | 🟡 Active | Module resolution in progress |
| worker-8 | 🟡 Approved | LSP config integration approved, awaiting start |

---

## Escalation Path

1. Worker commits → worker-X branch
2. EM-2 merges to em-team-2 → validates
3. EM-2 escalates to Director when stable
4. Director reviews → merges to rust

**Do NOT push directly to rust.**

---

## Recent Activity Log

### 2026-01-15
- EM-2 initialized
- Created em-team-2 branch
- Copied worker task lists
- Established priorities

### 2026-01-14 (from worker reports)
- worker-5: ASI complete, error suppression merged
- worker-6: TS2454 fix merged, TS2589 complete
- worker-7: Solver defaults inverted, module resolution started
- worker-8: TS2564 verified, LSP config integration proposed

---

## Next Actions for EM-2

1. ✅ Sync em-team-2 with rust
2. ✅ Create EM_2_TASKS.md
3. 🔄 Review worker-6 for merge (TS2454)
4. 🔄 Review worker-8 LSP proposal (approved)
5. 📅 Run conformance tests on em-team-2 baseline
6. 📋 Assign next tasks based on priorities

---

## Notes

- All work must stay in `wasm/` directory
- TypeScript source files (`src/compiler/`) are READ-ONLY
- Run `./wasm/test.sh` for Rust tests (Docker-only)
- Use `./scripts/ask-gemini.mjs` before coding (MANDATORY)
- Target: 95%+ exact match before production
