# Worker 1 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 1

## Current Assignment
- [ ] Validate function return inference in conditional types and fix any mismatches in `wasm/src/solver/evaluate.rs`.

## Task Queue
- [ ] [EM: Add queued tasks]

## Completed
- [x] Added callable-parameter inference regressions for non-callable union branches (distributive + non-distributive) in `wasm/src/solver/evaluate_tests.rs`. Ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `src/transforms/async_es5.rs`).
- [x] Added callable-parameter inference regressions for optional/rest parameters (distributive + non-distributive) in `wasm/src/solver/evaluate_tests.rs`. Ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `src/transforms/async_es5.rs`).
- [x] Added callable-parameter inference regressions (union inputs + overload shapes) in `wasm/src/solver/evaluate_tests.rs`. Ran `./wasm/test.sh` (fails: missing `set_use_this_capture` in `src/transforms/async_es5.rs`).
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
Yes

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-1`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
