# Worker 2 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 2

## Current Assignment
- Await next assignment.

## Task Queue
- [ ] Await next assignment.

## Completed
- [x] Fixed distributive conditional instantiation to evaluate branches per union member. Tests: `./wasm/test.sh test_conditional_instantiated_param_distributes_branch_substitution`.
- [x] Added nested/distributive conditional tests covering `extends` + `infer` and substituted infer during evaluation. Tests: `./wasm/test.sh test_conditional_distributive_`.
- [x] Added infer-in-branch conditional tests (true/false) to ensure substitution holds. Tests: `./wasm/test.sh test_conditional_infer_`.
- [x] Added infer extraction for array conditional evaluation. Tests: `./wasm/test.sh test_conditional_infer_array_element_extraction`.
- [x] Added infer extraction for tuple conditional evaluation. Tests: `./wasm/test.sh test_conditional_infer_tuple_element_extraction`.
- [x] Added readonly array/tuple infer extraction tests and unwrapped readonly in evaluation. Tests: `./wasm/test.sh test_conditional_infer_readonly_`.
- [x] Added mixed readonly/mutable array infer extraction test. Tests: `./wasm/test.sh test_conditional_infer_readonly_array_mixed_input`.
- [x] Added constrained infer extraction tests for arrays/tuples. Tests: `./wasm/test.sh test_conditional_infer_array_element_with_constraint`.
- [x] Added non-distributive infer extraction test for array of T. Tests: `./wasm/test.sh test_conditional_infer_array_element_non_distributive`.
- [x] Added distributive infer extraction test for object properties. Tests: `./wasm/test.sh test_conditional_infer_object_property_distributive`.
- [x] Added constrained infer extraction test for object properties. Tests: `./wasm/test.sh test_conditional_infer_object_property_with_constraint`.
- [x] Added nested object property infer extraction test. Tests: `./wasm/test.sh test_conditional_infer_nested_object_property_distributive`.
- [x] Added nested object property infer extraction test with constraint. Tests: `./wasm/test.sh test_conditional_infer_nested_object_property_with_constraint`.
- [x] Added nested object property infer extraction test with readonly. Tests: `./wasm/test.sh test_conditional_infer_nested_object_property_readonly`.
- [x] Added nested object property infer test with non-matching union branch. Tests: `./wasm/test.sh test_conditional_infer_nested_object_property_non_matching_branch`.
- [x] Added object property infer test with non-object union branch. Tests: `./wasm/test.sh test_conditional_infer_object_property_non_object_union_branch`.
- [x] Added non-distributive object infer test with non-object union branch. Tests: `./wasm/test.sh test_conditional_infer_object_property_non_distributive_union_branch`.
- [x] Added nested object infer test with readonly wrapper. Tests: `./wasm/test.sh test_conditional_infer_nested_object_property_readonly_wrapper`.
- [x] Added nested object infer test with union inner property. Tests: `./wasm/test.sh test_conditional_infer_nested_object_property_union_value`.
- [x] Added index signature infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_object_index_signature_distributive`.
- [x] Added tuple rest infer extraction test for array element conditional. Tests: `./wasm/test.sh test_conditional_infer_array_element_from_tuple_rest`.
- [x] Added optional property infer test for missing object (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_optional_property_missing_object`.
- [x] Added tuple rest tuple infer test for array element conditional (current behavior keeps rest tuple). Tests: `./wasm/test.sh test_conditional_infer_array_element_from_tuple_rest_tuple`.
- [x] Added object intersection infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_object_property_intersection_check`.
- [x] Added function parameter infer test for distributive conditionals (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_function_param_distributive`.
- [x] Added function return infer test for distributive conditionals (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_function_return_distributive`.
- [x] Added non-distributive object-property infer test over matching union inputs (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_object_property_non_distributive_union_all_match`.
- [x] Added non-distributive tuple wrapper array infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_array_element_non_distributive_tuple_wrapper`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
