# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 4

## Current Assignment
- [ ] End-to-end validation: add a compile-and-check regression for a non-trivial generic library snippet in `wasm/src/parallel_tests.rs` (or `wasm/src/thin_checker_tests.rs`) and fix the first panic or mismatch in `wasm/src/checker/mod.rs` or `wasm/src/solver/mod.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Add a multi-file generic library test case (redux/lodash-style types) and assert no panics + expected diagnostics.
- [ ] If a panic arises, minimize to a focused solver/checker regression test.
- [ ] Validate the new test still passes with `./wasm/test.sh`.

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
