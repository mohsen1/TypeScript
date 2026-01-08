# Worker 1 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 1

## Current Assignment
- Awaiting next assignment.

## Task Queue
- [ ] Add mixed-case exponent missing sign with leading zeros test (`1Ee01`).
- [ ] Add mixed-case exponent trailing double-minus test (`1Ee--`).
- [ ] Add uppercase exponent leading zeros with zero exponent test (`1E+00`).
- [ ] Add lowercase exponent plus missing digits test (`1e+`).
- [ ] Add uppercase exponent plus missing digits test (`1E+`).
- [ ] Add numeric separator underscore name test (`1_0` or `1_0e1`).
- [ ] Add numeric separator in integer name test (`1_000`).
- [ ] Add numeric separator in fractional name test (`1.0_0`).
- [ ] Add numeric separator adjacent to decimal point test (`1_.0`).
- [ ] Add numeric separator in exponent digits test (`1e1_0`).
- [ ] Add numeric separator in exponent sign/digits test (`1e+_1`).
- [ ] Add multiple underscore name test (`1__0`).
- [ ] Add trailing underscore name test (`1_`).
- [ ] Add leading-zero with underscore name test (`0_1`).
- [ ] Add numeric separator in hex/binary name test (`0x1_0`, `0b1_0`, `0o1_0`).

