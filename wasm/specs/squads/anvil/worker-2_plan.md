# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 2

## Current Assignment
- Audit computed `super[...]` lowering in `wasm/src/transforms/class_es5.rs` for nested arrows and field initializers; add focused regression in `wasm/src/transforms/class_es5_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [ ] Verify ES5 output for computed `super[...]` calls does not leak `super[` in class fields (add emitter transform integration test if needed).
- [ ] Confirm handling of `super()` ordering relative to field initializers when computed property names are present.

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
- Push to: `origin/worker/anvil-2`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
