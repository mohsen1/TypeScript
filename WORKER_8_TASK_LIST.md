# Worker-8 Task List

**Squad:** Semantics (Solver Strictness)
**Branch:** `worker-8`
**EM:** EM-2
*Assigned: 2025-01-14*
*Updated: 2025-01-14*

---

## Previous Tasks - Complete ✅

### 1. Recursion Guards (TS2589) ✅
- Added depth tracking to solver
- Emit TS2589 when recursion exceeds 100 levels
- No more stack overflow crashes

### 2. Class Property Initialization (TS2564) ✅
- Already implemented in codebase
- Verified working via conformance tests
- Not in missing errors list

---

## Priority Mission

**Invert Solver Defaults - Stop Being "Nice"**

Currently the compiler returns `TypeId::ANY` when it encounters unknown types or resolution failures. This suppresses legitimate errors and hides bugs.

**Goal:** Change defaults to `UNKNOWN` or `ERROR` to expose failing logic.

**Target Impact:** Reduce missing errors from 2961 (60%) to <15%

---

## New Assigned Tasks

### 1. Invert Function Return Type Defaults
**Priority:** P0 - Strategic
**File:** `wasm/src/checker/` and `wasm/src/solver/`

**Issue:** Functions without return type annotations default to `ANY` instead of `UNKNOWN`. This suppresses type mismatch errors.

**Solution:**
1. Find all places where function return types default to `ANY`
2. Change to return `UNKNOWN` or infer from function body
3. Locations to check:
   - `check_function_declaration` in `thin_checker.rs`
   - `check_function_expression` in `thin_checker.rs`
   - `check_arrow_function` in `thin_checker.rs`
   - Solver's `get_function_return_type` methods

**Test Case:**
```typescript
// Should error: Type 'string' is not assignable to 'number'
function foo(): number {
    return "hello";  // Currently silent, should emit TS2322
}
```

**Success Criteria:**
- Function return types default to `UNKNOWN` when not annotated
- Type mismatches in return statements are caught
- No regression in valid code

**Status:** ⏳ TODO

---

### 2. Invert Variable Declaration Defaults
**Priority:** P1 - High Impact
**File:** `wasm/src/checker/`

**Issue:** Variables without type annotations default to `ANY` from initializer, suppressing TS7006 (implicit any) and type mismatch errors.

**Solution:**
1. Change `get_type_of_variable_declaration` to infer more strictly
2. If initializer exists and type annotation missing:
   - Use inferred type (not widened to `ANY`)
   - Emit TS7006 if inferred type is `ANY` or `UNKNOWN`
3. Ensure `strictNullChecks` is respected

**Test Case:**
```typescript
// @strict: true
let x = 5;
x = "string";  // Should error: Type 'string' is not assignable to 'number'
```

**Success Criteria:**
- Variables infer narrower types from initializers
- Re-assignment with wrong type emits TS2322
- TS7006 emitted when type cannot be inferred

**Status:** ⏳ TODO

---

### 3. Invert Expression Statement Defaults
**Priority:** P2
**File:** `wasm/src/checker/`

**Issue:** Expression failures in type checking often return `ANY`, cascading into silence.

**Solution:**
1. Audit `get_type_of_binary_expression`, `get_type_of_call_expression`, etc.
2. Return `UNKNOWN` or `ERROR` when type checking fails
3. Ensure diagnostic is emitted before returning error type

**Test Case:**
```typescript
function foo() {
    return undefined;
}
const x: number = foo();  // Should error: Type 'undefined' not assignable to 'number'
```

**Success Criteria:**
- Failed type resolution returns `UNKNOWN` not `ANY`
- Errors are emitted for invalid operations
- No silent acceptance of invalid code

**Status:** ⏳ TODO

---

## Implementation Plan

### Phase 1: Analysis (1-2 hours)
1. Search codebase for `TypeId::ANY` returns
2. Identify which are "optimistic" defaults vs intentional
3. Create list of locations to fix

### Phase 2: Function Returns (2-3 hours)
1. Update `get_function_return_type` in solver
2. Update function checking in `thin_checker.rs`
3. Add tests for return type mismatches

### Phase 3: Variable Declarations (2-3 hours)
1. Update `get_type_of_variable_declaration`
2. Implement proper type inference from initializers
3. Test with strictNullChecks enabled

### Phase 4: Expressions (2-3 hours)
1. Audit and fix expression type resolution
2. Ensure error propagation works correctly
3. Test complex expressions

### Phase 5: Validation (1 hour)
1. Run conformance tests (expect extra errors - this is good!)
2. Verify no crashes
3. Check that new errors are legitimate (not false positives)

---

## Success Metrics

### Before Implementation
- **Missing Errors:** 2961 (60%)
- **TS2322 Missing:** 184 occurrences
- **TS7006 Missing:** 357 occurrences

### After Implementation (Target)
- **Missing Errors:** <15% (from 60%)
- **TS2322 Detected:** Most cases caught
- **TS7006 Detected:** Most cases caught
- **Extra Errors:** Will increase (this is expected and good)

---

## Notes

**Expected Outcome:**
- **"Extra Errors" WILL increase** - This is intentional and good!
- We will see where the compiler was being too permissive
- Some errors may be false positives that need refinement

**Risks:**
- May need to adjust error thresholds
- Some valid code might incorrectly error (needs tuning)
- Performance impact from more error checking

**Dependencies:**
- Solver changes require careful testing
- Must coordinate with Semantics Squad
- May reveal issues in other components

**Synergies:**
- Exposes bugs in other parts of the compiler
- Makes the type checker more strict and correct
- Aligns with TypeScript's actual behavior

---

## Next Steps

1. **Phase 1:** Analyze codebase for `TypeId::ANY` usage
2. **Phase 2:** Fix function return defaults
3. **Phase 3:** Fix variable declaration defaults
4. **Phase 4:** Fix expression defaults
5. **Phase 5:** Run tests and validate

**When complete:** Push to `worker-8` branch and notify EM-2 for review.
