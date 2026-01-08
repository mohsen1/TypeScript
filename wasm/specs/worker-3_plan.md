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

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
