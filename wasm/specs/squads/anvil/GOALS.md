# Squad Anvil Goals

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
| **Parser Noise** | **598** 📉 | - | **0** | **HIGH PRIORITY** |

**Parser Noise Breakdown:**
- **TS1005:** 384 errors (missing tokens, expected syntax)
- **TS1109:** 214 errors (unexpected tokens, expression expected)

**Impact:** These 598 parser failures cascade into:
- False TS2304 (Cannot find name) - symbols not parsed
- False `any` types - missing nodes prevent inference
- Overall test invalidation

---
## Completed Work (Reference)

| Task | Result | Notes |
|------|--------|-------|
| ~~Parser Recovery~~ | **92% reduction** | 1,122 → 85 errors - **PRIORITY 0 COMPLETE** |
| ~~TS2304 Binding~~ | **98.7% reduction** | 759 → 10 false positives |
| ~~TS2769 Overloads~~ | **Complete** | All overload tests passing |
| ~~TS2339 Private~~ | **100% fixed** | Private member access resolved |
| ~~TS7006 Implicit Any~~ | **74% reduction** | 46 → 12 false positives |

---
## Next Phase: False Positive Elimination

**Remaining Extra Errors (False Positives):**

| TS Code | Occurrences | Description | Difficulty | Worker |
|---------|-------------|-------------|------------|--------|
| **TS2339** | 292 | Property does not exist (narrowing) | Hard | W1, W3 |
| **TS2355** | 116 | Function must return (control flow) | Medium | W2 |
| **TS2322** | 101 | Type not assignable (false positive) | Hard | W4 |
| **TS2403** | 96 | Subsequent variable declarations | Medium | W5 |
| **TS2304** | 10 | Cannot find name (namespace edge cases) | Easy | W1 |

---
## Phase 10: False Positive Elimination

### Worker 1: Scope Resolution (TS2304) - 759 false positives [⚠️ BLOCKED BY PARSER]

**BLOCKED:** TS2304 fixes cannot be validated until parser TS1005/TS1109 errors are fixed

**Alternative Work:**
- Assist W2 with parser regression tests
- Work on Emitter parity features
- Document type resolution architecture

**Original Problem:** "Cannot find name 'X'" when X is clearly defined

Root Causes:
1. Namespace members not finding sibling exports
2. Module augmentation not merging correctly
3. Global ambient declarations not registered

Files: `thin_binder.rs`, `thin_checker.rs`

### Worker 2: Parser Bugs (TS1005/TS1109/TS1068/TS1128) - 1122 combined ⚠️ PRIORITY 0
**Problem:** Valid TypeScript syntax rejected by parser → cascades into TS2304/Any types

**CRITICAL FIX REQUIRED - EXCLUSIVE FOCUS:**
1. **Implement Error Recovery/Synchronization:**
   - When parser hits unexpected token, DON'T bail with ErrorNode
   - Scan forward to next `;` or `}` and RESUME parsing
   - Goal: Complete AST even with syntax errors
2. **Fix TS1068 (Class Members):**
   - Review `parse_class_member` in `thin_parser.rs`
   - Add support for newer TS syntax: `override`, `accessor`, decorator combinations
3. **Verify:** Run `node wasm/differential-test/conformance-runner.mjs parser --max=500`

**Success Criteria:** Reduce parser error count from 1,122 to < 100

**DO NOT WORK ON:** TS2454/TS7006 until parser fixed (can't trust control flow on broken AST)

---

**⚠️ CRITICAL: W1, W3, W4 ARE BLOCKED BY PARSER ISSUES**

**Why:** Workers W1, W3, W4 work on type checking which requires a valid AST. Parser noise (598 errors) means:
- Missing symbols → false TS2304
- Incomplete AST → wrong type inference
- Cannot accurately test type checking fixes

**Immediate Actions:**
- **W1, W3, W4:** ASSIST W2 with parser recovery
  - Add regression tests for parser edge cases
  - Test parser against real-world TS code
  - Document expected parser behavior

OR

- **W1, W3, W4:** Work on ISOLATED subsystems
  - Emitter parity (doesn't depend on parser)
  - Documentation
  - Build system improvements

**DO NOT:** Work on type checking features until parser TS1005/TS1109 fixed

Files: `wasm/src/parser/thin_parser.rs`, all parse functions

### Worker 3: Property Access (TS2339) - 292 false positives [BLOCKED]
**Problem:** "Property 'X' does not exist on type 'Y'" when it does

Root Causes:
1. Type narrowing not applied correctly
2. Index signatures not considered
3. Interface merging incomplete
4. Prototype chain not followed

Files: `thin_checker.rs` - property access checking

### Worker 4: Overload Matching (TS2769) - 125 false positives [⚠️ BLOCKED BY PARSER]

**BLOCKED:** Overload testing requires valid AST. Parser noise prevents validation

**Alternative Work:**
- Assist W2 with parser regression tests
- Work on Emitter parity features
- Document overload resolution architecture

**Original Problem:** "No overload matches this call" when one should

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
- Last Update: 2026-01-11
- Conformance: **30.8% exact match** (+7.5pp from 23.3%)
- Build: Passing

### Worker Assignments (New Phase)
| Worker | Assignment | Priority |
|--------|------------|----------|
| W1 | TS2339 - Property Narrowing + TS2304 edge cases | HIGH |
| W2 | TS2355 - Return Analysis (control flow) | MEDIUM |
| W3 | TS2339 - Index Signature handling | HIGH |
| W4 | TS2322 - False Positive Reduction | MEDIUM |
| W5 | TS2403 - Subsequent Variable Declarations | MEDIUM |

### Before Starting Any Task
**IMPORTANT:** Workers must consult Gemini before starting work:
```bash
./scripts/ask-gemini.mjs "I need to implement <your task>. What's the best approach?"
```

### Strategy
Focus on reducing Extra Errors (false positives) - we report errors TSC doesn't.
Each worker owns one error code category and works independently.
