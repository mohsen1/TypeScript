# Squad Forge Goals

Updated: 2026-01-11

Priority: 1

---
## Current Conformance Baseline

| Metric | Current | Previous | Target | Status |
|--------|---------|----------|--------|--------|
| Exact Match | **30.8%** | 23.3% | 50%+ | +7.5pp |
| Missing Errors | **57.8%** | 68.2% | <30% | -10.4pp |
| Extra Errors | **28.9%** | 35.8% | <20% | -6.9pp |
| Parser Errors | **~85** | 1,122 | <100 | **TARGET MET** |
| **Crashes** | **~478** 🚨 | ~143 | **0** | **CRITICAL REGRESSION** |

---

## 🚨 CRISIS: Crash Investigation (P0) - ASSIGNED TO W5

**Status:** Parser panics increased from ~143 to ~478 (10% test suite invalidation)

**Root Cause:** Recent merges (72+ commits affecting checker/binder) introduced regressions

**Investigation Priority:**
1. 1,453 potential panic sites in checker/solver/binder
2. Recent `unwrap()`/`expect()` calls in type checking
3. Missing null checks in property access
4. Symbol resolution failures

**Action:** W5 reassigned from TS7010 to **CRASH INVESTIGATION** 🔥

---

## Completed Work (Reference)

| TS Code | Result | Notes |
|---------|--------|-------|
| ~~TS7006~~ | 74% reduction | 46 → 12 false positives |
| ~~TS2304~~ | 98.7% reduction | 759 → 10 false positives |
| ~~TS2769~~ | Complete | Overload matching done |
| ~~Parser~~ | 92% reduction | 1,122 → 85 errors |

---
## Next Phase: Control Flow + Solver Strictness

**Priority Focus (Gemini-recommended):**

| TS Code | Occurrences | Description | Difficulty | Worker |
|---------|-------------|-------------|------------|--------|
| **TS2454** | 573 | Variable used before assigned | Medium | W1 |
| **TS2564** | 443 | Property not initialized in constructor | Medium | W2 |
| **TS2322** | 310 | Type not assignable (solver strictness) | Hard | W3 |
| **TS2339** | 142 | Property does not exist (missing) | Hard | W4 |
| **TS7010** | 179 | Function must return a value | Medium | W5 |

---
## Phase 10: Conformance Sprint

### Worker 1: Definite Assignment (TS2454) - 573 tests
**Error:** "Variable 'x' is used before being assigned"

Implementation:
1. Add control flow analysis to track variable assignments
2. Before each variable read, check if definitely assigned
3. Handle conditional branches (if/else must both assign)

Files: `thin_checker.rs`, `checker/control_flow.rs`

### Worker 2: Property Initialization (TS2564) - 443 tests
**Error:** "Property 'x' has no initializer and is not definitely assigned in constructor"

Implementation:
1. Track which properties are assigned in constructor
2. For each class property without initializer, check constructor assigns it
3. Handle `strictPropertyInitialization` flag

Files: `thin_checker.rs`, `checker/class_checker.rs`

### Worker 3: Implicit Any (TS7006/TS7008) - 526 tests combined
**Error:** "Parameter/Member implicitly has 'any' type"

Implementation:
1. Check function parameters have explicit types or can be inferred
2. Check class members have types when `noImplicitAny` is set
3. Emit error when type falls back to `any`

Files: `thin_checker.rs` - add `check_implicit_any()` pass

### Worker 4: Module Resolution (TS2792) - 204 tests
**Error:** "Cannot find module 'x' or its corresponding type declarations"

Implementation:
1. Track which imports couldn't be resolved
2. Emit proper error code (2792 vs 2307)
3. Handle relative vs package imports

Files: `thin_checker.rs`, `thin_binder.rs`

### Worker 5: 🚨 CRASH INVESTIGATION (P0) - ~478 parser panics
**CRITICAL:** 10% of test suite is invalidating due to panics

**Root Cause:** Recent checker/binder commits introduced regressions

