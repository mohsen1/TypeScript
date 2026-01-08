# Worker 5 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 5

## Current Assignment
- Add regression coverage for ES5 downleveling of `super()` in derived classes with field initializers and nested async/arrow `this` capture; target `wasm/src/transforms/async_es5.rs` and `wasm/src/emitter_transform_integration_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Review `async_es5.rs` for `this` capture handling in nested arrows inside class fields.
- [ ] Add a focused unit test in `wasm/src/transforms/async_es5_tests.rs` for `super()` + async field initializer ordering.

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
- Push to: `origin/worker/anvil-5`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
