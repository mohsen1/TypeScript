# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 3

## Current Assignment
- [ ] Fix FunctionId build error in `wasm/src/solver/evaluate.rs`; re-run `./wasm/test.sh`.

## Task Queue
- [ ] Add class-subtyping regressions where `this` appears in method parameters (base vs derived) and confirm assignment matches `tsc`.
- [ ] Confirm method vs function-property variance (including `this` parameters) in `wasm/src/solver/subtype.rs` matches `tsc`, add coverage if missing.
- [ ] Add a void-return exception regression (`() => void` accepts `() => string`) if not already covered, or pick the next unsoundness case from `wasm/specs/TS_UNSOUNDNESS_CATALOG.md`.

## Completed
- [x] Structural property/method variance: allow bivariant checks when either side is a method; added mixed method vs function-property test. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Function variance across union/intersection targets regression test. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Method vs function-property variance coverage for function-source to method-target. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Index-signature consistency with method bivariance regression (TS unsoundness #25). Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).

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
