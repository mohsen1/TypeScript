# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 2

## Current Assignment
- [ ] Conditional type evaluation: implement distributive conditional handling and template-literal inference in `wasm/src/solver/evaluate.rs`; add regressions in `wasm/src/solver/evaluate_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Add non-distributive conditional template-literal infer cases (prefix/suffix/middle/two-infer) in `wasm/src/solver/evaluate_tests.rs`.
- [ ] Cover constrained template-literal inference (`infer T extends ...`) and confirm behavior matches `tsc`.
- [ ] Validate distributive vs wrapped conditional behavior with unions and `never`/`any` in `wasm/src/solver/evaluate_tests.rs`.

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
