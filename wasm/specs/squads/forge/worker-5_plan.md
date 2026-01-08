# Worker 5 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 5

## Current Assignment
- [x] Inference edge cases: implement `this`-parameter inference and non-distributive optional tuple/property inference in `wasm/src/solver/infer.rs` and `wasm/src/solver/evaluate.rs`; add tests in `wasm/src/solver/infer_tests.rs` and `wasm/src/solver/evaluate_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Add tests for function `this`-parameter inference (contextual typing) in `wasm/src/solver/infer_tests.rs`.
- [ ] Add non-distributive optional tuple/property infer coverage in `wasm/src/solver/evaluate_tests.rs` and align behavior with `tsc`.
- [ ] Follow up with any missing `infer` placeholder vs `never` behavior from recent TODOs in solver tests.

## Completed
- [x] Implemented this-parameter bounds checking + conditional inference; added non-distributive optional tuple/property inference; updated tests (tests not run).

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-5`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
