# Worker 1 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 1

## Current Assignment
- [ ] Generic inference hardening: implement circular `extends` constraint resolution and contextual signature + usage inference in `wasm/src/solver/infer.rs`; add regressions in `wasm/src/solver/infer_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Add circular bounds tests (e.g., `T extends U`, `U extends T`, `U extends string`) and verify constraint merge order in `wasm/src/solver/infer_tests.rs`.
- [ ] Add contextual signature + usage-based inference cases in `wasm/src/solver/infer_tests.rs` and fix inference wiring in `wasm/src/solver/infer.rs`.
- [ ] Cover union targets with placeholder members and numeric index-name bound inference; confirm inference does not collapse to `never`.

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-1`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
