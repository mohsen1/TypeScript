# Worker-10 Task List

**Squad:** Semantics (Property Initialization)
**Branch:** `worker-10`
**EM:** EM-3
*Assigned: 2026-01-14*

---

## Priority Mission

Implement `strictPropertyInitialization` check (TS2564). **Target: Reduce from 413 to <20 missing errors.**

---

## Assigned Tasks

### 1. Implement Strict Property Initialization Check
**Priority:** P0 - Critical
**File:** `wasm/src/checker/thin_checker.rs`

**Problem:** TS2564 is the #1 missing error (413 occurrences). "Property 'x' has no initializer..." means we aren't running this check.

**Tasks:**
1. Detect class properties without initializers
2. Check constructor assigns all properties
3. Handle definite assignment assertions (`!`)
4. Handle optional properties (`?`)
5. Handle `declare` properties correctly

### 2. Control Flow Analysis for Property Assignment
**Priority:** P1
**File:** `wasm/src/checker/`

**Tasks:**
1. Track property assignments in constructor
2. Handle conditional assignments
3. Handle property assignments in called functions
4. Handle assignment in all code paths

### 3. Test Edge Cases
**Priority:** P2

**Test Cases:**
- Properties with initializers
- Properties assigned in constructor
- Properties with definite assignment assertion
- Optional properties
- Abstract class properties
- `declare` properties

---

## Success Criteria
- [ ] TS2564 errors detected for non-initialized properties
- [ ] No false positives on constructor-initialized props
- [ ] Handles `declare` properties correctly
- [ ] Handles definite assignment assertion correctly
- [ ] Reduce missing TS2564 from 413 to <20

---

## Status
**Status:** 🔄 IN PROGRESS
**Assigned:** 2026-01-14
