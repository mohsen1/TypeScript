# Worker 4 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 4

## Current Assignment
- Identify any additional ES5 class emission parity gaps after the edge-case checks.

## Task Queue
- [ ] Ask manager for next assignment if no new parity gaps are found.

## Completed
- [x] Async ES5 parity investigation: parity test passed; fixed nested arrow `this` capture in async ES5 emission; tests `./wasm/test.sh test_parity_async_es5`, `./wasm/test.sh nested_arrow`.
- [x] ES5 downleveling edge cases: fixed default-constructor arrow `this` capture and preserved pre-super statements with property initializer ordering; tests `./wasm/test.sh class_es5`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
