# Worker-3 Task List

## ✅ COMPLETED: Parser Noise Fix (TS1005 & TS1109)
**Priority:** 🔴 CRITICAL (Highest Priority)
**Owner:** worker-3
**Branch:** worker-3
**Status:** ✅ COMPLETE
**Assigned:** 2026-01-14
**Completed:** 2026-01-15

### Results
- **TS1005**: 24 extra errors (down from 439) - **95% reduction** ✅
- **TS1109**: 0 extra errors (down from 262) - **100% reduction** ✅
- **Combined**: 24 extra errors (down from 701) - **97% reduction** ✅
- **Target**: <40 combined errors - **MET** ✅

### Changes Made
1. **Fixed await expression parsing** - Now respects async context (only parses as await expression when in async function, otherwise parses as identifier)
2. **Added AwaitKeyword/YieldKeyword to parse_primary_expression** - Allows these keywords to be parsed as identifiers in expression contexts
3. **Enhanced is_array_element_start** - Added spread operator, this/super support, and proper fallback
4. **Removed duplicate function** - Cleaned up duplicate `is_array_element_start` definition

### Remaining Edge Cases
The 24 remaining TS1005 errors are all the same edge case: `function f(await = await) {}` (non-async function with `await` as parameter name with default value). This is valid TypeScript but requires additional context-aware handling in parameter declarations.

---

## 🔴 CURRENT TASK: Invert Solver Defaults (Stop being "Nice")
**Priority:** 🔴 CRITICAL (Strategic)
**Owner:** worker-3
**Branch:** worker-3
**Status:** 🟡 IN PROGRESS
**Assigned:** 2026-01-15

---

## Task Description

**Problem:** Missing 2,961 errors (60% of all errors). We are missing 184 `TS2322` (Type Mismatch) and 357 `TS7006` (Implicit Any) errors.

**Root Cause:** The compiler is "optimistic"—when it encounters an unknown type or a resolution failure, it returns `TypeId::ANY`. This hides type errors instead of exposing them.

**Target:** Change default from `ANY` to `UNKNOWN` to expose hidden type errors

---

## Implementation Plan

### Phase 1: Understand the Current Behavior

1. **Understand the current behavior:**
   - Search for all places where `TypeId::ANY` is returned as a default
   - Understand the difference between `TypeId::ANY`, `TypeId::UNKNOWN`, and `TypeId::ERROR`
   - Read `wasm/specs/SOLVER.md` for solver architecture

2. **Find the return points:**
   ```bash
   cd /tmp/orchestrator-workspace/worktrees/worker-3/wasm/src
   grep -rn "TypeId::UNKNOWN\|TypeId::ANY\|TypeId::ERROR" solver/
   grep -rn "return.*ANY" solver/
   ```

3. **Run baseline conformance tests:**
   ```bash
   cd /tmp/orchestrator-workspace/worktrees/worker-3/wasm
   ./differential-test/run-conformance.sh --max=100
   ```
   Record current TS2322 and TS7006 counts.

4. **Study TypeScript's error handling:**
   - When does tsc report "Implicit Any" vs "Unknown" vs "Error"?
   - What's the semantic difference?

---

### Phase 2: Change Defaults to UNKNOWN

**Goal:** Return `TypeId::UNKNOWN` or `TypeId::ERROR` instead of `TypeId::ANY` when a symbol cannot be resolved or a type operation fails.

**Key Files to Modify:**

1. **wasm/src/solver/operations.rs**
   - Search for `resolve_named_type` failing returns
   - Property access resolution failures
   - Method call resolution failures

2. **wasm/src/solver/constraints.rs** (if it exists)
   - Constraint solving failures
   - Type inference failures

3. **wasm/src/checker/thin_checker.rs**
   - Expression type checking failures
   - Variable declaration type inference failures

**Pattern to Find and Fix:**
```rust
// BEFORE (optimistic - hides errors):
fn some_resolution(&mut self) -> TypeId {
    match self.try_resolve() {
        Some(t) => t,
        None => TypeId::ANY,  // ❌ Too permissive
    }
}

// AFTER (strict - exposes errors):
fn some_resolution(&mut self) -> TypeId {
    match self.try_resolve() {
        Some(t) => t,
        None => TypeId::UNKNOWN,  // ✅ Exposes the problem
    }
}
```

### Phase 3: Handle the Error Spike

**Expected Result:** Massive spike in "Extra Errors" after the change.

**This is GOOD because:**
- It exposes exactly where our logic is failing
- It replaces hidden errors with visible diagnostics
- It shows us what we need to fix next

**Validation Steps:**
1. Run conformance tests after each major change
2. Check that the "Extra Errors" increase is in TS2322/TS7006 (expected)
3. Check for regressions in previously passing tests
4. Document which errors are "expected" vs "real bugs"

---

## Success Criteria

- [ ] All `TypeId::ANY` defaults changed to `TypeId::UNKNOWN` or `TypeId::ERROR`
- [ ] TS2322 (Type Mismatch) errors increase from 184 missing to >100 extra
- [ ] TS7006 (Implicit Any) errors increase from 357 missing to >200 extra
- [ ] No regressions in tests that were previously passing
- [ ] Baseline established for next round of fixes
- [ ] Code comments added explaining when to return UNKNOWN vs ERROR vs ANY

---

## Workflow

1. **Sync with latest rust:**
   ```bash
   git fetch origin
   git rebase origin/rust
   ```

2. **Investigation Phase:**
   - Find all locations returning `TypeId::ANY` as default
   - Understand semantic differences between ANY/UNKNOWN/ERROR
   - Run baseline conformance tests
   - Document current behavior

3. **Implementation Phase:**
   - Change defaults from ANY to UNKNOWN/ERROR
   - Run tests after each change
   - Document error increases
   - Fix any obvious regressions

4. **Validation:**
   - Run full conformance test suite
   - Verify TS2322/TS7006 errors increased as expected
   - Check for unexpected regressions
   - Document findings

5. **Commit and Push:**
   ```bash
   git add -A
   git commit -m "feat(solver): invert defaults from ANY to UNKNOWN"
   git push origin worker-3 --force
   ```

6. **STOP** - Wait for EM-1 review

---

## Deliverables

1. All solver/checker locations returning ANY as default changed to UNKNOWN/ERROR
2. Baseline test results showing error increases
3. Documentation of expected vs unexpected errors
4. Updated task list with "Complete" status
5. Conformance test report showing the change

---

## Known Risks

1. **Error Spike:** Expect 500+ new extra errors
   - **Mitigation:** Document which are expected (TS2322/TS7006 increases)

2. **Test Failures:** Some tests may fail due to exposed errors
   - **Mitigation:** Distinguish between "test was wrong" vs "real bug exposed"

3. **Performance:** More errors = slower type checking
   - **Mitigation:** Profile before/after if performance degrades

---

## Status

- **Current Task:** Invert Solver Defaults (Stop being "Nice")
- **Phase:** Investigation (Phase 1)
- **Last Updated:** 2026-01-15
- **Ready to Start:** ✅ YES
