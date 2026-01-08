# Worker 4 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 4

## Current Assignment
- Emitter fidelity: summarize current `async_es5` changes and whether they address `emitter_parity_tests::test_parity_async_es5`; run targeted test and propose next minimal change if needed. Coordinate with worker-5.

## Task Queue
- [ ] If parity_async_es5 is fixed, re-verify ES5 downleveling edge cases for `this` capture and super() property initializer ordering.

## Completed
- [x] (Move finished items here with brief notes and tests run.)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
