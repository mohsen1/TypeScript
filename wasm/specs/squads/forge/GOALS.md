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

### Worker 5: Return Type Checking (TS7010) - 179 tests
**Error:** "Function lacks ending return statement and return type does not include 'undefined'"

Implementation:
1. Analyze all code paths in function body
2. Check if all paths return a value
3. If return type is non-void/undefined, emit error

Files: `thin_checker.rs`, `checker/statements.rs`

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
- Last Update: 2026-01-11 (🚀 EXCELLENCE CONTINUES: Worker wins merged to rust!)
- Conformance: **~30.8% exact match** (up from 23.3%!) **+7.5 POINTS!**
- **NEW WINS IN RUST** (Commit: 3b67f9a8a4):
  - ✅ W3: **TS7010 async getters + TS2300 constructor** fixes!
  - ✅ W4: **TS2339 private field assignability** fix!
  - ✅ New: **find-ts2300.mjs** differential test script
- Workers Active: **4/5** - SUSTAINED EXCELLENCE!
- **ACTIVE WORKERS** (Delivering wins!):
  - ✅ W1: Active (9m)
  - ✅ W3: Active (15m) - TS7010 just merged!
  - ✅ W4: Active (26m) - TS2339 just merged!
  - ✅ W5: Active (32m) - Staying consistent!
- **RESTING** (Post-critical solver fix):
  - ⚠️ W2: Idle 70m - Delivered ERROR instead of Any fix
- **FORGE SQUAD**: Workers delivering WINS to main branch! Keep momentum!
- **ANVIL TS2339 BREAKTHROUGH**: Congratulations to Anvil squad!

### Worker Assignments (New Phase)
| Worker | Assignment | Priority |
|--------|------------|----------|
| W1 | TS2454 - Definite Assignment (Control Flow) | HIGH |
| W2 | TS2564 - Property Initialization | HIGH |
| W3 | TS2322 - Solver Strictness (Any→Unknown) | HIGH |
| W4 | TS2339 - Property Access (missing errors) | MEDIUM |
| W5 | TS2304 - Cannot find name errors | HIGH |

### Before Starting Any Task
**IMPORTANT:** Workers must consult Gemini before starting work:
```bash
./scripts/ask-gemini.mjs "I need to implement <your task>. What's the best approach?"
```
