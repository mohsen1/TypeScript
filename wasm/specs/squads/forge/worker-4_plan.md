# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 4

## Current Assignment
- [ ] Fix the `FunctionId` build error in `wasm/src/solver/evaluate.rs` and re-run `./wasm/test.sh`.

## Task Queue
- [ ] Verify the fix passes `./wasm/test.sh` and note any unrelated failures.

## Completed
- [x] Added generic library regression + fixed declare function overload handling. Tests: `./wasm/test.sh test_generic_library_snippet_compiles_and_checks` (full suite fails on existing `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`).
- [x] Added multi-file generic regression across two files. Tests: `./wasm/test.sh test_multi_file_generic_library_snippet_compiles_and_checks`.

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
