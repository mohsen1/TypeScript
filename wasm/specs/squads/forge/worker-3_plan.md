# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 3

## Current Assignment
- [x] Structural compatibility/variance: fix covariance/contravariance edge cases in `wasm/src/solver/subtype.rs`; add tests in `wasm/src/solver/subtype_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [x] Add regressions for function parameter variance across unions/intersections in `wasm/src/solver/subtype_tests.rs`.
- [ ] Confirm method vs function-property variance in `wasm/src/solver/subtype.rs` matches `tsc`, add coverage if missing.
- [ ] Pull next unsoundness case from `wasm/specs/TS_UNSOUNDNESS_CATALOG.md` and add a subtype regression test.

## Completed
- [x] Structural property/method variance: allow bivariant checks when either side is a method; added mixed method vs function-property test. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Function variance across union/intersection targets regression test. Tests: `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).

## Ready for Merge
Yes

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-3`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
