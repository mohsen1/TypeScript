# Worker-11 Task List

**Squad:** Semantics (Implicit Any Detection)
**Branch:** `worker-11`
**EM:** EM-3
*Assigned: 2026-01-14*

---

## Priority Mission

Fix TS7006 (Implicit Any) detection. **Target: Reduce from 357 to <10 missing errors.**

---

## Assigned Tasks

### 1. Detect Implicit Any in Parameters
**Priority:** P0 - High
**Files:** `wasm/src/solver/` and `wasm/src/checker/`

**Problem:** 357 missing TS7006 errors. Parameter type inference is too permissive.

**Tasks:**
1. Detect implicit any types in function parameters
2. Check function signatures for missing types
3. Apply strict type checking rules
4. Work配合 with worker-9 (UNKNOWN defaults)

### 2. Implicit Any in Other Contexts
**Priority:** P1
**Files:** `wasm/src/solver/` and `wasm/src/checker/`

**Tasks:**
1. Detect implicit any in variable declarations
2. Detect implicit any in return types
3. Detect implicit any in object properties
4. Handle `noImplicitAny` compiler option

### 3. Test Coverage
**Priority:** P2

**Test Cases:**
- Function parameters without types
- Object properties without types
- Variables inferred as any
- `noImplicitAny` enabled/disabled
- Properly typed code (no false positives)

---

## Success Criteria
- [ ] Implicit any parameters trigger TS7006
- [ ] No false positives on properly typed code
- [ ] Metrics showing reduction in missing TS7006
- [ ] Reduce missing TS7006 from 357 to <10

---

## Status
**Status:** 🔄 IN PROGRESS
**Assigned:** 2026-01-14
