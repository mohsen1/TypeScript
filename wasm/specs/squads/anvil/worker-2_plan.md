# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 2

## Current Assignment
- Fix async ES5 computed `super[...]` element access lowering in `wasm/src/transforms/async_es5.rs` for async methods/returned arrows; update expectations in `wasm/src/emitter_transform_integration_tests.rs` to assert lowered output and preserved `this`/`arguments` capture; run `./wasm/test.sh`.

## Task Queue
- [ ] Convert async computed `super[...]` TODOs in `wasm/src/emitter_transform_integration_tests.rs` into passing assertions (prioritize returned arrow + nested arrow cases, including no-args returns where body currently drops).
- [ ] Audit `_super` helper emission in `wasm/src/transforms/async_es5.rs` to ensure computed element access uses `.call` with correct receiver (no `void 0["m"]` or leftover `super[...]`).
- [ ] Add a regression for computed `super[...]` inside an async class field arrow in `wasm/src/emitter_transform_integration_tests.rs`.

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize emitter fidelity (ES5 downleveling/source maps) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-2`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
