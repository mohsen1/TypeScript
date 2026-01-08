# Worker 3 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 3

## Current Assignment
- Edge cases for optional/rest parameters and `this` parameters in assignability.

## Task Queue
- [ ] (Add more tasks as assigned.)

## Completed
- [x] Solver hardening: added callable rest contravariance + return covariance tests; aligned rest parameter variance in call signature checks. Ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_class_extends_helper`).

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
