# Worker 3 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 3

## Current Assignment
- Emitter fidelity: run `emitter_parity_tests::test_parity_async_es5` in your worktree with `--nocapture` if possible and report the actual output/diff versus expected (no code edits yet).

## Task Queue
- [ ] If the diff is noisy, trim to the smallest failing snippet and highlight the first semantic mismatch.

## Completed
- [x] (Move finished items here with brief notes and tests run.)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
