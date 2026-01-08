# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 3

## Current Assignment
- [ ] Structural compatibility/variance: implement covariant `this`-type handling for class method parameters (TS unsoundness #19) in `wasm/src/solver/subtype.rs`; add regressions in `wasm/src/solver/subtype_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Add class-subtyping regressions where `this` appears in method parameters (base vs derived) and confirm assignment matches `tsc`.
- [ ] Confirm method vs function-property variance (including `this` parameters) in `wasm/src/solver/subtype.rs` matches `tsc`, add coverage if missing.
- [ ] Add a void-return exception regression (`() => void` accepts `() => string`) if not already covered, or pick the next unsoundness case from `wasm/specs/TS_UNSOUNDNESS_CATALOG.md`.

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
- Push to: `origin/worker/forge-3`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
