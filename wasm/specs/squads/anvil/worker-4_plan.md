# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 4

## Current Assignment
- [ ] Awaiting EM assignment.

## Task Queue
- [ ] [EM: Add queued tasks]

## Completed
- [x] Add async/class ES5 transform source-map mappings and offsets. Tests: `./wasm/test.sh source_map`
- [x] Add async nested function source-map offset coverage. Tests: `./wasm/test.sh source_map`
- [x] Add async await detection test for nested functions. Tests: `./wasm/test.sh body_contains_await`
- [x] Add ES5 derived default constructor ordering test. Tests: `./wasm/test.sh default_derived_constructor`

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
- Proposed next tasks for EM assignment:
  - Validate ES5 class downleveling edge cases for `super()` + field initializers in `wasm/src/transforms/class_es5.rs`.
  - Add coverage for async downlevel source-map offsets in `wasm/src/transforms/async_es5.rs`.
