# Worker 5 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 5

## Current Assignment
- Complete: Expanded async ES5 source-map coverage for loop/try constructs in `wasm/src/source_map_tests.rs` with direct mapping checks; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).

## Task Queue
- [ ] (empty)

## Completed
- [x] Added direct mapping tests for for-loop header awaits, do/while await condition, and try/finally await in `finally` in `wasm/src/source_map_tests.rs`; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).

## Ready for Merge
Yes

## Notes
- Project Direction: integration and conformance-first; prioritize emitter fidelity (ES5 downleveling/source maps) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-5`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
