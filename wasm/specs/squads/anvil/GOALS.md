# Squad Anvil Goals

Updated: 2026-01-09 (14:00)

Priority: 1

---
## Current Conformance Baseline (5655 tests)

| Metric | Value | Notes |
|--------|-------|-------|
| Exact Match | 23.3% (1148/4928) | Up from 18.1% |
| Tests with Extra Errors | 35.8% (1766) | False positives - WASM says error, TSC doesn't |
| Skipped (multi-file) | 727 | Need WasmProgram API fixes |
| Crashed | 2 | Stack overflow, unreachable |

---
## Top Extra Error Codes (PARALLELIZABLE WORK - FALSE POSITIVES)

These are errors WASM reports that TSC doesn't. Each worker can own one error code independently.

| TS Code | Occurrences | Description | Difficulty | Worker |
|---------|-------------|-------------|------------|--------|
| **TS2304** | 759 | Cannot find name (false positive) | Hard | W1 |
| **TS1005** | 548 | Expected X (parser bug) | Medium | W2 |
| **TS2339** | 292 | Property does not exist (false positive) | Hard | W3 |
| **TS1109** | 273 | Expression expected (parser bug) | Medium | W2 |
| **TS1068** | 200 | Unexpected token (parser bug) | Medium | W2 |
| **TS2769** | 125 | No overload matches (false positive) | Hard | W4 |
| **TS2355** | 116 | Function must return (false positive) | Medium | W5 |
| **TS1128** | 101 | Declaration expected (parser bug) | Medium | W2 |
| **TS2322** | 101 | Type not assignable (false positive) | Hard | - |
| **TS2403** | 96 | Subsequent variable declarations (false positive) | Medium | - |

---
## Phase 10: False Positive Elimination

### Worker 1: Scope Resolution (TS2304) - 759 false positives
**Problem:** "Cannot find name 'X'" when X is clearly defined

Root Causes:
1. Namespace members not finding sibling exports
2. Module augmentation not merging correctly
3. Global ambient declarations not registered

Files: `thin_binder.rs`, `thin_checker.rs`

### Worker 2: Parser Bugs (TS1005/TS1109/TS1068/TS1128) - 1122 combined
**Problem:** Valid TypeScript syntax rejected by parser

Common patterns failing:
1. Type assertions in certain positions
2. Generic type parameters with defaults
3. JSX-like syntax in .ts files
4. Computed property names

Files: `parser/` - all parse functions

### Worker 3: Property Access (TS2339) - 292 false positives
**Problem:** "Property 'X' does not exist on type 'Y'" when it does

Root Causes:
1. Type narrowing not applied correctly
2. Index signatures not considered
3. Interface merging incomplete
4. Prototype chain not followed

Files: `thin_checker.rs` - property access checking

### Worker 4: Overload Matching (TS2769) - 125 false positives
**Problem:** "No overload matches this call" when one should

Root Causes:
1. Generic inference in overloads too strict
2. Rest parameter matching incorrect
3. Optional parameter handling wrong

Files: `thin_checker.rs`, `solver/` - call resolution

### Worker 5: Return Analysis (TS2355) - 116 false positives
**Problem:** "A function whose declared type is neither 'void' nor 'any' must return a value"

Root Causes:
1. Throw statements not counted as exits
2. Never-returning calls not recognized
3. Unreachable code after return still analyzed

Files: `thin_checker.rs`, `checker/control_flow.rs`

---
## Category Performance (Priority: Fix worst categories)

| Category | Exact Match | False Positive Rate | Focus |
|----------|-------------|---------------------|-------|
| interfaces | 9% (6/66) | HIGH | **CRITICAL** |
| async | 7% (12/179) | HIGH | Medium |
| types | 14% (117/826) | HIGH | **CRITICAL** |
| expressions | 14% (53/372) | Medium | HIGH |
| decorators | 8% (6/76) | Medium | LOW |
| internalModules | 17% (11/63) | Medium | Medium |

---
## Crashed Files (Fix Required)

These 2 files crash the WASM checker:

1. `es6/templates/TemplateExpression1.ts` - **unreachable**
   - Likely missing case in template literal handling

2. `types/mapped/recursiveMappedTypes.ts` - **Maximum call stack size exceeded**
   - Infinite recursion in recursive mapped type

---
## Anti-Priorities
- New emitter features
- Source map improvements
- LSP features
- CLI enhancements

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

# Single category
node conformance-runner.mjs expressions --max=500
```

---
## Worker Workflow (CRITICAL)

**Before starting work, run ALL conformance tests to establish baseline:**

```bash
cd wasm/differential-test
bash run-conformance.sh --all --workers=14
# Record: exact match %, extra error count, crash count
```

**Before completing work, run ALL tests again to verify no regressions:**

```bash
bash run-conformance.sh --all --workers=14
# Compare against your baseline
```

**Why:** Fixing one error code can easily break another. For example:
- Fixing TS2304 (scope resolution) might introduce new TS2339 (property access) errors
- Parser fixes for TS1005 might cause new TS1109 errors
- Type narrowing changes affect multiple error codes

**Acceptance Criteria:**
1. Target error code occurrences must decrease
2. Overall exact match % must NOT decrease from YOUR baseline
3. No new crashes introduced
4. Document any trade-offs in commit messages

---
## Squad Status
- Last Update: 2026-01-09 14:00
- False Positives: **1766 tests (35.8%)** - target: <500
- Workers: 5 available
- Strategy: Each worker owns one error code category, reduce false positives independently
