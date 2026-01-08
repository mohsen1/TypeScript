# Worker 4 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 4

## Current Assignment
- Stand by for next emitter fidelity task.

## Task Queue
- [ ] Stand by for next emitter fidelity task.

## Completed
- [x] Async ES5 variable statement emission: handle declaration lists so const/let initializers emit in async bodies; test `./wasm/test.sh nested_arrow_this_capture`.
- [x] Async ES5 parity investigation: parity test passed; fixed nested arrow `this` capture in async ES5 emission; tests `./wasm/test.sh test_parity_async_es5`, `./wasm/test.sh nested_arrow`.
- [x] Async ES5 await detection traversal: include property/element/conditional expressions; tests `./wasm/test.sh async_es5`.
- [x] ES5 downleveling edge cases: fixed default-constructor arrow `this` capture and preserved pre-super statements with property initializer ordering; tests `./wasm/test.sh class_es5`.
- [x] ES5 class parity gap: legacy emitter now downlevels classes when targeting ES5; test `./wasm/test.sh test_parity_es5_class`.
- [x] Re-verified async ES5 parity after var-statement fix; test `./wasm/test.sh test_parity_async_es5`.
- [x] ES5 class async method parity: emit __awaiter wrapper in class methods; test `./wasm/test.sh class_es5`.
- [x] ES5 class edge case: preserve pre-super statements before initializer emission in derived constructors; test `./wasm/test.sh class_es5`.
- [x] Emitter extends helper edge case: route test through LoweringPass transforms; test `./wasm/test.sh test_class_extends_helper`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
