# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 2

## Current Assignment
- [ ] Fix `parallel::tests::test_check_redux_lodash_style_generics` - cross-file type alias resolution (5 remaining diagnostics)

## Task Queue
- [ ] Implement TypeResolver that can resolve symbols to provide type alias expansion in SubtypeChecker
- [ ] Wire up the TypeResolver to CompatChecker/SubtypeChecker in thin_checker.rs
- [ ] Test and verify all 5 remaining diagnostics are resolved

## Completed
- [x] Cross-file type resolution infrastructure: added `decl_file_idx` to Symbol, `alloc_from` for symbol cloning, `all_arenas` to CheckerContext for multi-file arena access.
- [x] Fixed type parameter scope propagation to TypeLowering via `import_type_params` method.
- [x] Added Application type expansion infrastructure to SubtypeChecker (`try_expand_application`, `extract_type_params_from_type`).
- [x] Fixed regression in `invalidate_paths_with_dependents_symbols_handles_import_equals` - handle require() string literals in import equals declarations.
- [x] Reduced diagnostics from 6 to 5; type parameters now resolve correctly (showing `<R>` instead of `<error>`).
- [x] Added strict-mode assignability comparison for rest unknown[] vs number[]; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Rest-parameter assignability parity checks for unknown[] vs number[]; restored AsyncES5Emitter setter to unblock builds; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added rest number[] assignability guard test; ran `./wasm/test.sh` (fails: missing `AsyncES5Emitter::set_use_this_capture` in `es5_helpers.rs` and `class_es5.rs`).
- [x] Refactored compat subtype configuration and tightened rest mismatch diagnostics assertions; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Added rest-parameter explain_failure coverage for source rest mismatches; ran `./wasm/test.sh` (fails: `parallel::tests::test_check_redux_lodash_style_generics`).
- [x] Implemented template-literal infer matching (including union-aware bindings) and updated conditional template inference tests. Ran `./wasm/test.sh` (fails: solver::compat::tests::test_explain_failure_reports_rest_mismatch).
- [x] Deferred `TooManyParameters` reporting for rest targets so `explain_failure` surfaces rest element mismatches; `./wasm/test.sh test_explain_failure_reports_rest_mismatch` passes. Full `./wasm/test.sh` now fails at `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports`.

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
