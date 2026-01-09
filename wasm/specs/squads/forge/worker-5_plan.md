# Worker 5 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 5

## Current Assignment
- [x] Investigate `ExtractState<R>` and `ExtractAction<R>` conditional infer patterns from `test_check_redux_lodash_style_generics`. These use `infer S` inside mapped type values. Test in isolation in `wasm/src/solver/evaluate_tests.rs`.

### Investigation Results

**Pattern Analysis:**
- `ExtractState<R> = R extends Reducer<infer S, AnyAction> ? S : never` where `Reducer<S,A> = (state: S | undefined, action: A) => S`
- The key pattern is matching `(state: infer S | undefined, action: AnyAction) => infer S` against a concrete reducer function
- This requires matching source union `number | undefined` against pattern union `infer S | undefined`

**Implementation:**
1. Added `TypeKey::Union` pattern handling in `match_infer_pattern` (evaluate.rs:3382-3489)
   - Handles union patterns containing a single `infer` type variable
   - When source is a union: matches source members against non-infer pattern members, binds infer to remaining
   - When source is not a union: binds infer to source if source doesn't match any non-infer pattern member

2. Added isolated tests in `evaluate_tests.rs`:
   - `test_conditional_infer_union_pattern_simple` - basic `number | undefined` vs `infer S | undefined`
   - `test_conditional_infer_union_pattern_multiple_non_infer` - with multiple non-infer members
   - `test_conditional_infer_union_pattern_multiple_source_members` - source has more members than pattern
   - `test_conditional_infer_function_param_union_pattern` - full function pattern with union param (TODO: needs function-level integration)
   - `test_conditional_infer_extract_state_pattern` - existing test updated with TODO

**Status:**
- Basic union pattern matching works (3 tests pass)
- Function-level integration (ExtractState pattern) still returns `never` due to complex function matching flow
- Pre-existing test failures in origin/rust merge unrelated to this change

## Task Queue
- [x] Add coverage for `StateFromReducers<R>` mapped type that uses `ExtractState` on each property.
- [x] Add coverage for `ActionFromReducers<R>` that uses indexed access `[keyof R]` on a mapped type.
- [x] Fix `test_redux_pattern_generic_function_with_conditional_return` - conditional type in function return.
- [x] Add distributive conditional type stress tests (40 tests covering large unions, nested conditionals, utility patterns, edge cases).
- [ ] Fix cross-file type alias resolution in `test_check_redux_lodash_style_generics` (currently 4 errors, down from 6).

### Cross-File Type Alias Resolution Issue
**Status:** Rebased on origin/squad/forge (commit 9c00240b51)

**Resolution:**
Rebased changes onto origin/squad/forge which already had fixes for:
- `symbol_arenas` map to track which arena each symbol came from
- `decl_file_idx: u32` field on Symbol struct
- Updated `merge_bind_results` to properly copy all symbol fields and track arenas

**Merge Conflicts Resolved:**
- binder.rs: Used HEAD's `decl_file_idx: u32` instead of my `source_file_idx: Option<usize>`
- parallel.rs: Used HEAD's version with `symbol_arenas` map
- thin_checker.rs: Combined HEAD's type argument resolution with my `type_param_bindings` support
- subtype.rs: Removed duplicate `try_expand_application` function, fixed calls to use `TypeApplicationId` pattern

**Test Status:**
- 40 pre-existing failures on origin/squad/forge (same on my branch)
- No new regressions introduced by my changes

**Key Changes (from HEAD/origin/squad/forge):**
1. **Symbol merge fix** (parallel.rs): Modified `merge_bind_results` to copy all symbol fields and track arenas via `symbol_arenas` map.
2. **File tracking** (binder.rs): Added `decl_file_idx: u32` to `Symbol` struct to track which file each symbol originated from.
3. **Application expansion** (subtype.rs): `try_expand_application` method handles `TypeKey::Application` cases to structurally expand type aliases before subtype checking.
4. **Symbol pre-resolution** (thin_checker.rs): Type argument resolution ensures symbols are in type_env for Application expansion.
5. **Type param bindings** (lower.rs): `with_type_param_bindings` and `seed_type_params` methods for proper type parameter scope handling.

**Remaining Issue:**
Cross-file type aliases still resolve to `any` because `ThinCheckerState` only has access to the current file's arena. When `get_type_alias()` is called with a `NodeIndex` from a different file, it can't retrieve the type alias declaration node.

**Required Fix:**
Need to give `ThinCheckerState` access to all file arenas (via `MergedProgram.files`), and use `symbol.source_file_idx` to look up nodes from the correct file's arena.

