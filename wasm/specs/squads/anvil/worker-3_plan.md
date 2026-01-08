# Worker 3 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 3

## Current Assignment
- Validate source map generation for async/await downleveling in `wasm/src/thin_emitter/source_map.rs` and `wasm/src/thin_emitter/source_writer.rs`; add regression in `wasm/src/emitter_transform_integration_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Add a unit test covering source map entries for `await` inside nested arrow functions.
- [ ] Check source map attachment metadata for ES5 async transforms.

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
- Push to: `origin/worker/anvil-3`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
