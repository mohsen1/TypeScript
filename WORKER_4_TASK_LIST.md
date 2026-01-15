# Worker-4 Task List

## Previous Assignment: Flow Analysis Tests ✅ COMPLETED
- **Status:** Complete and merged
- **Summary:** Fixed flow recording and literal type narrowing for control flow analysis
- **Test Results:** All 54/54 control_flow tests passing 🎉

---

## Assignment: Application Expansion Tests ✅ IN PROGRESS
- **Status:** In Progress (29/34 tests passing)
- **Objective:** Fix remaining failing application expansion tests in the type solver

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
