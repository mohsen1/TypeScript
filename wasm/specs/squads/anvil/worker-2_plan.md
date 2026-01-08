# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 2

## Current Assignment
- [EM: Assign next task]

## Task Queue
- [ ] [EM: Add queued tasks]

## Completed
- [x] Moved async ES5 transform tests into `async_es5_tests.rs`, added nested async await coverage, and fixed extra call-expression brace; `./wasm/test.sh` failed at `parallel::tests::test_check_redux_lodash_style_generics` (unrelated).
- [x] Cleared merge artifact in async ES5 emitter while validating computed `super[...]` lowering + integration coverage; `./wasm/test.sh` failed at `emitter_parity_tests::test_parity_commonjs_export` (trailing newline mismatch).
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
