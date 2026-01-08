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
- [x] Solver inference: include function/callable `this` types in occurs-checks and add `test_inference_occurs_check_function_this_type`. Tests: `./wasm/test.sh test_inference_occurs_check_function_this_type`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
