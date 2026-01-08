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

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
