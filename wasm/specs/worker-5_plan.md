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
- [x] Added ES5 async/await property access mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_await_property_access_mapping`.
- [x] Added ES5 async arrow function source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_arrow_mapping`.
- [x] Added ES5 async class method source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_class_method_mapping`.
- [x] Added ES5 async conditional await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_await_conditional_mapping`.
- [x] Added ES5 async function capturing `this` source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_arrow_captures_this_mapping`.
- [x] Added ES5 async try/catch source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_catch_mapping`.
- [x] Added ES5 async try/finally source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_finally_mapping`.
- [x] Added ES5 async try/catch/finally source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_catch_finally_mapping`.
- [x] Added ES5 async nested arrow capture source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_nested_arrow_capture_mapping`.
- [x] Added ES5 async try/catch nested arrow source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_catch_nested_arrow_mapping`.
- [x] Added ES5 async object literal arrow source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_object_literal_arrow_mapping`.
- [x] Added ES5 async switch source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_switch_mapping`.
- [x] Added ES5 async for-loop source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_for_loop_mapping`.
- [x] Added ES5 async while-loop source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_while_loop_mapping`.
- [x] Added ES5 async do/while source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_do_while_mapping`.
- [x] Added ES5 async for-of source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_for_of_mapping`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
- Re-ran `./wasm/test.sh test_source_map_es5_transform_async_await_property_access_mapping`.
- Re-ran `./wasm/test.sh test_source_map_es5_transform_async_await_conditional_mapping`.
- Re-ran `./wasm/test.sh test_source_map_es5_transform_async_arrow_captures_this_mapping`.
