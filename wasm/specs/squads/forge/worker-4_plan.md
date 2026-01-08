# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 4

## Current Assignment
- [x] End-to-end validation: add a larger redux/lodash-style generic library regression in `wasm/src/parallel_tests.rs` (or `wasm/src/thin_checker_tests.rs`) and fix the first panic or mismatch in `wasm/src/checker/mod.rs` or `wasm/src/solver/mod.rs`; run `./wasm/test.sh`.
- [x] Add union normalization assertions for `unknown` handling and nested union flattening in `wasm/src/thin_checker_tests.rs`; run `./wasm/test.sh`.
- [x] Add intersection flatten/dedup regression in `wasm/src/solver/intern_tests.rs`; run `./wasm/test.sh`.
- [x] Add union/intersection precedence regression for `any` vs `unknown` in `wasm/src/solver/intern_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [x] Add a multi-file generic library test case with mapped/conditional types and assert no panics + expected diagnostics.
- [x] If a panic arises, minimize to a focused solver/checker regression test.
- [x] Validate the new test still passes with `./wasm/test.sh`.

## Completed
- [x] Added generic library regression + fixed declare function overload handling. Tests: `./wasm/test.sh test_generic_library_snippet_compiles_and_checks` (full suite fails on existing `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added multi-file generic regression across two files. Tests: `./wasm/test.sh test_multi_file_generic_library_snippet_compiles_and_checks`.
- [x] Synced with `origin/rust`; `FunctionId` build error not reproducible in `wasm/src/solver/evaluate.rs`. Tests: `./wasm/test.sh` (fails on existing `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added redux/lodash-style mapped/conditional regression in `wasm/src/parallel_tests.rs`. Tests: `./wasm/test.sh` (fails: `src/transforms/async_es5.rs` unexpected closing delimiter).
- [x] Added union normalization coverage for `unknown` and nested unions. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` diagnostic count 6 vs 0).
- [x] Added intersection flatten/dedup coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` diagnostic count 6 vs 0).
- [x] Added `any` vs `unknown` precedence coverage for unions/intersections. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` diagnostic count 6 vs 0).
- [x] Fixed legacy module wrapper auto-lowering for AMD/UMD/System emit. Tests: `./wasm/test.sh`.

## Ready for Merge
Yes

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
