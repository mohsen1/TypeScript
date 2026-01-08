# Worker 3 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 3

## Current Assignment
- Awaiting next manager assignment.

## Task Queue
- [ ] (none)

## Completed
- [x] Solver variance: added param contravariance and return covariance tests in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_function_variance_`.
- [x] Solver variance: added optional/rest method/constructor edge cases in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_variance_optional_rest_`.
- [x] Solver variance: required vs optional parameter count coverage plus required-count checks in `wasm/src/solver/subtype.rs`. Tests: `./wasm/test.sh test_function_required_count_`.
- [x] Solver variance: method vs function `this` parameter assignability tests in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_this_parameter_`.
- [x] Solver variance: optional/rest + `this` parameter assignability for method vs function properties in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_variance_optional_rest_`.
- [x] Solver unsoundness: void return exception coverage in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_void_return_exception_subtype`.
- [x] Solver unsoundness: method bivariance regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_method_bivariant_required_param`.
- [x] Solver unsoundness: Function top assignability regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_function_top_assignability`.
- [x] Solver unsoundness: covariant mutable arrays regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_array_covariant_mutable_unsoundness`.
- [x] Solver unsoundness: rest parameter bivariance regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_rest_any_bivariant_subtyping_toggle`.
- [x] Solver unsoundness: tuple-array assignment regressions in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_tuple_array_assignment_`.
- [x] Solver unsoundness: Object vs object vs {} trifecta regressions in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_object_trifecta_`.
- [x] Solver unsoundness: weak type detection regressions in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_weak_type_detection_`.
- [x] Solver unsoundness: legacy null/undefined subtyping in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_legacy_null_undefined_subtyping`.
- [x] Solver unsoundness: no-unchecked indexed access tuple subtyping in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_no_unchecked_indexed_access_tuple_subtyping`.
- [x] Solver unsoundness: error poisoning regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_error_poisoning_`.
- [x] Solver unsoundness: optional property undefined toggle in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_exact_optional_property_types_toggle`.
- [x] Solver unsoundness: split accessor variance regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_split_accessor_variance`.
- [x] Solver unsoundness: constructor void exception regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_constructor_void_exception_subtype`.
- [x] Solver unsoundness: intersection reduction (disjoint intrinsics) regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_intersection_reduction_disjoint_intrinsics`.
- [x] Solver unsoundness: primitive boxing (`number` to `Number` interface) regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_primitive_boxing_assignability`.
- [x] Solver unsoundness: primitive boxing (`bigint` to `BigInt` interface) regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_primitive_boxing_bigint_assignability`.
- [x] Solver unsoundness: primitive boxing (`boolean` to `Boolean` interface) regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_primitive_boxing_boolean_assignability`.
- [x] Solver unsoundness: primitive boxing (`string` to `String` interface) regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_primitive_boxing_string_assignability`.
- [x] Solver unsoundness: primitive boxing (`symbol` to `Symbol` interface) regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_primitive_boxing_symbol_assignability`.
- [x] Solver unsoundness: any top/bottom regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_any_top_bottom_subtyping`.
- [x] Solver unsoundness: apparent string member regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_apparent_string_member_subtyping`.
- [x] Solver unsoundness: keyof union disjoint keys regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_keyof_union_disjoint_object_keys_is_never`.
- [x] Solver unsoundness: keyof union overlapping keys regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_keyof_union_overlapping_keys_is_common`.
- [x] Solver unsoundness: keyof union string index + literal narrowing regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_keyof_union_string_index_and_literal_narrows`.
- [x] Solver unsoundness: keyof intersection union-of-keys regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_keyof_intersection_union_of_keys`.
- [x] Solver unsoundness: mapped type over primitive number keys regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_mapped_type_over_number_keys_subtyping`.
- [x] Solver unsoundness: mapped type over primitive string keys regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_mapped_type_over_string_keys_subtyping`.
- [x] Solver unsoundness: index signature consistency regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_object_to_indexed_property_mismatch_string_index`.
- [x] Solver unsoundness: apparent boolean member regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_apparent_boolean_member_subtyping`.
- [x] Solver unsoundness: apparent symbol member regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_apparent_symbol_member_subtyping`.
- [x] Solver unsoundness: apparent bigint member regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_apparent_bigint_member_subtyping`.
- [x] Solver unsoundness: mapped type over primitive boolean keys regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_mapped_type_over_boolean_keys_subtyping`.
- [x] Solver unsoundness: apparent object member regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_apparent_object_member_subtyping`.
- [x] Solver unsoundness: mapped type over primitive symbol keys regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_mapped_type_over_symbol_keys_subtyping`.
- [x] Solver unsoundness: mapped type over primitive bigint keys regression in `wasm/src/solver/subtype_tests.rs`. Tests: `./wasm/test.sh test_mapped_type_over_bigint_keys_subtyping`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
