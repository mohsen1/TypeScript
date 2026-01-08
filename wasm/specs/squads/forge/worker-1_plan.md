# Worker 1 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 1

## Current Assignment
- [x] Solver inference hardening: handle circular `extends` constraints and usage-based inference in `wasm/src/solver/infer.rs`; add regressions in `wasm/src/solver/infer_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [x] Add context-sensitive typing inference cases (contextual signatures + `extends` constraints) in `wasm/src/solver/infer_tests.rs`.
- [x] Verify constraint merge order for circular bounds (e.g., `T extends U`, `U extends T`, `U extends string`) and adjust `wasm/src/solver/infer.rs`.
- [x] Add coverage for union targets with placeholder members in `wasm/src/solver/infer_tests.rs` if still failing vs `tsc`.

## Completed
- [x] Solver inference hardening: add cyclic upper bound expansion + usage-based inference tests. Ran `./wasm/test.sh` (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).
- [x] Added contextual signature bounds tests for function parameter/return variance. Ran `./wasm/test.sh` (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).
- [x] Added circular upper-bound order regression test. Ran `./wasm/test.sh` (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).
- [x] Added union target placeholder inference test. Ran `./wasm/test.sh` (fails: emitter_edge_case_tests::test_export_assignment_suppresses_other_exports).

## Ready for Merge
Yes

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-1`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
