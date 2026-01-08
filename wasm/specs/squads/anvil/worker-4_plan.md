# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 4

## Current Assignment
- [ ] Await EM assignment.

## Task Queue
- [ ] [EM: Add queued tasks]

## Completed
- [x] Add async/class ES5 transform source-map mappings and offsets. Tests: `./wasm/test.sh source_map`

## Ready for Merge
Yes - branch `worker/anvil-4` is ready for merge.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