**Investigation Steps:**
1. Find all `unwrap()`, `expect()`, `panic!` in checker/solver/binder
2. Add proper error handling and propagation
3. Test against failing test cases to identify specific crash
4. Fix regressions with proper Option/Result handling

**Files:** `thin_checker.rs`, `checker/*.rs`, `solver/*.rs`, `binder/*.rs`

**Previous Task (TS7010):** Suspended due to P0 crisis - will resume after crashes fixed

---
## Category Breakdown (from conformance tests)

| Category | Total | Exact Match | Priority |
|----------|-------|-------------|----------|
| es6 | 991 | 36% (360) | Medium |
| types | 826 | 14% (117) | **HIGH** |
| parser | 768 | 28% (215) | Medium |
| classes | 456 | 25% (116) | **HIGH** |
| expressions | 372 | 14% (53) | HIGH |
| statements | 202 | 18% (37) | Medium |
| externalModules | 190 | 19% (36) | Medium |
| async | 179 | 7% (12) | LOW |
| jsdoc | 148 | 28% (42) | LOW |
| interfaces | 66 | 9% (6) | **HIGH** |

**Focus on: types, classes, expressions, interfaces** - lowest match rates, core type system

---
## Anti-Priorities
- Template literal types
- JSDoc inference
- Decorators (ES vs legacy)
- Async/await edge cases

---
## Running Conformance Tests

```bash
# Fast parallel run (14 workers)
cd wasm/differential-test
bash run-conformance.sh --all --workers=14

# Quick subset (500 tests)
bash run-conformance.sh --max=500 --workers=8

# Sequential (for debugging)
bash run-conformance.sh --sequential --max=100
```

---
## Worker Workflow (CRITICAL)

**Before starting work, run ALL conformance tests to establish baseline:**

```bash
cd wasm/differential-test
bash run-conformance.sh --all --workers=14
# Record: exact match %, missing error count, extra error count, crash count
```

**Before completing work, run ALL tests again to verify no regressions:**

```bash
bash run-conformance.sh --all --workers=14
# Compare against your baseline
```

**Why:** Implementing one error check can easily break others. For example:
- Adding TS2454 (definite assignment) might trigger false TS2322 (type assignability) errors
- TS7006 (implicit any) detection might conflict with existing type inference
- Control flow analysis changes affect multiple error codes

**Acceptance Criteria:**
1. Target error code occurrences must be implemented correctly
2. Overall exact match % must NOT decrease from YOUR baseline
3. No new crashes introduced
4. Extra error count must NOT increase significantly
5. Document any trade-offs in commit messages

---
## Squad Status
- Last EM Update: 2026-01-11 (W1 merged: 2 commits)
- Conformance: **30.8% exact match** (+7.5pp from 23.3%)
- Build: Passing
- Workers: W1 reassigned, all 5 active
- Recent: W1(2) → squad/forge → origin

### Worker Assignments (UPDATED: Crisis Response)

| Worker | Priority | Assignment | Status |
|--------|----------|------------|--------|
| W1 | HIGH | TS2454: Definite Assignment (CFG) | Active |
| W2 | HIGH | TS2564: Property Initialization | Active |
| W3 | P1 | TS2322: Solver Strictness (reduce `any` fallback) | **REEVALUATE** - currently on TS7006 |
| W4 | P1 | TS2792: Module Resolution (fix `any` in imports) | Active - correctly assigned |
| W5 | **P0** 🔥 | **CRASH INVESTIGATION** (478 panics) | **STOP TS7010 - SWITCH TO CRASHES** |

**PRIORITY ORDER:**
1. **W5** - Fix crashes immediately (10% test suite broken)
2. **W4** - Module resolution (reduces permissiveness)
3. **W3** - Switch to TS2322 (solver strictness)
4. **W1/W2** - Continue on control flow (important but not urgent)

### Before Starting Any Task
**IMPORTANT:** Workers must consult Gemini before starting work:
```bash
./scripts/ask-gemini.mjs "I need to implement <your task>. What's the best approach?"
```
