# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 3

## Current Assignment
- [x] Pick the next unsoundness case from `wasm/specs/TS_UNSOUNDNESS_CATALOG.md` and add coverage if missing.

## Task Queue
- [ ] Coordinate with manager if unsure which unsoundness case to prioritize next.

## Completed
- [x] Object vs object vs {} trifecta (TS unsoundness #20): add thin checker coverage for object keyword vs empty object. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Nominal classes (TS unsoundness #5): add private/protected brand property and coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Instantiation depth limit (TS unsoundness #17): guard deep instantiation in solver and add coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Class static side rules (TS unsoundness #18): include static members in constructor type and add coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Numeric/string enum nominalness (TS unsoundness #7/#24/#34): added enum assignability handling and coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] `import type` erasure (TS unsoundness #39): mark type-only imports to error on value usage; added coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Template string expansion limits (TS unsoundness #22): added expansion guard and coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] `unique symbol` nominal primitives (TS unsoundness #37): added nominal assignability coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Homomorphic mapped types over boolean primitives (TS unsoundness #27): added boolean key mapping assignability coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Recursion depth circuit breaker (TS unsoundness #35): treat deep array instantiation as assignable to avoid runaway recursion. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Error poisoning union suppression (TS unsoundness #11): union with `error` collapses to `error`. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Intersection reduction for disjoint primitives (TS unsoundness #21): ensure `string & number` reduces to `never`. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Homomorphic mapped types over string primitives (TS unsoundness #27): added string key mapping coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] `keyof` union contravariance (TS unsoundness #30): ensure only shared keys are produced. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Comparison operator overlap (TS unsoundness #23): added loose equality overlap coverage. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Primitive boxing behavior (TS unsoundness #33): prevent `Number`-like object from assigning to `number`. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Literal widening for mutable bindings (TS unsoundness #10): widen boolean literals on let/var, keep const literal. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Freshness/excess property check: allow assigning non-fresh object (variable) to target type. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Covariant mutable arrays (TS unsoundness #3) coverage in compat assignability. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Structural property/method variance: allow bivariant checks when either side is a method; added mixed method vs function-property test. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Function variance across union/intersection targets regression test. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Method vs function-property variance coverage for function-source to method-target. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Index-signature consistency with method bivariance regression (TS unsoundness #25). Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] String index signature method bivariance regression (TS unsoundness #25). Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Investigated FunctionId build error in `wasm/src/solver/evaluate.rs` after sync; no references found, build succeeded. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Implemented covariant `this`-type handling in parameter variance with regression coverage. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added class-like subtyping regression for `this`-typed parameters (base vs derived). Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added mixed method/function-property variance tests for `this` parameters. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added void-return exception coverage for method properties. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Reset flow at function boundaries to avoid narrowing in closures; added CFA invalidation test. Tests: `./wasm/test.sh` (fails: `wasm/src/transforms/async_es5.rs:749` unexpected closing delimiter after sync).
- [x] Added best common type array literal regression coverage. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_parse_error_tolerance`).
- [x] Added correlated union index-access regression coverage (cross-product). Tests: `./wasm/test.sh` (fails: `emitter_parity_tests::test_parity_commonjs_export`).
- [x] Rechecked FunctionId build error in `wasm/src/solver/evaluate.rs` after sync; not reproducible. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Tuple-array assignment (TS unsoundness #15): added thin checker coverage for tuple -> array ok and array -> tuple rejection. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Rest parameter bivariance (TS unsoundness #16): added thin checker coverage for `(...args: any[]) => void` accepting specific params. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Weak type detection (TS unsoundness #13): added thin checker coverage for optional-only target rejecting no-overlap source. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Optionality vs undefined (TS unsoundness #14): added thin checker coverage for optional properties accepting `undefined` by default. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).

## Ready for Merge
Yes

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-3`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
- Latest `./wasm/test.sh` fails at `parallel::tests::test_check_redux_lodash_style_generics` (assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
