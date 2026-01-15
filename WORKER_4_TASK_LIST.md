# Worker-4 Task List

## ✅ COMPLETED: Flow Recording (2026-01-15)
- **Status:** Complete and merged to em-team-1
- **Summary:** Fixed flow recording for statements and identifiers
- **Test Results:** All 54/54 control_flow tests passing 🎉
- **Commits:**
  - a163cbed8c9 [wasm] binder: add flow recording for statements and identifiers
  - 4c2544eb316 [wasm] flow: fix literal type narrowing in assignments

---

## ✅ COMPLETED: Application Expansion Tests (2026-01-15)
- **Status:** Complete and pushed (commit b2b7678f5bf)
- **Summary:** Fixed all failing application expansion tests in the type solver
- **Test Results:** All 34/34 application expansion tests passing 🎉

## Changes Made:
1. **Fixed test setup** (`evaluate_tests.rs`):
   - Changed `env.insert()` to `env.insert_with_params()` to register type parameters
   - Added `.clone()` when creating TypeParameter types to allow reuse

2. **Added default type parameter support** (`instantiate.rs`):
   - Modified `TypeSubstitution::from_args()` to handle default type parameters
   - When fewer type arguments than parameters, defaults are now used

## Tests Fixed:
- test_application_ref_expansion_with_constraints
- test_application_ref_expansion_with_defaults
- test_application_ref_expansion_with_never_arg
- test_application_ref_expansion_with_unknown_arg
- test_application_ref_expansion_with_any_arg
- test_application_ref_expansion_with_union_arg
- test_application_ref_expansion_nested
- test_application_ref_expansion_reducer_function

---

## 🎯 CRITICAL: Recursion Guards (Stack Overflow) 🔴
**Priority:** CRITICAL (STABILITY)
**Assigned:** 2026-01-15
**Owner:** worker-4
**Branch:** worker-4

### Task Description
Fix stack overflow crashes in the type checker by adding recursion depth counters. The recursiveTypes test currently causes 2 stack overflow crashes, blocking all validation work.

### Problem Analysis
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
- `wasm/src/solver/subtype.rs` - Subtype checking (already has depth counter!)
- `wasm/src/solver/evaluate.rs` - Type evaluation
- `wasm/src/checker/thin_checker.rs` - Type declaration checking

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

### Status
- **Flow Recording:** ✅ Complete
- **Application Expansion:** ✅ Complete
- **Recursion Guards:** ⚠️ REASSIGNED - See RECURSION_GUARDS_FINDINGS.md
  - **Note:** Worker-3 investigated and verified recursion guards are already implemented
  - Zero crashes in all test scenarios
- **Ready for Merge:** ✅ YES - Merged to em-team-1 (commit e3feed998c5)
- **Last Updated:** 2026-01-15

---

## Worker-4 Merge Summary

**Merge Commit:** `e3feed998c5` (pushed to origin/em-team-1)

### Completed Tasks Merged:
1. **Flow Recording** ✅
   - Fixed flow recording for statements and identifiers
   - All 54/54 control_flow tests passing
   - Commits: a163cbed8c9, 4c2544eb316

2. **Application Expansion Tests** ✅
   - Fixed all 34/34 application expansion tests
   - Added default type parameter support
   - Fixed test setup with `insert_with_params()`

### Code Changes:
- `wasm/src/solver/evaluate_tests.rs` - Test improvements
- `wasm/src/solver/instantiate.rs` - Default type parameter support

### Reassigned:
- **Recursion Guards** - Reassigned to worker-3 (investigation complete, already implemented)
