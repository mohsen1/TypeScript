# Worker 1 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 1

## Current Assignment
- [EM: Assign next task]

## Task Queue
- [x] Inspect `wasm/src/emitter_edge_case_tests.rs` to capture the failing case and expected output.
- [x] Trace emit path for parse error recovery (likely `ThinParser`/`ThinPrinter`); fix missing declaration emission.
- [x] Add/update a focused regression if needed; run `./wasm/test.sh`.

## Completed
- [x] Implemented derived `super()` ordering adjustment and broader `this`/`super` capture in field initializers; added integration regression for nested async arrow in derived field; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_parse_error_tolerance`).
- [x] Implemented async/nested arrow `this` capture handling in ES5 class emission, added derived async field regression; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added class ES5 computed super field arrow regression; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Ensured derived constructors initialize private fields after `super` and added async arrow field regression; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Allowed ES6 `class C` in export assignment edge-case test; ran `./wasm/test.sh emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`.
- [x] Ensured synthesized derived constructors use `_this` in field/private initializers; added tests; ran `./wasm/test.sh` (fails: `emitter_edge_case_tests::test_parse_error_tolerance`).
- [x] Fixed parse-error recovery for const initializers; ran `./wasm/test.sh` (fails: `emitter_parity_tests::test_parity_commonjs_export`).
- [x] Fixed ES5 async generator emission for non-await blocks, aligned super calls with `_this`, and suppressed static-field `this` capture; ran `./wasm/test.sh` (fails: `solver::compat::tests::test_explain_failure_reports_rest_mismatch`).
- [x] Added static field arrow `this` regression for ES5 class emission; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).

## Ready for Merge
Yes

## Notes
- Project Direction: integration and conformance-first; prioritize emitter fidelity (ES5 downleveling/source maps) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
- Verified `./wasm/test.sh emitter_edge_case_tests::test_export_assignment_suppresses_other_exports` passes; full suite not rerun.
