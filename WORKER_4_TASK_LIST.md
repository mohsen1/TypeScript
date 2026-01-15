# Worker-4 Task List

## ✅ COMPLETED: Flow Recording (2026-01-15)
- **Status:** Complete and merged to em-team-1
- **Summary:** Fixed flow recording for statements and identifiers
- **Test Results:** 53/54 control_flow tests passing (up from 44/54)
- **Commits Merged:**
  - a163cbed8c9 [wasm] binder: add flow recording for statements and identifiers
  - 4c2544eb316 [wasm] flow: fix literal type narrowing in assignments
  - df0a397b83a [wasm] tests: fix application expansion test setup

---

## 🔄 IN PROGRESS: Application Expansion Tests (2026-01-15)
- **Status:** In Progress (29/34 tests passing)
- **Objective:** Fix remaining failing application expansion tests in the type solver
- **Priority:** 🟡 MEDIUM (can be paused for critical task)

## Changes Made So Far:
1. **Fixed test setup** (`evaluate_tests.rs`):
   - Changed `env.insert()` to `env.insert_with_params()` to register type parameters
   - Added `.clone()` when creating TypeParameter types to allow reuse

2. **Added default type parameter support** (`instantiate.rs`):
   - Modified `TypeSubstitution::from_args()` to handle default type parameters
   - When fewer type arguments than parameters, defaults are now used

## Test Results:
- **Before:** 20/34 application expansion tests passing
- **After:** 29/34 application expansion tests passing 🎉

## Remaining Work (5 tests):
The following tests still fail and likely need the same fixes:
1. `test_application_ref_expansion_with_any_arg`
2. `test_application_ref_expansion_with_unknown_arg`
3. `test_application_ref_expansion_with_union_arg`
4. `test_application_ref_expansion_nested`
5. `test_application_ref_expansion_reducer_function`

## Pattern to Apply:
For each failing test:
1. Add `.clone()` to type parameters when creating TypeParameter types
2. Change `env.insert(SymbolRef(N), body)` to `env.insert_with_params(SymbolRef(N), body, vec![param.clone()])`
3. Update assertions to expect expanded types instead of passing through unchanged

## Files to Modify:
- `wasm/src/solver/evaluate_tests.rs`

## Status
- **Assigned:** 2026-01-15
- **In Progress:** Yes
- **Note:** Can be paused for critical Recursion Guards task

---

## 🎯 NEW CRITICAL ASSIGNMENT: Recursion Guards (Stack Overflow)
**Priority:** 🔴 CRITICAL (STABILITY)
**Assigned:** 2026-01-15
**Owner:** worker-4
**Branch:** worker-4

### Task Description
Fix stack overflow crashes in the type checker by adding recursion depth counters. The recursiveTypes test currently causes 2 stack overflow crashes, blocking all validation work.

### Problem Analysis
From PROJECT_DIRECTION.md baseline:
- **Crashes:** 2 (stack overflow in type checker)
- **Test:** `recursiveTypes` test file triggers crashes
- **Impact:** HIGH - crashes block all conformance testing
- **Root Cause:** Type checker doesn't limit recursion depth when checking recursive type definitions

### Action Items

#### Phase 1: Investigation
- [ ] Read `wasm/specs/WASM_ARCHITECTURE.md` type checker section
- [ ] Locate crash trigger: Run recursiveTypes test to reproduce stack overflow
- [ ] Identify recursive code paths in type checker:
  - Likely in `wasm/src/checker/thin_checker.rs` (type checking)
  - Or `wasm/src/checker/solver.rs` (type solving)
- [ ] Study how TypeScript handles recursion guards

#### Phase 2: Implementation
- [ ] Add recursion depth counter to relevant type checking functions
- [ ] Implement depth limit (start with 100, adjust if needed)
- [ ] Add graceful fallback when limit reached:
  - Return `Any` type or `Unknown` type
  - Or skip checking deeply nested types
- [ ] Add diagnostic/warning when recursion limit hit (optional)

#### Phase 3: Validation
- [ ] Run recursiveTypes test - should complete without crash
- [ ] Run `./wasm/test.sh` (Docker-only!)
- [ ] Run conformance tests: `./wasm/differential-test/run-conformance.sh --all`
- [ ] Verify zero crashes on all tests
- [ ] Check that recursion guards don't break valid recursive types

### Success Metrics
- **Crashes:** Reduce from 2 to 0
- **recursiveTypes test:** Completes without stack overflow
- **Regressions:** No new errors introduced by depth limiting

### Implementation Guidance

**Where to Add Recursion Guards:**

Look for recursive functions in type checker:

```rust
// Example pattern (actual code may vary)
fn check_type_recursive(&mut self, type_id: TypeId) -> Type {
    // Add depth check at start
    if self.recursion_depth > MAX_RECURSION_DEPTH {
        return self.any_type(); // Graceful fallback
    }

    self.recursion_depth += 1;
    let result = self.check_type_recursive_impl(type_id);
    self.recursion_depth -= 1;
    result
}
```

**Possible Locations:**
- `wasm/src/checker/solver.rs` - Type solving with unions/intersections
- `wasm/src/checker/thin_checker.rs` - Type declaration checking
- `wasm/src/checker/types.rs` - Type instantiation/substitution

**Depth Limit:**
- Start with 100 (TypeScript uses similar values)
- Adjust based on test results
- Too low: Breaks valid deep types
- Too high: Doesn't prevent crashes

### Deliverables
1. Code changes adding recursion depth counters
2. Test showing recursiveTypes test passes without crash
3. Conformance test report showing zero crashes
4. Set `Ready for Merge: Yes` when complete

### Workflow
1. **PAUSE** Application Expansion Tests work (commit current progress)
2. Sync: `git fetch origin && git merge origin/rust --no-edit`
3. Investigate crash by running recursiveTypes test
4. Implement recursion guards
5. Test: `./wasm/test.sh`
6. Commit: `[wasm] checker: add recursion guards to prevent stack overflow`
7. Push to worker-4 branch
8. Run conformance tests to verify zero crashes
9. Mark `Ready for Merge: Yes`
10. **OPTIONAL:** Resume Application Expansion Tests after Recursion Guards complete

### Priority Note
🔴 **This is a CRITICAL stability task that blocks all testing.** Complete this before finishing Application Expansion Tests.

## Status
- **Flow Recording:** ✅ Complete
- **Application Expansion:** 🔄 In Progress (can pause)
- **Recursion Guards:** 🔴 NEW - Critical Priority
- **Ready for Merge:** No
- **Last Updated:** 2026-01-15
