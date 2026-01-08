# Worker 1 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 1

## Current Assignment
- [ ] Solver inference hardening: handle circular `extends` constraints and usage-based inference in `wasm/src/solver/infer.rs`; add regressions in `wasm/src/solver/infer_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Add context-sensitive typing inference cases (contextual signatures + `extends` constraints) in `wasm/src/solver/infer_tests.rs`.
- [ ] Verify constraint merge order for circular bounds (e.g., `T extends U`, `U extends T`, `U extends string`) and adjust `wasm/src/solver/infer.rs`.
- [ ] Add coverage for union targets with placeholder members in `wasm/src/solver/infer_tests.rs` if still failing vs `tsc`.

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-1`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
