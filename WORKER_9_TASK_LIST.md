# Worker-9 Task List

**Squad:** Semantics (Solver Defaults)
**Branch:** `worker-9`
**EM:** EM-3
*Assigned: 2026-01-14*

---

## Priority Mission

Invert solver defaults - change from `ANY` to `UNKNOWN`/`ERROR`. **Target: Expose hidden errors instead of hiding them.**

---

## Assigned Tasks

### 1. Change Solver Default Return Type
**Priority:** P0 - Critical
**File:** `wasm/src/solver/`

**Problem:** 2961 missing errors (60%). Missing 184 TS2322 (Type Mismatch) and 357 TS7006 (Implicit Any). Our compiler is "optimistic" - when it encounters an unknown type or resolution failure, it returns `TypeId::ANY`. This hides real errors.

**Tasks:**
1. Change solver to return `TypeId::UNKNOWN` or `TypeId::ERROR` instead of `TypeId::ANY`
2. Apply to all symbol resolution failures
3. Apply to all type operation failures
4. Update all downstream code to handle UNKNOWN/ERROR properly

### 2. Update Downstream Error Handling
**Priority:** P0 - Critical
**Files:** `wasm/src/solver/`, `wasm/src/checker/`

**Tasks:**
1. Ensure type relationships work with UNKNOWN/ERROR types
2. Update constraint solving to handle error types
3. Verify subtyping checks propagate errors correctly
4. Add proper error messages for UNKNOWN types

### 3. Test and Validate
**Priority:** P1

**Tasks:**
1. Run conformance tests - expect spike in "Extra Errors" (this is GOOD)
2. Document which hidden errors are now exposed
3. Verify no crashes from error propagation
4. Generate before/after metrics

---

## Success Criteria
- [ ] Unknown types return UNKNOWN, not ANY
- [ ] Type mismatches are detected, not hidden
- [ ] Metrics showing increase in caught errors
- [ ] Documented analysis of new "Extra Errors"

---

## Status
**Status:** 🔄 IN PROGRESS
**Assigned:** 2026-01-14
