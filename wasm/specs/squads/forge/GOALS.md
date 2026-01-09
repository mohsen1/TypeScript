# Squad Forge Goals

Updated: 2026-01-09 (14:00)

Priority: 1

---
## Current Conformance Baseline (5655 tests)

| Metric | Value | Notes |
|--------|-------|-------|
| Exact Match | 23.3% (1148/4928) | Up from 18.1% |
| Same Error Count | 26.2% (1293) | |
| Tests with Missing Errors | 68.2% (3361) | WASM misses errors TSC catches |
| Tests with Extra Errors | 35.8% (1766) | False positives |
| Skipped (multi-file) | 727 | Need WasmProgram API fixes |
| Crashed | 2 | Stack overflow, unreachable |

---
## Top Missing Error Codes (PARALLELIZABLE WORK)

These are errors TSC produces that WASM doesn't. Each worker can own one error code independently.

| TS Code | Occurrences | Description | Difficulty | Worker |
|---------|-------------|-------------|------------|--------|
| **TS2454** | 573 | Variable used before assigned | Medium | W1 |
| **TS2564** | 443 | Property not initialized in constructor | Medium | W2 |
| **TS7006** | 357 | Parameter implicitly has 'any' type | Easy | W3 |
| **TS2322** | 310 | Type not assignable | Hard | - |
| **TS2792** | 204 | Cannot find module | Easy | W4 |
| **TS7010** | 179 | Function must return a value | Medium | W5 |
| **TS7008** | 169 | Member implicitly has 'any' type | Easy | W3 |
| **TS2339** | 142 | Property does not exist | Hard | - |
| **TS2304** | 138 | Cannot find name | Hard | - |
| **TS2300** | 105 | Duplicate identifier | Easy | - |

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
- Last Update: 2026-01-09 14:00
- Conformance: **23.3% exact match** (up from 18.1%)
- Workers: 5 available
- Strategy: Each worker owns one error code, implement independently
