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
- [x] Added ES5 async try/catch/finally awaits source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_catch_finally_awaits_mapping`.
- [x] Added ES5 async for-in source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_for_in_mapping`.
- [x] Added ES5 async try/catch await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_catch_await_mapping`.
- [x] Added ES5 async if/else await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_if_else_mapping`.
- [x] Added ES5 async ternary await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_ternary_mapping`.
- [x] Added ES5 async switch default await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_switch_default_await_mapping`.
- [x] Added ES5 async try/finally await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_finally_await_mapping`.
- [x] Added ES5 async try/finally return await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_finally_return_mapping`.
- [x] Added ES5 async switch case await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_switch_case_await_mapping`.
- [x] Added ES5 async try/catch return await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_catch_return_await_mapping`.
- [x] Added ES5 async try/catch throw await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_catch_throw_await_mapping`.
- [x] Added ES5 async switch fallthrough await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_switch_fallthrough_await_mapping`.
- [x] Added ES5 async switch await discriminant source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_switch_await_discriminant_mapping`.
- [x] Added ES5 async try/catch await-only source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_catch_only_await_mapping`.
- [x] Added ES5 async switch default-only await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_switch_default_only_await_mapping`.
- [x] Added ES5 async try/catch/finally await-only source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_catch_finally_only_mapping`.
- [x] Added ES5 async try/finally await-only source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_finally_only_await_mapping`.
- [x] Added ES5 async switch return await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_switch_return_await_mapping`.
- [x] Added ES5 async try/catch return await-in-try source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_try_catch_return_await_in_try_mapping`.
- [x] Added ES5 async logical-and await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_logical_and_mapping`.
- [x] Added ES5 async logical-or await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_logical_or_mapping`.
- [x] Added ES5 async if await-condition source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_if_await_condition_mapping`.
- [x] Added ES5 async if await-and source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_if_await_and_mapping`.
- [x] Added ES5 async while await-condition source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_while_await_condition_mapping`.
- [x] Added ES5 async do/while await-condition source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_do_while_await_condition_mapping`.
- [x] Added ES5 async for-loop await-condition source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_for_loop_await_condition_mapping`.
- [x] Added ES5 async for-loop await-initializer source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_for_loop_await_initializer_mapping`.
- [x] Added ES5 async for-loop await-update source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_for_loop_await_update_mapping`.
- [x] Added ES5 async for-of await-RHS source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_for_of_await_rhs_mapping`.
- [x] Added ES5 async for-in await-RHS source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_for_in_await_rhs_mapping`.
- [x] Added ES5 async array literal await source map mapping test in `wasm/src/source_map_tests.rs`.
      Tests: `./wasm/test.sh test_source_map_es5_transform_async_array_literal_mapping`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
- Re-ran `./wasm/test.sh test_source_map_es5_transform_async_await_property_access_mapping`.
- Re-ran `./wasm/test.sh test_source_map_es5_transform_async_await_conditional_mapping`.
- Re-ran `./wasm/test.sh test_source_map_es5_transform_async_arrow_captures_this_mapping`.
