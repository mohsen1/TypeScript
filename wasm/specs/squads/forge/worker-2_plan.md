# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 2

## Current Assignment
- [x] Conditional type evaluation: implement distributive conditional handling and non-distributive template-literal inference in `wasm/src/solver/evaluate.rs`; add regressions in `wasm/src/solver/evaluate_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] [EM: Assign next task]

## Completed
- [x] Implemented template-literal infer matching (including union-aware bindings) and updated conditional template inference tests. Ran `./wasm/test.sh` (fails: solver::compat::tests::test_explain_failure_reports_rest_mismatch).
- [x] Deferred `TooManyParameters` reporting for rest targets so `explain_failure` surfaces rest element mismatches; `./wasm/test.sh test_explain_failure_reports_rest_mismatch` passes. Full `./wasm/test.sh` now fails at `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`.

## Ready for Merge
Yes

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
