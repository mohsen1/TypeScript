# Worker 5 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 5

## Current Assignment
- Emitter fidelity: validate source map correctness. Add a focused test in `wasm/src/source_map_tests.rs` that checks mappings for a transformed ES5 output, then fix `wasm/src/source_writer.rs` or `wasm/src/source_map.rs` if needed.

## Task Queue
- [ ] If mappings are correct, expand coverage to include async/await downleveling and verify debuggers can attach.

## Completed
- [x] (Move finished items here with brief notes and tests run.)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
