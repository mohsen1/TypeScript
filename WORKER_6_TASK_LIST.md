# Worker 6 Task List (Rust/WASM)

**Maintained by:** EM-2
**Target Branch:** rust
**Worker Directory:** /tmp/orchestrator-workspace/worktrees/worker-6

---

## Task 1: Add Recursion Guards to Prevent Stack Overflow

**Priority:** 🔴 STABILITY (Project Zang Priority #5)
**Status:** ⏳ TODO
**Assigned:** 2025-01-14

### Context
The TypeScript compiler tests are currently causing **2 crashes** (stack overflows) in the WASM compiler, specifically in the `types/typeRelationships/recursiveTypes` test cases. When the Rust WASM process panics due to stack overflow, the entire test run fails.

### Problem
Recursive type checking can cause infinite recursion in:
- `solve_subtype` - Subtype checking can loop on circular type references
- `check_expression` - Expression checking can recurse deeply

Current behavior: **Panic/crash** when stack overflows
Desired behavior: Return error TS2589 ("Type instantiation is excessively deep and possibly infinite")

### Task Requirements

1. **Add recursion depth counter to `solve_subtype`**:
   - Location: Likely `wasm/src/checker/` or `wasm/src/solver/` (need to locate)
   - Add a `recursion_depth: u32` counter parameter
   - Increment on each recursive call
   - When `recursion_depth > 100`:
     - Return early with a synthetic error state
     - Emit diagnostic TS2589: "Type instantiation is excessively deep and possibly infinite"

2. **Add recursion depth counter to `check_expression`**:
   - Same approach as above
   - Same limit (100 levels)

3. **Verify the fix**:
   - Run conformance tests: `types/typeRelationships/recursiveTypes`
   - Confirm no more panics/crashes
   - Confirm TS2589 errors are emitted where appropriate

### Success Criteria
- [ ] No stack overflow panics in `types/typeRelationships/recursiveTypes` tests
- [ ] TS2589 errors emitted for excessively deep recursion
- [ ] All existing non-recursive tests still pass (no regression)

### Notes
- The recursion limit of 100 is a suggestion; adjust if TypeScript uses a different value
- Do NOT remove the recursion - just guard it with an error
- The goal is "fail gracefully" not "infinite loops"

---

## Completed Tasks
*None yet*

---

**Next Steps:**
1. Search for `solve_subtype` and `check_expression` in wasm/src/
2. Understand current implementation
3. Add recursion guards
4. Test with recursiveTypes conformance tests
5. Commit with message: "Add recursion guards to solve_subtype and check_expression"
6. Push to worker-6 branch and STOP
