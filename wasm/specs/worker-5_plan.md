# Worker 5 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 5

## Current Assignment
- Await next manager assignment.

## Task Queue
- [ ] (empty)

## Completed
- [x] Added ES5 async/await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_await_mapping`.
- [x] Added ES5 class extends source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_class_extends_mapping`.
- [x] Added ES5 class property initializer source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_class_property_initializer_mapping`.
- [x] Added ES5 derived class constructor super + initializer source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_derived_ctor_super_initializer_mapping`.
- [x] Added ES5 async/await return mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_await_return_mapping`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
