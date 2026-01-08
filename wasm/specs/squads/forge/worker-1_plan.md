# Worker 1 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 1

## Current Assignment
- [ ] Redux test (`test_check_redux_lodash_style_generics`) has 3 remaining diagnostics:
  - store.ts:716 - Object literal not assignable to Store<StateFromReducer<R>, ActionFromReducer<R>> where conditional types contain type param R
  - app.ts:375 (x2) - Action type inference with nested mapped/conditional types
  - Root cause investigation complete:
    1. Conditional types with type params ARE being deferred correctly (lines 386-392 in evaluate.rs)
    2. Application types ARE being expanded correctly (debug confirmed TypeId(177) -> TypeId(236))
    3. The issue is in comparing function signatures where return types are deferred conditionals
    4. Source: `getState: () => R extends Reducer<infer S, AnyAction> ? S : never`
    5. Target after Store expansion: `getState: () => StateFromReducer<R>` (which should evaluate to same conditional)
    6. Both should be equivalent but the comparison is failing somewhere in the structural Object comparison
  - Next steps: Add debug to Object subtype comparison to see which property is failing

## Task Queue
- [ ] Add callable-parameter inference regressions (e.g., union inputs, overload shapes) in `wasm/src/solver/evaluate_tests.rs`.
- [ ] Validate function return inference in conditional types and fix any mismatches in `wasm/src/solver/evaluate.rs`.

## Completed
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
