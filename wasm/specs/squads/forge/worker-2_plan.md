# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 2

## Current Assignment
- [ ] Conditional type evaluation: implement distributive conditional handling and non-distributive template-literal inference in `wasm/src/solver/evaluate.rs`; add regressions in `wasm/src/solver/evaluate_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Add non-distributive conditional template-literal infer cases (prefix/suffix/middle/two-infer) in `wasm/src/solver/evaluate_tests.rs`.
- [ ] Add constrained template-literal inference (`infer T extends ...`) and confirm behavior matches `tsc`.
- [ ] Validate distributive vs wrapped conditional behavior with unions and `never`/`any`, including tuple/object/function-property conditional infer edges; fix `wasm/src/solver/evaluate.rs` if mismatched.

## Completed
- [x] FunctionId build error fixed in `wasm/src/solver/evaluate.rs` via worker-5 (no action needed here).

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
