# Worker 4 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 4

## Current Assignment
- Ask manager for next assignment (ES5 class async method parity handled).

## Task Queue
- [ ] Stand by for next emitter fidelity task.

## Completed
- [x] Async ES5 parity investigation: parity test passed; fixed nested arrow `this` capture in async ES5 emission; tests `./wasm/test.sh test_parity_async_es5`, `./wasm/test.sh nested_arrow`.
- [x] ES5 downleveling edge cases: fixed default-constructor arrow `this` capture and preserved pre-super statements with property initializer ordering; tests `./wasm/test.sh class_es5`.
- [x] ES5 class parity gap: legacy emitter now downlevels classes when targeting ES5; test `./wasm/test.sh test_parity_es5_class`.
- [x] Re-verified async ES5 parity after cleanup; test `./wasm/test.sh test_parity_async_es5`.
- [x] ES5 class async method parity: emit __awaiter wrapper in class methods; test `./wasm/test.sh class_es5`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
