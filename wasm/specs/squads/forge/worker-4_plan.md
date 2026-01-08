# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 4

## Current Assignment
- [x] Investigate `DeepPartial<T>` and `PickValue<T, V>` mapped type patterns from `test_check_redux_lodash_style_generics`. Test these patterns in isolation in `wasm/src/solver/evaluate_tests.rs` to identify if they contribute to the 6 diagnostics. **Result:** The 6 diagnostics issue is now fixed (test passes with 0 diagnostics). Added solver tests for both patterns.

## Task Queue
- [x] Add additional end-to-end tests for other lodash-style utility types (Omit, Pick, Required, Readonly). **Done:** Added Required and Pick pattern tests. Omit covered by existing `test_mapped_type_key_remap_filters_keys`. Readonly covered by existing `test_mapped_type_with_readonly_modifier`.
- [x] Verify mapped type + conditional type nesting works correctly. **Done:** Added `test_mapped_type_with_nested_conditionals`.

## Completed
- [x] Added generic library regression + fixed declare function overload handling. Tests: `./wasm/test.sh test_generic_library_snippet_compiles_and_checks` (full suite fails on existing `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added multi-file generic regression across two files. Tests: `./wasm/test.sh test_multi_file_generic_library_snippet_compiles_and_checks`.
- [x] Synced with `origin/rust`; `FunctionId` build error not reproducible in `wasm/src/solver/evaluate.rs`. Tests: `./wasm/test.sh` (fails on existing `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added redux/lodash-style mapped/conditional regression in `wasm/src/parallel_tests.rs`. Tests: `./wasm/test.sh` (fails: `src/transforms/async_es5.rs` unexpected closing delimiter).
- [x] Added union normalization coverage for `unknown` and nested unions. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` diagnostic count 6 vs 0).
- [x] Added intersection flatten/dedup coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` diagnostic count 6 vs 0).
- [x] Added `any` vs `unknown` precedence coverage for unions/intersections. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` diagnostic count 6 vs 0).
- [x] Fixed legacy module wrapper auto-lowering for AMD/UMD/System emit. Tests: `./wasm/test.sh`.
- [x] Added union flatten/dedup regression for interner normalization. Tests: `./wasm/test.sh`.
- [x] Added function/rest subtyping regression for required params accepting undefined. Tests: `./wasm/test.sh`.
- [x] Added binder coverage for namespace export merges across declarations. Tests: `./wasm/test.sh`.
- [x] Added binder coverage for class/namespace merge exports. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for class/namespace merged value member access. Tests: `./wasm/test.sh`.
- [x] Added binder coverage for function/namespace merge exports. Tests: `./wasm/test.sh`.
- [x] Added binder coverage for enum/namespace merge exports. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for enum/namespace merged value member access. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for enum/namespace merged type member access. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for function/namespace merged value member access. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for function/namespace merged type member access. Tests: `./wasm/test.sh`.
- [x] Added binder coverage for namespace/enum merge reverse order. Tests: `./wasm/test.sh`.
- [x] Added binder coverage for namespace/function merge reverse order. Tests: `./wasm/test.sh`.
- [x] Added binder coverage for namespace/class merge reverse order. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for class/namespace merged type member access reverse order. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for class/namespace merged value member access reverse order. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for function/namespace merged value member access reverse order. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for enum/namespace merged value member access reverse order. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for enum/namespace merged type member access reverse order. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for function/namespace merged type member access reverse order. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for namespace merge across declarations with value access. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for namespace merge across declarations with type access. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for typeof namespace alias member type queries. Tests: `./wasm/test.sh`.
- [x] Added checker coverage for class/namespace merged element access. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for mapped type with conditional value filter (PickValue pattern). Tests: `./wasm/test.sh`.
- [x] Investigated DeepPartial/PickValue patterns - 6 diagnostics issue is now fixed. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for mapped type with optional modifier and conditional (DeepPartial pattern). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Required utility type pattern (removes optional modifier). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Pick utility type pattern (subset key iteration). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for mapped type with nested conditionals. Tests: `./wasm/test.sh`.

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
