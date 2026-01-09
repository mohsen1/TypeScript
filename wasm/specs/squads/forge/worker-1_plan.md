# Worker 1 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 1

## Current Assignment
- [ ] Redux test (`test_check_redux_lodash_style_generics`) has 2 remaining diagnostics:
  - store.ts:590 - Object literal with replaceState not assignable to Store type
  - app.ts:508 - Property 'tags' does not exist on type 'S'
  - Progress made:
    1. Fixed type param extraction for Mapped types in `collect_type_params`
       - Removed incorrect addition of `mapped.type_param` (iteration var K)
       - Added `TypeKey::KeyOf` handling to extract operand type param (T from keyof T)
       - Added `TypeKey::IndexAccess` handling to extract both obj and idx type params
    2. Reduced diagnostics from 3 to 2
  - Remaining issue: replaceState param type comparison still fails
  - Analysis of remaining issues:
    1. store.ts:590: `DeepPartial<StateFromReducer<R>>` param not matching Store interface
    2. app.ts:508: State type 'S' not being resolved to RootState (missing 'tags' property)
  - Root cause: Conditional type `StateFromReducer<R>` not fully evaluating through the generic chain
  - Next steps: Debug conditional type evaluation in `evaluate_conditional`

## Task Queue
- [ ] Validate function return inference in conditional types and fix any mismatches in `wasm/src/solver/evaluate.rs`.

## Completed
- [x] Added callable-parameter inference regression tests: union of signatures, overloaded callable, mixed union, param+return extraction, multiple params. All tests pass.
- [x] Fixed type param extraction for Mapped types: removed iteration var (K), added KeyOf/IndexAccess handlers. Reduced redux diagnostics from 3 to 2.
- [x] Added 5 template literal hyphen pattern tests for type inference (prefix/suffix extraction, two-part extraction, distributive union, no-match returns never).
- [x] Application type expansion in TypeEvaluator with fallback extraction of type params from resolved Object properties (reduced redux test from 4 to 3 diagnostics).
- [x] Fixed type predicate alias narrowing (`test_user_defined_type_predicate_alias_narrows` passes).
- [x] Covered function optional/rest parameter inference in conditional types (distributive + non-distributive) in `wasm/src/solver/evaluate_tests.rs`. Ran `./wasm/test.sh` (fails: parallel::tests::test_check_redux_lodash_style_generics).
- [x] Conditional type evaluation: implement function parameter/return inference in `wasm/src/solver/evaluate.rs`; updated regressions in `wasm/src/solver/evaluate_tests.rs`. Ran `./wasm/test.sh` (fails: parallel::tests::test_check_redux_lodash_style_generics).
- [x] Solver inference hardening: add cyclic upper bound expansion + usage-based inference tests. Ran `./wasm/test.sh` (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).
- [x] Added contextual signature bounds tests for function parameter/return variance. Ran `./wasm/test.sh` (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).
- [x] Added circular upper-bound order regression test. Ran `./wasm/test.sh` (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).
- [x] Added union target placeholder inference test. Ran `./wasm/test.sh` (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).
- [x] Fix FunctionId build error in `wasm/src/solver/evaluate.rs`; ran `./wasm/test.sh` (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).
- [x] Checked `wasm/src/solver/evaluate.rs` for FunctionId build error (not reproducible after sync). Ran `./wasm/test.sh` (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).
- [x] Re-ran `./wasm/test.sh`; FunctionId build error still not reproducible (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).
- [x] Re-ran `./wasm/test.sh` (FAIL: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports). FunctionId build error not reproducible after sync.

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-1`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
