# Worker 1 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 1

## Current Assignment
- Implement ES5 derived `super()` + field initializer ordering and nested arrow/async `this` capture in `wasm/src/transforms/class_es5.rs`; add regression in `wasm/src/emitter_transform_integration_tests.rs`; run `./wasm/test.sh`.

## Task Queue
- [x] Audit computed `super[...]` lowering paths in `wasm/src/transforms/class_es5.rs` for nested arrows.
- [x] Add a focused unit test in `wasm/src/transforms/class_es5_tests.rs` for pre-`super()` statement ordering (already covered by existing test).
- [x] Confirm ES5 output removes `super[` for computed super calls in class fields.

## Completed
- [x] Implemented async/nested arrow `this` capture handling in ES5 class emission, added derived async field regression; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added class ES5 computed super field arrow regression; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
