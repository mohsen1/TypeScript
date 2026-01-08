# Worker 1 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 1

## Current Assignment
- Emitter fidelity: investigate `emitter_parity_tests::test_parity_async_es5`. Run targeted test, capture actual vs expected output, and propose the minimal fix (test or code). Coordinate with worker-4/5.

## Task Queue
- [ ] If parity_async_es5 fix is unclear, isolate the failing snippet and report minimal repro with expected/actual output.

## Completed
- [x] (Move finished items here with brief notes and tests run.)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
