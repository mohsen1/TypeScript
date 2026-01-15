# EM-3 TASK LIST

## Team: em-team-3
## Manager: EM-3
## Base Branch: rust
## Team Branch: em-team-3

---

## Mission Statement

EM-3 is responsible for **type checker accuracy**. Your team ensures that:
1. Type assignability is correctly enforced (TS2322)
2. `this` type handling is accurate (TS2683, TS2571)
3. Generic type constraints are properly checked
4. Object-oriented type checking works correctly (super(), extends, etc.)

**Why this matters:** Type checker correctness is critical for catching real errors while avoiding false positives. Users trust TypeScript to catch type mismatches.

---

## Team Composition

| Worker | Squad | Focus Area | Status | Throughput |
|--------|-------|------------|--------|------------|
| worker-1 | Type Squad | Implicit `this` handling (TS2683) | ✅ Complete | High |
| worker-2 | Type Squad | `super()` call handling (TS2322) | ✅ Complete | High |

**EM Branch:** em-team-3

---

## Completed Work

### Worker 1: TS2683 Implementation ✅
**Commit:** c958fc9cb - "fix: implement TS2683 for implicit this in functions"

**Problem:** When `this` is used inside a regular function (not a method), TypeScript should emit TS2683 ("'this' implicitly has type 'any'") but WASM was typing it as `unknown` and emitting TS2571 instead.

**Fix:** Modified `current_this_type()` in `thin_checker.rs` to detect non-method functions and emit the correct error.

**Impact:** Correct type checking for `this` in regular functions and callbacks.

---

### Worker 2: super() Call Handling ✅
**Commit:** dc7519914 - "fix: add special handling for super() calls in ThinCheckerState"

**Problem:** `super()` calls in constructors weren't properly type-checked, causing false positives or missing errors.

**Fix:** Added special handling for `super()` calls in constructor contexts.

**Impact:** Correct type checking for class inheritance and super() calls.

---

## Team Priorities (Updated 2026-01-15)

### Priority 1: TS2322 Type Assignability Accuracy
**Owner:** worker-1 or worker-2 (new assignment)

**Goal:** Reduce TS2322 extra errors from 548 to <200

**Current State:**
- 548 extra TS2322 errors (mostly from "Invert Solver Defaults" fix)
- Some are legitimate errors that were previously hidden
- Some may be false positives from overly strict type checking

**Action Items:**
1. Categorize TS2322 errors: legitimate vs false positive
2. Fix assignability logic for false positives
3. Ensure generic type constraints are handled correctly
4. Handle union/intersection type assignability edge cases

**Target Metrics:**
| Error Code | Current | Target |
|------------|---------|--------|
| TS2322 extra | ~548 | <200 |

**Key Files:**
- `wasm/src/checker/type_compatibility.rs` (if exists)
- `wasm/src/thin_checker.rs` - type comparison functions

---

### Priority 2: TS2571 Over-reporting
**Owner:** worker-1 or worker-2 (new assignment)

**Goal:** Eliminate TS2571 false positives (should be TS2683 instead)

**Current State:**
- TS2571: "Object is of type 'unknown'"
- TS2683 missing: "'this' implicitly has type 'any'"
- These are the same underlying issue in different contexts

**Action Items:**
1. Review all TS2571 emissions
2. Determine when they should be TS2683 instead
3. Fix type inference for `this` in non-class methods
4. Test with arrow functions, callbacks, event handlers

**Target Metrics:**
| Error Code | Current | Target |
|------------|---------|--------|
| TS2571 extra | Unknown | <50 |
| TS2683 missing | Unknown | Fill gaps |

---

### Priority 3: Generic Type Constraints
**Owner:** worker-1 or worker-2 (future assignment)

**Goal:** Correct handling of generic type parameters and constraints

**Current State:**
- Generic types may not be properly checked
- Type parameter constraints (extends) may not be enforced
- Variance and covariance may have issues

**Action Items:**
1. Review generic type instantiation
2. Ensure type parameter constraints are checked
3. Handle generic type defaults
4. Test with complex generic scenarios

---

## Conformance Test Baseline (2026-01-15)

Current baseline from rust branch (commit 05236939f):

**Top Type Checker Errors:**
- TS2322: ~548 extra (Type mismatch - intentional regression from solver fix)
- TS2571: Unknown count (Object is of type 'unknown')
- TS2683: Missing count (implicit this in functions)

**Target:** Reduce type checker noise by 70% while maintaining accuracy

---

## Workflow

### For EM-3:
1. **Daily sync**: `git pull origin rust` → merge to em-team-3
2. **Review worker branches**: Check commits, test results
3. **Merge locally**: `git merge worker-X` into em-team-3
4. **Run validation**: `./wasm/differential-test/run-conformance.sh --max=500 --workers=4`
5. **Push to director**: Only when stable and validated

### For Workers:
1. Create branch from em-team-3
2. Work on assigned task ONLY
3. Commit frequently with `[wasm] checker: <description>`
4. Push to worker-X branch
5. Update task list with status
6. Notify EM-3 when ready for merge

---

## Merge Readiness Status (2026-01-15)

| Worker | Status | Notes |
|--------|--------|-------|
| worker-1 | 🟢 Complete | TS2683 fix complete (c958fc9cb), needs new task |
| worker-2 | 🟢 Complete | super() fix complete (dc7519914), needs new task |

---

## Escalation Path

1. Worker commits → worker-X branch
2. EM-3 merges to em-team-3 → validates
3. EM-3 escalates to Director when stable
4. Director reviews → merges to rust

**Do NOT push directly to rust.**

---

## Next Actions for EM-3

1. ✅ Create em-team-3 branch
2. ✅ Create EM_3_TASKS.md
3. 🔄 Assign Priority 1 (TS2322 accuracy) to worker-1 or worker-2
4. 🔄 Assign Priority 2 (TS2571 over-reporting) to remaining worker
5. 📅 Run conformance tests to establish baseline
6. 📋 Track type checker error counts

---

## Notes

- All work must stay in `wasm/` directory
- TypeScript source files (`src/compiler/`) are READ-ONLY
- Run `./wasm/test.sh` for Rust tests (Docker-only)
- Use `./scripts/ask-gemini.mjs` before coding (if applicable)
- Target: 95%+ exact match before production
- **Focus:** Type checker accuracy, not parser or binder work

---

## Team Size and Capacity

**Current Workers:** 2 (worker-1, worker-2)
**Capacity:** Can accept up to 2 more workers (limit is 4)

**Future Considerations:**
- If worker-3 or worker-4 are reassigned from EM-1, they should be assigned:
  - Clear, specific tasks
  - Close mentorship from high-throughput workers
  - Regular progress checkpoints

---

## Success Metrics

| Metric | Current | Target (EM-3) |
|--------|---------|---------------|
| TS2322 extra errors | ~548 | <200 |
| TS2571 extra errors | Unknown | <50 |
| TS2683 missing errors | Unknown | Fill gaps |
| Exact Match Rate | ~30% | 40%+ |

**Overall EM-3 Goal:** Improve type checker accuracy by 70%, increase exact match rate by 10%
