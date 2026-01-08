# Worker 1 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 1

## Current Assignment
- Awaiting next manager assignment.

## Task Queue
- [ ] If contextual inference is already correct, target circular `extends` constraints and add a regression test.

## Completed
- [x] Hardened function bound checks to respect `this` types; added `test_resolve_bounds_function_this_type_mismatch` and updated inference subtyping. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_class_extends_helper`).
- [x] Added occurs-check coverage for function/callable `this` types with `test_inference_occurs_check_function_this_type`. Tests: `./wasm/test.sh test_inference_occurs_check_function_this_type`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