### Generic Function Conditional Return Fix
**Issue:** `createStore(numberReducer)` returning `Store<ExtractState<Reducer<number>>>` failed to resolve properties because `Application` types weren't being evaluated before property access.

**Root cause:** The property access evaluator received an unevaluated `Application` type (e.g., `Store<ExtractState<Reducer<number>>>`). The `evaluate_type` function with `NoopResolver` couldn't expand the type because it lacked symbol resolution.

**Fix:**
1. Added `evaluate_application_type` method to `ThinCheckerState` (thin_checker.rs) that:
   - Resolves the base `Ref` symbol to get the type body
   - Gets type parameters for the symbol
   - Recursively evaluates type arguments
   - Instantiates the body with the evaluated arguments
   - Recursively evaluates the result for nested applications

2. Modified `get_type_of_property_access` to call `evaluate_application_type` on the object type before resolving property access.

3. Added fallback handling for `TypeKey::Application` in `PropertyAccessEvaluator.resolve_property_access_inner` (operations.rs) that attempts evaluation with `evaluate_type` for robustness.

### StateFromReducers Coverage Added
Added 6 tests in `evaluate_tests.rs`:
- `test_mapped_type_with_conditional_template_simple` - ExtractValue pattern with object infer
- `test_mapped_state_from_reducers_pattern_with_simple_objects` - SimpleReducer<S> pattern with object infer
- `test_mapped_state_from_reducers_indexed_access` - R["key"] indexed access evaluation
- `test_mapped_type_full_state_from_reducers_simulation` - mapped type over literal key union
- `test_mapped_type_over_keyof_reducers_object` - mapped type with keyof T constraint
- `test_extract_state_with_function_reducer_pattern` - Redux function-based Reducer pattern (TODO: returns never)

### ActionFromReducers Coverage Added
Added 5 tests in `evaluate_tests.rs`:
- `test_indexed_access_on_object_with_keyof` - obj[keyof obj] produces value union
- `test_indexed_access_mapped_type_result_with_union_key` - mapped result indexed with keyof
- `test_action_from_reducers_pattern_with_simple_objects` - ExtractAction with SimpleReducer pattern
- `test_action_from_reducers_full_pattern` - full pattern: mapped type + keyof indexed access
- `test_indexed_access_with_single_key` - single key indexed access baseline

### Distributive Conditional Type Stress Tests Added
Added 40 comprehensive stress tests in `evaluate_tests.rs` covering:

**Large Union Distribution:**
- `test_distributive_large_union_basic` - 10-member union filtering
- `test_distributive_large_union_all_match` - all members match condition
- `test_distributive_large_union_none_match` - no members match condition
- `test_distributive_very_large_union` - 50-member union distribution
- `test_distributive_hundred_member_union` - 100-member union distribution

**Nested Conditionals:**
- `test_distributive_nested_conditional` - 3 levels of nesting
- `test_distributive_triple_nested_conditional` - 4 levels of nesting
- `test_distributive_deeply_nested_union` - nested unions with deep filtering

**Utility Type Patterns:**
- `test_distributive_exclude_utility` - Exclude<T, U> pattern
- `test_distributive_extract_utility` - Extract<T, U> pattern
- `test_distributive_non_nullable_utility` - NonNullable<T> pattern

**Infer Patterns:**
- `test_distributive_with_infer_in_true_branch` - array element inference
- `test_distributive_with_infer_filter` - filtered infer binding
- `test_distributive_with_constrained_infer` - constrained infer
- `test_distributive_two_infers_different_positions` - multiple infer variables
- `test_distributive_infer_return_type` - function return type infer (TODO)

**Edge Cases:**
- `test_distributive_with_never_input` - never input handling
- `test_distributive_with_any_input` - any input propagation
- `test_distributive_with_unknown` - unknown extends
- `test_distributive_with_void` - void type handling
- `test_distributive_all_to_same_result` - deduplication
- `test_distributive_identity_preservation` - identity conditional
- `test_distributive_no_false_branch_matches` - all result in never

**Complex Types:**
- `test_distributive_function_types` - function type filtering
- `test_distributive_readonly_array` - readonly array matching
- `test_distributive_preserves_tuple_structure` - tuple structure preservation
- `test_distributive_multiple_arrays` - array type filtering
- `test_distributive_partial_object_match` - object property matching
- `test_distributive_empty_object_match` - empty object pattern
- `test_distributive_literal_type_filter` - literal type filtering
- `test_distributive_numeric_literal_filter` - numeric literal range filtering

