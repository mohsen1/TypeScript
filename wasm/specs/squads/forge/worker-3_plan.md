# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 3

## Current Assignment
- [x] Support Worker 2 on `test_check_redux_lodash_style_generics` by isolating which specific generic patterns in `wasm/src/parallel_tests.rs:321-438` produce the 6 diagnostics. Create minimal repro tests in `wasm/src/thin_checker_tests.rs` for each failing pattern.

## Task Queue
- [ ] Once Worker 2 fixes the core issue, verify all minimal repros pass (5 of 6 currently fail).
- [ ] Once constraint property lookup is implemented, update `test_cross_scope_generic_constraints` to expect 0 errors.
- [ ] Once setter type checking is implemented, update `test_split_accessors_write_error` to expect 1 error.
- [ ] Once typeof class types work, update `test_abstract_constructor_assignability` to expect 0 errors.
- [ ] Once class inheritance type checking works, update `test_concrete_extends_abstract` and `test_best_common_type_class_hierarchy` to expect 0 errors.
- [ ] Once namespace-interface value merging works, update `test_namespace_interface_merging` to expect 0 errors.
- [ ] Once enum member access works, update `test_enum_namespace_merging` to expect 0 errors.
- [ ] Pick the next unsoundness case from `wasm/specs/TS_UNSOUNDNESS_CATALOG.md` and add coverage if missing.

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
- [x] Unchecked indexed access (TS unsoundness #8): added thin checker coverage for array element access returning element type without `undefined`. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Apparent members of primitives (TS unsoundness #12): added thin checker coverage for primitive method access via wrapper interfaces. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Void return exception (TS unsoundness #6): added thin checker coverage for assigning a non-void return function to `() => void`. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Distributivity disabling (TS unsoundness #40): added thin checker coverage for `[T] extends [U]` pattern that disables conditional type distribution. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Constructor void exception (TS unsoundness #28): added thin checker coverage for `new () => void` accepting concrete classes. Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Key remapping syntax (TS unsoundness #41): added thin checker coverage for `[P in keyof T as ...]: T[P]` key filtering syntax (Omit, Pick). Tests: `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics` assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
- [x] Redux/Lodash pattern minimal repros: created 6 tests isolating specific patterns (ExtractState infer, StateFromReducers mapped, DeepPartial, createStore generic, ActionFromReducers index, ReducersMapObject). Root cause: generic Application types not expanded. 5/6 fail, 1 passes. Tests: `./wasm/test.sh -- test_redux_pattern`.
- [x] Base constraint assignability (TS unsoundness #31): added 4 tests for generic type parameter constraint checking. Tests cover T <: Constraint(T), rejection of Constraint -> T assignments, param identity checks, and cross-scope constraint property access (currently 3 expected errors until constraint property lookup is implemented). Tests: `./wasm/test.sh -- test_base_constraint\|test_generic_constraint\|test_generic_param\|test_cross_scope`.
- [x] Split accessors (TS unsoundness #26): added 3 tests for getter/setter variance. Tests cover basic accessor usage, read type mismatch errors, and write type mismatch (currently 0 expected errors for write until setter type checking is implemented). Tests: `./wasm/test.sh -- test_split_accessors`.
- [x] Abstract class instantiation (TS unsoundness #43): added 3 tests for abstract class behavior. Tests cover instantiation error, constructor type assignability (4 expected errors until typeof class works), and concrete-to-abstract assignment (3 expected errors until class inheritance works). Tests: `./wasm/test.sh -- test_abstract_class_instantiation\|test_abstract_constructor\|test_concrete_extends`.
- [x] Global Function type (TS unsoundness #29): added 3 tests for untyped callable supertype. Tests cover callable-to-Function assignability, Function-to-specific assignability (with any), and function type hierarchy. Tests: `./wasm/test.sh -- test_global_function_type\|test_function_not_assignable\|test_function_type_hierarchy`.
- [x] Best Common Type inference (TS unsoundness #32): added 3 tests for array literal type inference. Tests cover mixed array literals, class hierarchy (1 expected error until class inheritance works), and literal widening. Tests: `./wasm/test.sh -- test_best_common_type`.
- [x] Module Augmentation Merging (TS unsoundness #44): added 6 tests for declaration merging. Tests cover interface merging, method overloads, extend+merge, namespace-interface merging (2 expected errors), class-namespace merging, and enum-namespace merging (4 expected errors). Tests: `./wasm/test.sh -- test_interface_merging\|test_namespace_interface\|test_class_namespace\|test_enum_namespace`.

## Ready for Merge
Yes

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-3`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
- Latest `./wasm/test.sh` fails at `parallel::tests::test_check_redux_lodash_style_generics` (assertion left 6 right 0 at `wasm/src/parallel_tests.rs:437:5`).
