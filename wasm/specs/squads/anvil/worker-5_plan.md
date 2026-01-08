# Worker 5 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 5

## Current Assignment
- Expand async ES5 source-map coverage for loop/try constructs in `wasm/src/source_map_tests.rs` (e.g., `for` init/condition/increment awaits, `do/while` await condition); ensure mappings are non-trivial; run `./wasm/test.sh`.

## Task Queue
- [ ] Add async `for` loop mapping tests with await in init/condition/update positions.
- [ ] Add async `do/while` or `switch` mapping test with await in the condition/discriminant.
- [ ] Add an async `try/finally` mapping test to cover await in `finally` and verify map entries.

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
- Push to: `origin/worker/anvil-5`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
