# Worker 1 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 1

## Current Assignment
- Awaiting next assignment.

## Task Queue
- [ ] None.

## Completed
- [x] Prefer upper bounds when lower bounds are only `any`/`unknown`; added `test_resolve_any_lower_prefers_upper_bound`. Tests: `./wasm/test.sh test_resolve_any_lower_prefers_upper_bound`.

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
