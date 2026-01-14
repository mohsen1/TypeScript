# Worker 6 Task List (Rust/WASM)

**Maintained by:** EM-2
**Target Branch:** rust
**Worker Directory:** /tmp/orchestrator-workspace/worktrees/worker-6

---

## Completed Tasks

### Task 1: Add Recursion Guards to Prevent Stack Overflow ✅

**Priority:** 🔴 STABILITY (Project Zang Priority #5)
**Status:** ✅ COMPLETED
**Assigned:** 2025-01-14
**Completed:** 2025-01-14

**Summary:** Added TS2589 diagnostic for excessive recursion depth in SubtypeChecker. The depth tracking already existed; added diagnostic emission when limit (100) is exceeded.

**Changes:**
- `wasm/src/checker/types/diagnostics.rs` - Added TS2589 code/message
- `wasm/src/solver/subtype.rs` - Added depth_exceeded flag
- `wasm/src/thin_checker.rs` - Added diagnostic emission logic

---

## Current Task

### Task 2: Fix Class Property Initialization (TS2564)

**Priority:** 🟡 TACTICAL (Project Zang Priority #4)
**Status:** ⏳ IN PROGRESS
**Assigned:** 2025-01-14

#### Context
TS2564 ("Property 'x' has no initializer and is not definitely assigned in the constructor.") is the **#1 missing error** with **413 occurrences**.

TypeScript's `strictPropertyInitialization` check ensures that class properties are either:
1. Declared with an initializer, OR
2. Definitely assigned in the constructor

This check is **not implemented** in the WASM compiler, which is why we're missing 413 errors.

#### Problem
The WASM compiler is not running the `strictPropertyInitialization` check, so properties without initializers are not flagged even when they should be.

#### Task Requirements

1. **Understand the TypeScript behavior:**
   - Only applies when `strictPropertyInitialization` compiler option is enabled
   - Checks non-optional instance properties of classes
   - Excludes: `static` properties, `readonly` properties with initializers, properties marked with `!` (definite assignment assertion)

2. **Implement the check in `wasm/src/checker/thin_checker.rs`:**
   - Find where class declarations are checked
   - For each non-optional instance property without an initializer:
     - Track whether it's assigned in the constructor
     - If not assigned AND has no initializer, emit TS2564

3. **Implementation hints:**
   - Look for existing class declaration checking code
   - Use the flow analyzer (if available) to track definite assignment
   - The context has `ctx.strict_property_initialization` flag
   - Check for the definite assignment assertion operator (`!`)

4. **Verify the fix:**
   - Run conformance tests and count TS2564 errors
   - Should see ~413 new TS2564 errors appear
   - Verify errors match tsc's output

#### Success Criteria
- [ ] TS2564 diagnostic code defined in diagnostics.rs (if not already)
- [ ] Check implemented in thin_checker.rs
- [ ] Respects `strict_property_initialization` flag
- [ ] Correctly excludes static, readonly with initializers, and `!`-marked properties
- [ ] ~413 new TS2564 errors appear in conformance tests
- [ ] No regressions (no extra TS2564 where tsc doesn't emit)

#### Reference Test Files
Look for existing test cases that should emit TS2564:
```
tests/cases/conformance/strictPropertyInitialization/
tests/baselines/reference/strictPropertyInitialization*.errors.txt
```

#### Notes
- This is a **high-ROI task** - one fix catches 413 missing errors
- The check runs **after** type checking (in the checker phase, not solver)
- May need to leverage flow analysis or constructor body scanning
- The `!` definite assignment assertion should suppress the error

---

## Pending Tasks
*None - awaiting EM-2 assignment*