## Completed
- [x] Number index ignores mixed-case negative exponent leading zeros; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_negative_leading_zeros`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_negative_leading_zeros`.
- [x] Docker build failure in `wasm/src/cli/driver.rs` (E0515) resolved via rust merge; validated by `./wasm/test.sh test_resolve_bounds_number_index_ignores_positive_exponent_zero`.
- [x] Conditional infer object coverage in `wasm/src/solver/evaluate.rs` with tests in `wasm/src/solver/evaluate_tests.rs`. Tests: `./wasm/test.sh test_conditional_infer_object_`.
- [x] Prefer upper bounds when lower bounds are only `any`/`unknown`; added `test_resolve_any_lower_prefers_upper_bound`. Tests: `./wasm/test.sh test_resolve_any_lower_prefers_upper_bound`.
- [x] Ignore circular upper bounds during inference resolution; added `test_resolve_circular_upper_bound_defaults_unknown`. Tests: `./wasm/test.sh test_resolve_circular_upper_bound_defaults_unknown`.
- [x] Prefer upper bounds when lower bounds are only `error`; added `test_resolve_error_lower_prefers_upper_bound`. Tests: `./wasm/test.sh test_resolve_error_lower_prefers_upper_bound`.
- [x] Contextual inference prefers specific lower bounds over `any`; added `test_resolve_contextual_ignores_any_lower_with_literal`. Tests: `./wasm/test.sh test_resolve_contextual_ignores_any_lower_with_literal`.
- [x] Drop mutual circular upper bounds across type params; added `test_resolve_mutual_circular_upper_bounds_unknown`. Tests: `./wasm/test.sh test_resolve_mutual_circular_upper_bounds_unknown`.
- [x] Guard self-recursive object bounds on multiple params; added `test_resolve_self_recursive_object_bounds_two_params_unknown`. Tests: `./wasm/test.sh test_resolve_self_recursive_object_bounds_two_params_unknown`.
- [x] Guard mutual recursive object bounds; added `test_resolve_mutual_recursive_object_bounds_unknown`. Tests: `./wasm/test.sh test_resolve_mutual_recursive_object_bounds_unknown`.
- [x] Contextual literal selection over `any`; added `test_apply_contextual_any_uses_literal_context`. Tests: `./wasm/test.sh test_apply_contextual_any_uses_literal_context`.
- [x] Union context preserves literal expressions; added `test_apply_contextual_union_preserves_literal`. Tests: `./wasm/test.sh test_apply_contextual_union_preserves_literal`.
- [x] Contextual union keeps inferred literal for generic call; added `test_contextual_generic_call_union_preserves_literal`. Tests: `./wasm/test.sh test_contextual_generic_call_union_preserves_literal`.
- [x] Contextual union keeps inferred literal for generic return; added `test_contextual_generic_return_union_preserves_literal`. Tests: `./wasm/test.sh test_contextual_generic_return_union_preserves_literal`.
- [x] Union function context keeps literal return; added `test_contextual_union_function_return_preserves_literal`. Tests: `./wasm/test.sh test_contextual_union_function_return_preserves_literal`.
- [x] Union function param/return context keeps literal; added `test_contextual_union_function_param_return_preserves_literal`. Tests: `./wasm/test.sh test_contextual_union_function_param_return_preserves_literal`.
- [x] Union parameter context keeps literal; added `test_contextual_union_param_preserves_literal`. Tests: `./wasm/test.sh test_contextual_union_param_preserves_literal`.
- [x] Union arity context preserves literal param; added `test_contextual_union_arity_param_preserves_literal`. Tests: `./wasm/test.sh test_contextual_union_arity_param_preserves_literal`.
- [x] Union rest param context preserves literal; added `test_contextual_union_rest_param_preserves_literal`. Tests: `./wasm/test.sh test_contextual_union_rest_param_preserves_literal`.
- [x] Union empty/one param context preserves literal; added `test_contextual_union_empty_param_preserves_literal`. Tests: `./wasm/test.sh test_contextual_union_empty_param_preserves_literal`.
- [x] Union optional/required param preserves literal; added `test_contextual_union_optional_param_preserves_literal`. Tests: `./wasm/test.sh test_contextual_union_optional_param_preserves_literal`.
- [x] Circular extends constraints resolve to unknown; added `test_resolve_all_with_circular_extends_unknown`. Tests: `./wasm/test.sh test_resolve_all_with_circular_extends_unknown`.
- [x] Contextual generic return uses union context for `any`; added `test_contextual_generic_return_union_any_uses_context`. Tests: `./wasm/test.sh test_contextual_generic_return_union_any_uses_context`.
- [x] Mutual circular upper bounds with concrete bound resolves to concrete; added `test_resolve_mutual_circular_upper_bounds_with_concrete`. Tests: `./wasm/test.sh test_resolve_mutual_circular_upper_bounds_with_concrete`.
- [x] Error lower bound with literal prefers literal; added `test_resolve_error_lower_with_literal_prefers_literal`. Tests: `./wasm/test.sh test_resolve_error_lower_with_literal_prefers_literal`.
- [x] Unknown lower bound prefers upper bound; added `test_resolve_unknown_lower_prefers_upper_bound`. Tests: `./wasm/test.sh test_resolve_unknown_lower_prefers_upper_bound`.
- [x] Unified vars reuse merged constraints; added `test_resolve_unified_vars_merged_constraints`. Tests: `./wasm/test.sh test_resolve_unified_vars_merged_constraints`.
- [x] Self upper bound with concrete upper resolves to concrete; added `test_resolve_self_upper_bound_with_concrete`. Tests: `./wasm/test.sh test_resolve_self_upper_bound_with_concrete`.
- [x] Union lower bound violating string upper errors; added `test_resolve_bounds_union_lower_vs_string_upper`. Tests: `./wasm/test.sh test_resolve_bounds_union_lower_vs_string_upper`.
- [x] Lower bounds ignore `never`; added `test_resolve_lower_bounds_ignores_never`. Tests: `./wasm/test.sh test_resolve_lower_bounds_ignores_never`.
- [x] Duplicate upper bounds resolve without intersection; added `test_resolve_bounds_duplicate_upper_bounds_no_intersection`. Tests: `./wasm/test.sh test_resolve_bounds_duplicate_upper_bounds_no_intersection`.
- [x] Optional upper property accepts required lower; added `test_resolve_bounds_optional_property_compatible`. Tests: `./wasm/test.sh test_resolve_bounds_optional_property_compatible`.
- [x] Optional lower property rejected by required upper; added `test_resolve_bounds_optional_property_mismatch`. Tests: `./wasm/test.sh test_resolve_bounds_optional_property_mismatch`.
- [x] Missing optional property is allowed; added `test_resolve_bounds_optional_property_missing_ok`. Tests: `./wasm/test.sh test_resolve_bounds_optional_property_missing_ok`.
- [x] Object keyword upper rejects string lower; added `test_resolve_bounds_object_keyword_rejects_string`. Tests: `./wasm/test.sh test_resolve_bounds_object_keyword_rejects_string`.
- [x] Missing optional readonly property is allowed; added `test_resolve_bounds_object_readonly_property_missing_ok`. Tests: `./wasm/test.sh test_resolve_bounds_object_readonly_property_missing_ok`.
- [x] String index rejects incompatible property; added `test_resolve_bounds_string_index_property_mismatch`. Tests: `./wasm/test.sh test_resolve_bounds_string_index_property_mismatch`.
- [x] Number index treats Infinity name as numeric; added `test_resolve_bounds_number_index_accepts_infinity_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_accepts_infinity_name`.
- [x] Mutable number index rejects readonly numeric property; added `test_resolve_bounds_number_index_readonly_property_mismatch`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_readonly_property_mismatch`.
- [x] Mutable number index rejects readonly number signature; added `test_resolve_bounds_number_index_readonly_signature_mismatch`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_readonly_signature_mismatch`.
- [x] Readonly number index accepts mutable source; added `test_resolve_bounds_number_index_readonly_signature_allows_mutable_source`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_readonly_signature_allows_mutable_source`.
- [x] Number index treats NaN name as numeric; added `test_resolve_bounds_number_index_accepts_nan_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_accepts_nan_name`.
- [x] Number index treats -Infinity name as numeric; added `test_resolve_bounds_number_index_accepts_negative_infinity_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_accepts_negative_infinity_name`.
- [x] Number index ignores -0 name; added `test_resolve_bounds_number_index_ignores_negative_zero_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_negative_zero_name`.
- [x] Number index ignores -0 property on object; added `test_resolve_bounds_number_index_ignores_negative_zero_property`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_negative_zero_property`.
- [x] Number index treats 1e-6 decimal name as numeric; added `test_resolve_bounds_number_index_accepts_decimal_boundary_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_accepts_decimal_boundary_name`.
- [x] Number index treats 1e+21 exponent name as numeric; added `test_resolve_bounds_number_index_accepts_exponent_boundary_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_accepts_exponent_boundary_name`.
- [x] Number index ignores non-canonical exponent name; added `test_resolve_bounds_number_index_ignores_non_canonical_exponent_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_non_canonical_exponent_name`.
- [x] Number index ignores uppercase exponent name; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_name`.
- [x] Number index ignores uppercase exponent missing sign; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_missing_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_missing_sign`.
- [x] Number index ignores uppercase exponent leading zeros; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_leading_zeros`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_leading_zeros`.
- [x] Number index ignores uppercase exponent leading zeros without sign; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_leading_zeros_without_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_leading_zeros_without_sign`.
- [x] Number index ignores uppercase negative exponent leading zeros; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_negative_leading_zeros`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_negative_leading_zeros`.
- [x] Number index ignores mixed-case exponent; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent`.
- [x] Number index ignores mixed-case exponent with sign; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_with_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_with_sign`.
- [x] Number index ignores mixed-case exponent missing digits; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_missing_digits`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_missing_digits`.
- [x] Number index ignores uppercase exponent missing sign with leading zero; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_missing_sign_with_leading_zero`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_missing_sign_with_leading_zero`.
- [x] Number index ignores mixed-case exponent double sign; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_double_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_double_sign`.
- [x] Number index ignores mixed-case exponent with lowercase e; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_with_lowercase_e`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_with_lowercase_e`.
- [x] Number index ignores mixed-case exponent double minus; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_double_minus`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_double_minus`.
- [x] Number index ignores mixed-case exponent plus-minus; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_plus_minus`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_plus_minus`.
- [x] Number index ignores mixed-case exponent minus-plus; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_minus_plus`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_minus_plus`.
- [x] Number index ignores mixed-case exponent trailing sign; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_trailing_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_trailing_sign`.
- [x] Number index ignores mixed-case exponent trailing minus; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_trailing_minus`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_trailing_minus`.
- [x] Number index ignores mixed-case exponent leading zeros; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_leading_zeros`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_leading_zeros`.
- [x] Number index ignores mixed-case exponent leading zeros without sign; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_leading_zeros_without_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_leading_zeros_without_sign`.
- [x] Number index ignores mixed-case negative exponent zero; added `test_resolve_bounds_number_index_ignores_mixed_case_negative_exponent_zero`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_negative_exponent_zero`.
- [x] Number index ignores mixed-case exponent positive zero; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_positive_zero`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_positive_zero`.
- [x] Number index ignores mixed-case exponent zero without sign; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_zero_without_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_zero_without_sign`.
- [x] Number index ignores mixed-case exponent trailing double sign; added `test_resolve_bounds_number_index_ignores_mixed_case_exponent_double_sign_trailing`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_mixed_case_exponent_double_sign_trailing`.
- [x] Number index ignores uppercase exponent missing digits; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_missing_digits`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_missing_digits`.
- [x] Number index ignores uppercase exponent missing negative digits; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_minus_missing_digits`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_minus_missing_digits`.
- [x] Number index ignores uppercase exponent double sign; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_double_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_double_sign`.
- [x] Number index ignores uppercase exponent double minus; added `test_resolve_bounds_number_index_ignores_uppercase_exponent_double_minus`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_uppercase_exponent_double_minus`.
- [x] Number index ignores negative exponent leading zeros; added `test_resolve_bounds_number_index_ignores_exponent_leading_zeros_negative`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_exponent_leading_zeros_negative`.
- [x] Number index ignores positive exponent leading zeros; added `test_resolve_bounds_number_index_ignores_exponent_leading_zeros_positive`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_exponent_leading_zeros_positive`.
- [x] Number index ignores exponent leading zeros without sign; added `test_resolve_bounds_number_index_ignores_exponent_leading_zeros_without_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_exponent_leading_zeros_without_sign`.
- [x] Number index ignores missing exponent sign; added `test_resolve_bounds_number_index_ignores_missing_exponent_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_missing_exponent_sign`.
- [x] Number index ignores leading-zero decimal name; added `test_resolve_bounds_number_index_ignores_leading_zero_decimal_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_leading_zero_decimal_name`.
- [x] Number index ignores hex literal name; added `test_resolve_bounds_number_index_ignores_hex_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_hex_name`.
- [x] Number index ignores binary literal name; added `test_resolve_bounds_number_index_ignores_binary_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_binary_name`.
- [x] Number index ignores octal literal name; added `test_resolve_bounds_number_index_ignores_octal_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_octal_name`.
- [x] Number index ignores leading-zero mantissa exponent; added `test_resolve_bounds_number_index_ignores_exponent_leading_zero_mantissa`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_exponent_leading_zero_mantissa`.
- [x] Number index ignores leading dot decimal name; added `test_resolve_bounds_number_index_ignores_leading_dot_decimal_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_leading_dot_decimal_name`.
- [x] Number index ignores multiple leading zeros; added `test_resolve_bounds_number_index_ignores_multiple_leading_zeros`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_multiple_leading_zeros`.
- [x] Number index ignores negative hex literal; added `test_resolve_bounds_number_index_ignores_negative_hex_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_negative_hex_name`.
- [x] Number index ignores negative binary literal; added `test_resolve_bounds_number_index_ignores_negative_binary_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_negative_binary_name`.
- [x] Number index ignores negative octal literal; added `test_resolve_bounds_number_index_ignores_negative_octal_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_negative_octal_name`.
- [x] Number index ignores double-sign exponent; added `test_resolve_bounds_number_index_ignores_exponent_double_sign`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_exponent_double_sign`.
- [x] Number index ignores double-minus exponent; added `test_resolve_bounds_number_index_ignores_exponent_double_minus`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_exponent_double_minus`.
- [x] Number index ignores missing exponent digits; added `test_resolve_bounds_number_index_ignores_exponent_missing_digits`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_exponent_missing_digits`.
- [x] Number index ignores missing negative exponent digits; added `test_resolve_bounds_number_index_ignores_exponent_minus_missing_digits`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_exponent_minus_missing_digits`.
- [x] Number index ignores -0 exponent form; added `test_resolve_bounds_number_index_ignores_negative_exponent_zero`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_negative_exponent_zero`.
- [x] Number index ignores positive exponent zero; added `test_resolve_bounds_number_index_ignores_positive_exponent_zero`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_positive_exponent_zero`.
- [x] Number index treats negative decimal boundary as numeric; added `test_resolve_bounds_number_index_accepts_negative_decimal_boundary_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_accepts_negative_decimal_boundary_name`.
- [x] Number index ignores trailing decimal name; added `test_resolve_bounds_number_index_ignores_trailing_decimal_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_trailing_decimal_name`.
- [x] Number index ignores leading plus name; added `test_resolve_bounds_number_index_ignores_leading_plus_name`. Tests: `./wasm/test.sh test_resolve_bounds_number_index_ignores_leading_plus_name`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
