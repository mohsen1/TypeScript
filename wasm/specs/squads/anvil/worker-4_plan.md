# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 4

## Current Assignment
- [ ] Awaiting EM assignment.

## Task Queue
- [ ] [EM: Add queued tasks]

## Completed
- [x] Add async/class ES5 transform source-map mappings and offsets. Tests: `./wasm/test.sh source_map`
- [x] Add async nested function source-map offset coverage. Tests: `./wasm/test.sh source_map`
- [x] Add async await detection test for nested functions. Tests: `./wasm/test.sh body_contains_await`
- [x] Add ES5 derived default constructor ordering test. Tests: `./wasm/test.sh default_derived_constructor`
- [x] Add ES5 derived constructor ordering test (super/field/body). Tests: `./wasm/test.sh` (fails in `parallel::tests::test_check_redux_lodash_style_generics`)
- [x] Add async await initializer assignment coverage. Tests: `./wasm/test.sh async_es5`
- [x] Restore async ES5 emitter this-capture setter for new call sites. Tests: `./wasm/test.sh async_es5`
- [x] Add async ES5 this-capture test. Tests: `./wasm/test.sh async_es5`
- [x] Add derived ctor pre-super ordering test with field init. Tests: `./wasm/test.sh derived_constructor_preserves_pre_super`
- [x] Add async await var initializer source-map coverage. Tests: `./wasm/test.sh async_await_var_initializer_mapping`
- [x] Add async await call property source-map coverage. Tests: `./wasm/test.sh async_await_call_property_mapping`
- [x] Add async await element access source-map coverage. Tests: `./wasm/test.sh async_await_element_access_mapping`
- [x] Add async await call source-map coverage. Tests: `./wasm/test.sh async_await_call_mapping`
- [x] Add async await call argument source-map coverage. Tests: `./wasm/test.sh async_await_call_argument_mapping`
- [x] Add async await detection for call arguments. Tests: `./wasm/test.sh body_contains_await_in_call_argument`
- [x] Add async await detection for call expression callee. Tests: `./wasm/test.sh body_contains_await_in_call_expression_callee`
- [x] Add async await detection for binary expressions. Tests: `./wasm/test.sh body_contains_await_in_binary_expression`
- [x] Add async await detection for unary expressions. Tests: `./wasm/test.sh body_contains_await_in_unary_expression`
- [x] Add derived ctor param property ordering test. Tests: `./wasm/test.sh derived_constructor_orders_param_property`
- [x] Add derived ctor private field ordering test. Tests: `./wasm/test.sh derived_constructor_orders_private`
- [x] Add async await property name source-map coverage. Tests: `./wasm/test.sh async_await_property_name_mapping`

## Ready for Merge
Yes

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
- `./wasm/test.sh` currently fails on `parallel::tests::test_check_redux_lodash_style_generics` (left 6, right 0).
- Proposed next tasks for EM assignment:
  - Validate ES5 class downleveling edge cases for `super()` + field initializers in `wasm/src/transforms/class_es5.rs`.
  - Add coverage for async downlevel source-map offsets in `wasm/src/transforms/async_es5.rs`.
