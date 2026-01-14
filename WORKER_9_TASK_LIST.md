# Worker-9 Task List

**Squad:** Stability
**Branch:** `worker-9`
**EM:** EM-3
*Assigned: 2026-01-14*

---

## Priority Mission

Fix recursion guard crashes. **Target: Zero stack overflow crashes in conformance tests.**

---

## Assigned Tasks

### 1. Add Recursion Depth Counter
**Priority:** P0 - Critical
**Files:** `wasm/src/solver/`, `wasm/src/checker/`

**Issue:** The test `types/typeRelationships/recursiveTypes` causes a panic due to unbounded recursion.

**Tasks:**
1. Add `recursion_depth: u32` counter to relevant solver structs
2. Increment counter on recursive calls in `solve_subtype`
3. Increment counter on recursive calls in `check_expression`
4. Set limit to 100 (standard TypeScript limit)

### 2. Return TS2589 Error at Limit
**Priority:** P0 - Critical
**Files:** `wasm/src/solver/`, `wasm/src/checker/`

**Tasks:**
1. When recursion limit is hit, return error instead of recursing
2. Emit TS2589: "Type instantiation is excessively deep and possibly infinite."
3. Ensure error propagates correctly through call chain

### 3. Test Recursive Type Patterns
**Priority:** P1
**Files:** Conformance tests

**Test Cases:**
- Direct recursion: `type T = { x: T }`
- Mutual recursion: `type A = B; type B = A`
- Generic recursion: `type Box<T> = { value: Box<T> }`

---

## Success Criteria
- [ ] No stack overflow crashes on conformance tests
- [ ] TS2589 errors emitted for excessively deep types
- [ ] All `recursiveTypes` tests pass with errors (not crashes)

---

## Status
**Status:** 🔄 IN PROGRESS
**Assigned:** 2026-01-14
