# Worker 5 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 5

## Current Assignment
- [ ] [EM: Add queued tasks]

## Task Queue
- [x] Add tests for function `this`-parameter inference (contextual typing + call-site inference) in `wasm/src/solver/infer_tests.rs`.
- [x] Convert TODOs in `wasm/src/solver/evaluate_tests.rs` for optional property inference (missing vs `undefined`) and optional tuple element inference (undefined inclusion).
- [ ] [EM: Add queued tasks]

## Completed
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
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-5`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
