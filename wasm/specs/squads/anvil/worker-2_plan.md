# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 2

## Current Assignment
- [EM: Assign next task]

## Task Queue
- [ ] Convert async computed `super[...]` TODOs in `wasm/src/emitter_transform_integration_tests.rs` into passing assertions (prioritize returned arrow + nested arrow cases, including no-args returns where body currently drops).
- [ ] Audit `_super` helper emission in `wasm/src/transforms/async_es5.rs` to ensure computed element access uses `.call` with correct receiver (no `void 0["m"]` or leftover `super[...]`).
- [ ] Add regression coverage for computed `super[...]` inside async arrow returns with `this`/`arguments` capture.

## Completed
- [x] Lowered async ES5 computed `super[...]` element access in returned/nested arrows + updated integration expectations; `./wasm/test.sh` failed at `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports` (unrelated).
- [x] Lowered computed `super[...]` calls in async ES5 emitter + updated integration expectations; `./wasm/test.sh` failed at `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports` (unrelated).
- [x] Added integration coverage for computed `super[...]` in class field arrow initializers; `./wasm/test.sh` failed at `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports` (unrelated).
- [x] Confirmed `super()` ordering remains stable with computed field initializers via regression; `./wasm/test.sh` failed at `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports` (unrelated).

## Ready for Merge
Yes - branch ready for merge into rust.

## Notes
- Project Direction: integration and conformance-first; prioritize emitter fidelity (ES5 downleveling/source maps) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Conformance focus: tie regressions to official TypeScript conformance cases when possible.
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-2`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
