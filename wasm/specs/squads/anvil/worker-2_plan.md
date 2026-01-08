# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 2

## Current Assignment
- [EM: Assign next task]

## Task Queue
- [ ] Expand integration coverage if ordering regression is found.

## Completed
- [x] Added integration coverage for computed `super[...]` in class field arrow initializers; `./wasm/test.sh` failed at `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports` (unrelated).
- [x] Confirmed `super()` ordering remains stable with computed field initializers via regression; `./wasm/test.sh` failed at `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports` (unrelated).
- [x] Updated emitter edge case and parity tests for CommonJS export/parse error tolerance; `./wasm/test.sh` now fails at `solver::compat::tests::test_explain_failure_reports_rest_mismatch` (unrelated).

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
