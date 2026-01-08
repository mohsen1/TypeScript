# Worker 2 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 2

## Current Assignment
- Emitter fidelity: analyze expected output for `emitter_parity_tests::test_parity_async_es5` by reading the test and any golden helpers. Report expected output/invariants (no code edits).

## Task Queue
- [ ] If the expected output is underspecified, propose what a stable expected output should assert.

## Completed
- [x] (Move finished items here with brief notes and tests run.)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
