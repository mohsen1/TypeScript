# Post-Merge Test Failures Analysis

**Date:** 2026-01-10
**Branch:** worker/forge-2
**Merge:** origin/rust + origin/squad/forge

## Summary
After merging `origin/rust` and `origin/squad/forge`, the test suite shows **68 failures** (4898 passed).

## Failure Categories

### 1. New Error Checks from Other Workers (~25 failures)
These are legitimate new features that now trigger on existing test code:

- **TS2564** (Property Initialization) - Worker 1
  - `test_new_expression_infers_class_instance_type`
  - `test_new_expression_infers_base_class_properties`
  - `test_interface_extends_class_applies_type_arguments`

- **TS7006/7010/7011** (Implicit Any) - Worker 5
  - `test_generic_library_snippet_compiles_and_checks`
  - `test_overload_call_handles_generic_signatures`
  - `test_redux_pattern_*` (multiple)

**Action Needed:** Update test expectations or fix test code to avoid implicit any.

### 2. Type Parameter Scoping Regressions (~15 failures)
**TS2304 "Cannot find name"** errors on type parameters:

- `test_cross_scope_generic_constraints` - `infer I` not scoped
- `test_tuple_wrapped_conditional_pattern` - `infer U` not scoped
- `test_redux_pattern_extract_state_with_infer` - `infer S` not scoped
- `test_interface_extends_generic_method_compatible` - method type params
- `test_interface_generic_call_signature_uses_type_params`
- `test_interface_generic_construct_signature_uses_type_params`

**Root Cause:** `infer` type parameters in conditional types are not being scoped when checking the true/false branches. This is likely related to commit `a608e2a5ae` which added scoping for mapped types and type aliases but may have missed conditional types.

**Action Needed:** Add `infer` parameter scoping in `check_type_for_missing_names` for conditional types.

### 3. Namespace Merging Broken (~10 failures)
Tests expecting namespace+class/enum/function merging are failing:

- `test_checker_namespace_merges_with_class_exports` - Returns `Any` instead of interface type
- `test_checker_namespace_merges_with_class_value_exports` - Wrong TypeId
- `test_checker_namespace_merges_with_enum_value_exports`
- `test_checker_namespace_merges_with_function_value_exports`
- All reverse-order variants

**Symptom:** `Foo.Bar` resolves to `Any` instead of the expected interface/type.

**Root Cause:** Unknown - something in the merge broke namespace type-side merging. The binder or checker may have regressed.

**Action Needed:** Investigate when/how namespace merging broke.

### 4. Other Type System Issues (~18 failures)

- **Element Access/Lowering** (6 tests) - Optional chaining or type lowering issues
- **Split Accessors** (2 tests) - Getter/setter type checking changed
- **Control Flow** (3 tests) - TS2355 duplicates, TS2454 new errors
- **Type Aliases/Namespaces** (3 tests) - Namespace type member resolution
- **Misc** (4 tests) - Various assignability and type resolution

## Current Status

- **TS2339 Conformance:** Still 0 false positives (1000-file scan)
- **Core Mission:** Unaffected - TS2339 work is stable
- **Merge Impact:** Significant test regressions from squad/forge changes

## Recommendations

1. **Short-term:** Update tests for TS2564/TS7006/TS7010 to get ~25 passing
2. **Medium-term:** Fix `infer` scoping regression (~15 tests)
3. **Critical:** Debug namespace merging regression (~10 tests)
4. **Long-term:** Triage remaining 18 failures

## Next Steps

Need guidance on priority:
- Focus on my TS2339 mission (currently 0 false positives)?
- Fix these regressions before proceeding?
- Document and delegate to other workers?