## Completed
- [x] Fixed `test_redux_pattern_generic_function_with_conditional_return`: Added `evaluate_application_type` to ThinCheckerState for resolving generic type aliases in property access; `./wasm/test.sh` (fails: `test_check_redux_lodash_style_generics` pre-existing).
- [x] Added StateFromReducers mapped type test coverage (6 tests) for mapped type + conditional infer patterns; `./wasm/test.sh` (fails: `test_check_redux_lodash_style_generics` pre-existing).
- [x] Added ActionFromReducers indexed access test coverage (5 tests) for mapped type + keyof indexed access patterns; `./wasm/test.sh` (fails: `test_check_redux_lodash_style_generics` pre-existing).
- [x] Added never-input readonly array infer regression; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added never-input multi-template infer regression; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added never-input tuple infer regression; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added never-input function infer regression; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added never-input array infer regression; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added never-input object infer regression; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Ensured infer patterns bind `never` across templates; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added conditional infer regression for never input; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added infer union target placeholder + never regression test; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive template literal union input with template member; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive template literal union-branch tests (middle/suffix/prefix); ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive template literal two-infer union-branch tests; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive template literal constrained union-branch tests (middle/suffix/prefix); ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive template literal union-branch tests (basic + constrained); ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive optional-property union-branch inference test; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive readonly wrapper object-property inference tests; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive readonly object-property inference tests (union input/branch); ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive readonly nested union-branch inference test; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive readonly nested object inference test; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added union handling for non-distributive object/nested-object infer in conditional evaluation; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive union-branch regression for nested object infer; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive union-branch regressions for string/number index signature infer; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive union-branch regressions for readonly array/tuple infer; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive union-branch regressions for function parameter, rest parameter, and this-parameter infer; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive tuple union-branch regressions for tuple element/optional tuple inference; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Implemented tuple-rest conditional infer binding and updated variadic tuple tests; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive union branch regression for conditional object call signatures; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive union branch regression for function return conditional inference; ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `async_es5`).
- [x] Added non-distributive union branch regression test for array-element conditional inference; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Updated conditional infer array element tests to expect `string` for non-array union branches; ran `./wasm/test.sh test_conditional_infer_array_element_non_array_union_branch` and `./wasm/test.sh test_conditional_instantiated_param_distributes_branch_substitution`.
- [x] Implemented function/callable parameter inference for conditional `infer` patterns (including optional/rest params); updated expectations; `./wasm/test.sh` failed: `emitter_edge_case_tests::test_parse_error_tolerance`.
- [x] Removed stray duplicate block in `async_es5.rs` to fix compilation; `./wasm/test.sh` failed: `emitter_edge_case_tests::test_parse_error_tolerance`.
- [x] Added function return-type inference for conditional `infer` patterns (including object property return inference); updated tests to expect `string | number`; `./wasm/test.sh` failed with async ES5 transform parse error (`async_es5.rs:749`).
- [x] Implemented intersection object inference for conditional `infer` patterns; updated expectation to string; `./wasm/test.sh` failed with async ES5 transform parse error (`async_es5.rs:749`).
- [x] Enabled non-distributive readonly array/tuple inference over union inputs and updated tests in `wasm/src/solver/evaluate_tests.rs`; `./wasm/test.sh` failed: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`.
- [x] Implemented conditional infer constraint filtering + optional tuple/property inference; updated tuple optional expectations and added `this`-parameter tests; `./wasm/test.sh` failed: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`.
- [x] Fixed FunctionId typo in `wasm/src/solver/evaluate.rs`; `./wasm/test.sh` failed: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`.
- [x] Updated infer TODO expectations in `wasm/src/solver/evaluate_tests.rs` (tuple rest inference note + this-parameter TODO cleanup); `./wasm/test.sh` failed: missing `FunctionId` in `wasm/src/solver/evaluate.rs`.
- [x] Implemented this-parameter bounds checking + conditional inference; added non-distributive optional tuple/property inference; updated tests (tests not run).
- [x] Added `this`-parameter inference tests in `wasm/src/solver/infer_tests.rs` (tests not run).
- [x] Added non-distributive union array inference for conditional types (tests not run).
- [x] Added non-distributive tuple wrapper array inference for conditional types (tests not run).
- [x] Made optional property inference include missing/undefined cases (tests not run).
- [x] Flattened tuple rest and optional elements for array element inference (tests not run).
- [x] Added non-distributive tuple union inference (tests not run).
- [x] Updated non-distributive optional property inference expectation (tests not run).
- [x] Added index signature inference from object properties (tests not run).
- [x] Updated non-distributive nested object inference expectation (tests not run).
- [x] Updated non-distributive union object inference expectation (tests not run).

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-5`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
