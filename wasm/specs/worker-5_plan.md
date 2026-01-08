# Worker 5 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 5

## Current Assignment
- Emitter fidelity: own the fix for `emitter_parity_tests::test_parity_async_es5`. Run targeted test, capture actual vs expected output, implement minimal fix, and rerun.

## Task Queue
- [ ] If parity_async_es5 is resolved, rerun emitter-focused tests and report remaining failures.

## Completed
- [x] (Move finished items here with brief notes and tests run.)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
