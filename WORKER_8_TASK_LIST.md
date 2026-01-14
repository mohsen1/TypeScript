# Worker-8 Task List

**Squad:** CFA & Stability (Critical Path)
**Branch:** `worker-8`
**EM:** EM-2
*Assigned: 2025-01-14*

---

## Priority Mission

Fix Class Property Initialization and add Recursion Guards. **Target: Reduce TS2564 missing errors from 413 to <20.**

---

## Assigned Tasks

### 1. Implement Recursion Guards (TS2589)
**Priority:** P0 - Critical (Stability)
**File:** `wasm/src/solver/` or `wasm/src/checker/`

**Issue:** `types/typeRelationships/recursiveTypes` test cases cause stack overflow panics. The compiler crashes instead of emitting TS2589 ("Type instantiation is excessively deep").

**Root Cause:** No recursion depth tracking in `solve_subtype` or `check_expression`. Infinite recursion occurs with mutually recursive types.

**Solution:**
1. Add `recursion_depth: u32` counter to solver context
2. Increment counter in `solve_subtype` before recursive calls
3. Return error type when limit exceeded (default: 100 levels)
4. Emit TS2589 at the call site
5. Add same guard to `check_expression` for expression recursion

**Code Locations:**
- `wasm/src/solver/mod.rs` - Main solver implementation
- `wasm/src/checker/thin_checker.rs` - Expression checking

**Success Criteria:**
- No stack overflow crashes
- TS2589 emitted for deep recursion
- Tests pass without panic

**Status:** ⏳ TODO

---

### 2. Fix Class Property Initialization (TS2564)
**Priority:** P1 - High Impact
**File:** `wasm/src/checker/`

**Issue:** TS2564 ("Property 'x' has no initializer and is not definitely assigned in the constructor") is the #1 missing error with 413 occurrences.

**Root Cause:** The `strictPropertyInitialization` check is not implemented in `thin_checker.rs`.

**Solution:**
1. Implement control flow analysis for class constructors
2. Track property assignments in all constructor code paths
3. Emit TS2564 for properties without:
   - Default initializer (`x: number = 5`)
   - Assignment in all constructor branches
   - Definite assignment assertion (`x!`)
4. Handle `declare` properties (always allowed)
5. Handle abstract classes (skip check)

**Code Locations:**
- `wasm/src/checker/thin_checker.rs` - Main checker
- `wasm/src/checker/control_flow.rs` - CFA infrastructure (exists)
- `wasm/src/checker/flow_analyzer.rs` - Flow analysis (exists)

**Test Cases:**
```typescript
class A {
    x: number;  // ERROR: TS2564
}

class B {
    y: number;  // OK: has initializer
    constructor() {
        this.y = 5;
    }
}

class C {
    z!: number;  // OK: definite assignment assertion
}
```

**Success Criteria:**
- Detect missing property initialization
- Handle all code paths in constructor
- Reduce TS2564 missing errors from 413 to <20

**Status:** ⏳ TODO

---

### 3. Enable Strict Property Initialization in Tests
**Priority:** P2
**File:** `wasm/src/cli/` or test runner

**Issue:** The `strictPropertyInitialization` compiler option may not be enabled in test runs.

**Solution:**
1. Verify test runner enables `strictPropertyInitialization`
2. Update compiler options if needed
3. Re-run conformance tests after Task 2 implementation

**Code Locations:**
- `wasm/src/cli/mod.rs`
- Test configuration files

**Status:** ⏳ TODO

---

## Implementation Plan

### Phase 1: Recursion Guards (Task 1)
1. Read `wasm/src/solver/mod.rs` to understand solver structure
2. Add recursion depth tracking to solver context
3. Implement depth limit check in `solve_subtype`
4. Add TS2589 emission
5. Add unit tests for recursive types
6. Verify no crashes on `recursiveTypes` tests

### Phase 2: Class Property Checks (Task 2)
1. Study existing CFA infrastructure in `control_flow.rs`
2. Understand property declaration flow
3. Implement property initialization tracking
4. Add TS2564 emission logic
5. Handle edge cases (declare, abstract, definite assignment)
6. Write comprehensive tests

### Phase 3: Validation (Task 3)
1. Enable strictPropertyInitialization in tests
2. Run full conformance suite
3. Measure TS2564 reduction
4. Verify TS2589 emissions prevent crashes

---

## Success Metrics

### Before Implementation
- **TS2564 Missing:** 413 errors
- **Stack Overflow Crashes:** 2+ test failures
- **Stability:** Crashes on recursive types

### After Implementation (Target)
- **TS2564 Missing:** <20 errors (95%+ reduction)
- **Stack Overflow Crashes:** 0
- **Stability:** All tests complete with proper errors

---

## Notes

**Dependencies:**
- CFA infrastructure already exists in `wasm/src/checker/control_flow.rs`
- Solver code in `wasm/src/solver/` needs recursion guards

**Synergies:**
- Recursion guards enable testing of more complex type relationships
- TS2564 implementation validates CFA correctness

**Risks:**
- CFA may need enhancements for all code paths
- Constructor control flow can be complex (try/catch, early returns)

---

## Next Steps

1. Start with Task 1 (Recursion Guards) - straightforward, stabilizes tests
2. Move to Task 2 (TS2564) - uses existing CFA infrastructure
3. Complete Task 3 (Validation) - measure and verify

**When complete:** Push to `worker-8` branch and notify EM-2 for review.
