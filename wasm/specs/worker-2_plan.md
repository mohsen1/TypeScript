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
- [x] Added non-distributive union-array infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_array_element_non_distributive_union_input`.
- [x] Added non-distributive nested object infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_nested_object_property_non_distributive_union_input`.
- [x] Added tuple infer test with non-tuple union branch. Tests: `./wasm/test.sh test_conditional_infer_tuple_element_non_tuple_union_branch`.
- [x] Added non-distributive tuple infer test over union input (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_tuple_element_non_distributive_union_input`.
- [x] Added non-distributive readonly array infer test over union input (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_readonly_array_element_non_distributive_union_input`.
- [x] Added non-distributive readonly tuple infer test over union input (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_readonly_tuple_element_non_distributive_union_input`.
- [x] Added readonly array infer test with non-array union branch. Tests: `./wasm/test.sh test_conditional_infer_readonly_array_element_non_array_union_branch`.
- [x] Added readonly tuple infer test with non-tuple union branch. Tests: `./wasm/test.sh test_conditional_infer_readonly_tuple_element_non_tuple_union_branch`.
- [x] Added non-distributive index signature infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_object_index_signature_non_distributive_union_input`.
- [x] Added index signature infer test with non-object union branch (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_object_index_signature_non_object_union_branch`.
- [x] Added number index signature infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_number_index_signature_distributive`.
- [x] Added non-distributive number index signature infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_number_index_signature_non_distributive_union_input`.
- [x] Added non-distributive object property infer test with non-object union branch (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_object_property_non_distributive_non_object_union_branch`.
- [x] Added object property function-return infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_object_property_function_return_distributive`.
- [x] Added template literal infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_template_literal_distributive`.
- [x] Added readonly object property infer test. Tests: `./wasm/test.sh test_conditional_infer_object_property_readonly`.
- [x] Added array infer test with non-array union branch. Tests: `./wasm/test.sh test_conditional_infer_array_element_non_array_union_branch`.
- [x] Added optional tuple element infer test. Tests: `./wasm/test.sh test_conditional_infer_tuple_optional_element_distributive`.
- [x] Added non-distributive optional tuple infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_tuple_optional_element_non_distributive_union_input`.
- [x] Added non-distributive optional property infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_optional_property_non_distributive_union_input`.
- [x] Added optional tuple element array infer test (current behavior omits undefined). Tests: `./wasm/test.sh test_conditional_infer_array_element_from_optional_tuple_element`.
- [x] Added function this-parameter infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_function_this_param_distributive`.
- [x] Added optional property present infer test (current behavior omits undefined). Tests: `./wasm/test.sh test_conditional_infer_optional_property_present_distributive`.
- [x] Added tuple rest infer test (current behavior yields infer placeholder). Tests: `./wasm/test.sh test_conditional_infer_tuple_rest_distributive`.
- [x] Added union true-branch infer preservation test. Tests: `./wasm/test.sh test_conditional_infer_union_true_branch_distributive`.
- [x] Added union false-branch infer preservation test. Tests: `./wasm/test.sh test_conditional_infer_union_false_branch_distributive`.
- [x] Added any-check infer preservation test. Tests: `./wasm/test.sh test_conditional_infer_any_check_type_distributive`.
- [x] Added non-distributive function parameter infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_function_param_non_distributive_union_input`.
- [x] Added non-distributive function this-parameter infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_function_this_param_non_distributive_union_input`.
- [x] Added non-distributive function return infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_function_return_non_distributive_union_input`.
- [x] Added template literal prefix infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_template_literal_with_prefix_distributive`.
- [x] Added template literal suffix infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_template_literal_with_suffix_distributive`.
- [x] Added function rest parameter infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_function_rest_param_distributive`.
- [x] Added non-distributive template literal infer test (current behavior yields never). Tests: `./wasm/test.sh test_conditional_infer_template_literal_non_distributive_union_input`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
