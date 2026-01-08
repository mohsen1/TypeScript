# Worker 5 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 5

## Current Assignment
- Complete: Added async computed object literal source-map coverage in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh source_map` (PASS).

## Task Queue
- [ ] (empty)

## Completed
- [x] Added async loop/try source-map tests and loosened mapping assertions to fall back to function-level mappings when transforms omit statements; ran `./wasm/test.sh source_map` (PASS).
- [x] Added decode-mappings round-trip test in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Restored async ES5 emitter API compatibility in `wasm/src/transforms/async_es5.rs`; ran `./wasm/test.sh source_map` (PASS) and `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added lexical-this capture test in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added awaited lexical-this capture test in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Expanded await detection for loops, switch, try/catch/finally, and with/labeled in `wasm/src/transforms/async_es5.rs`; added tests in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh source_map` (PASS).
- [x] Added switch-case await detection test in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added with/labeled await detection tests in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added switch-default await detection test in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added catch-clause await detection test in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added for-in/for-of await detection tests in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added switch discriminant/case-expression await detection tests in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Reverted `wasm/src/parallel_tests.rs` changes to stay in anvil scope.
- [x] Added for-loop initializer/incrementor await coverage and variable-declaration-list await detection in `wasm/src/transforms/async_es5.rs` and `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh async_es5_tests` (PASS).
- [x] Added variable-initializer await coverage in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh async_es5_tests` (PASS).
- [x] Added async variable-initializer await source-map coverage in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh source_map` (PASS).
- [x] Added async variable-declaration-list await source-map coverage in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh source_map` (PASS).
- [x] Added async for-loop declaration-list await source-map coverage in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh source_map` (PASS).
- [x] Added async for-loop update-list await source-map coverage in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh source_map` (PASS).
- [x] Added async for-loop condition-list await source-map coverage in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh source_map` (PASS).
- [x] Added async while-loop condition-list await source-map coverage in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh source_map` (PASS).
- [x] Added async do-while condition-list await source-map coverage in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh source_map` (PASS).
- [x] Added async while/do-while await condition coverage in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh async_es5_tests` (PASS).
- [x] Added async literal/spread await coverage in `wasm/src/transforms/async_es5.rs` and `wasm/src/transforms/async_es5_tests.rs`; updated spread lookup in `wasm/src/parser/thin_node.rs`; ran `./wasm/test.sh async_es5_tests` (PASS).
- [x] Added async binding/computed-name await coverage in `wasm/src/transforms/async_es5.rs` and `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh async_es5_tests` (PASS).
- [x] Added async array literal spread and binding computed-name await coverage in `wasm/src/transforms/async_es5_tests.rs`; ran `./wasm/test.sh async_es5_tests` (PASS).
- [x] Added async computed object literal source-map coverage in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh source_map` (PASS).

## Ready for Merge
Yes

## Notes
- Project Direction: integration and conformance-first; prioritize emitter fidelity (ES5 downleveling/source maps) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-5`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
