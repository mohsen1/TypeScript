# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 3

## Current Assignment
- [ ] Pick another unsoundness case from `wasm/specs/TS_UNSOUNDNESS_CATALOG.md` and add coverage if missing.

## Task Queue
- [ ] Coordinate with manager if unsure which unsoundness case to prioritize next.

## Completed
- [x] Structural property/method variance: allow bivariant checks when either side is a method; added mixed method vs function-property test. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Function variance across union/intersection targets regression test. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Method vs function-property variance coverage for function-source to method-target. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Index-signature consistency with method bivariance regression (TS unsoundness #25). Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Investigated FunctionId build error in `wasm/src/solver/evaluate.rs` after sync; no references found, build succeeded. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Implemented covariant `this`-type handling in parameter variance with regression coverage. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added class-like subtyping regression for `this`-typed parameters (base vs derived). Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added mixed method/function-property variance tests for `this` parameters. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added void-return exception coverage for method properties. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Reset flow at function boundaries to avoid narrowing in closures; added CFA invalidation test. Tests: `./wasm/test.sh` (fails: `wasm/src/transforms/async_es5.rs:749` unexpected closing delimiter after sync).

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-3`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
- Latest `./wasm/test.sh` fails at `emitter_edge_case_tests::test_parse_error_tolerance` (expects "x" declaration) after sync; compile error in `async_es5.rs` fixed locally.
