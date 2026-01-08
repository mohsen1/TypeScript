# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 2

## Current Assignment
- Verify ES5 output for computed `super[...]` calls does not leak `super[` in class fields (add emitter transform integration test if needed).

## Task Queue
- [ ] Confirm handling of `super()` ordering relative to field initializers when computed property names are present.

## Completed
- [x] Added regression for computed `super[...]` in field arrow initializers; `./wasm/test.sh` failed at `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports` (unrelated).

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
