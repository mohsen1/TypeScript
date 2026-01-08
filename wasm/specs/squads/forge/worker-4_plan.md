# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 4

## Current Assignment
- [x] Add solver tests for type operations (applications, tuples, arrays, objects, keyof, infer, instantiation, primitives). **Done:** Added 38 tests.

## Task Queue
(empty - awaiting next assignment)

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
- [x] Added solver coverage for Exclude utility type pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Extract utility type pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for NonNullable utility type pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for ReturnType utility type pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Parameters utility type pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for ConstructorParameters utility type pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for InstanceType utility type pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Parameters with optional params. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Parameters with rest params. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Awaited utility type pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Awaited with non-Promise (returns T). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for ThisParameterType utility type pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for ThisParameterType with no this (returns unknown). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for OmitThisParameter utility type pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for OmitThisParameter with no this. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Partial<T> pattern with keyof and index access. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Readonly<T> pattern with keyof and index access. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for -readonly modifier (Mutable pattern). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for combined modifiers (-readonly -?). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for combined modifiers (+readonly +?). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Record<K, V> with string literal keys. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Record with template value (key as value). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Record with single key. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Record with index signature (string key). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Readonly Record combination. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Partial Record combination. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal simple concatenation. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal prefix/suffix patterns. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal in mapped type key remapping. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal union distribution. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for NoInfer<T> identity pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for NoInfer<T> in function params. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Uppercase<T> intrinsic pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Lowercase<T> intrinsic pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Capitalize<T> intrinsic pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for Uncapitalize<T> intrinsic pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for chained string intrinsics. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal Head/Tail split pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal three-segment split. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal prefix extraction. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal suffix extraction. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal middle extraction. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal path split pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal union distribution with infer. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal kebab-to-camel pattern. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for template literal dot notation parse. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for recursive conditional types (Flatten, Awaited, DeepReadonly patterns). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for distributive conditional with mapped type interactions (FunctionKeys, PickByValue, Getters/Setters, NestedKeyOf patterns). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for variadic tuple types (spread, concat, push/pop, shift/unshift, labeled rest, infer patterns). Tests: `./wasm/test.sh`.
- [x] Added solver coverage for default type params, index access, keyof union/intersection, homomorphic mapped, conditional edge cases. Tests: `./wasm/test.sh`.
- [x] Added solver coverage for mapped type edge cases (homomorphic modifiers, key remapping with template literals, intrinsics, conditionals). Tests: `./wasm/test.sh`.
- [x] Added solver edge case tests (intersections with never/unknown/any, union normalization, generic constraints, nested conditionals, function subtyping, index access, literal types). Tests: `./wasm/test.sh`.
- [x] Added solver tests for type operations (type applications, tuples with labels/optional/rest, arrays, objects with index signatures, keyof primitives, infer types, type instantiation, primitives). Tests: `./wasm/test.sh`.

## Ready for Merge
Yes

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
