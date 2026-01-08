# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 4

## Current Assignment
- Drive end-to-end conformance by compiling a non-trivial generic library (e.g., lodash types) without panics; investigate failures in `wasm/src/cli/driver.rs` and `wasm/src/thin_emitter/mod.rs`; add minimal repro tests; run `./wasm/test.sh`.

## Task Queue
- [ ] Identify top panic site during library compile and file actionable follow-up tasks for Forge if type checking issues surface.
- [ ] Add a conformance regression test that exercises generic library emit.

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
