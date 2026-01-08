# Worker 2 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 2

## Current Assignment
- (awaiting next assignment)

## Task Queue
- [ ] (none)

## Completed
- [x] Solver hardening: added infer/distributive conditional matching in `wasm/src/solver/evaluate.rs`, plus nested/distributive infer tests in `wasm/src/solver/evaluate_tests.rs`. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_class_extends_helper`).
- [x] Conditional infer object coverage: added object property/index signature infer matching in `wasm/src/solver/evaluate.rs` and tests in `wasm/src/solver/evaluate_tests.rs`. Tests: `./wasm/test.sh test_conditional_infer_object_`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
