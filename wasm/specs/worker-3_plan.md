# Worker 3 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 3

## Current Assignment
- Solver hardening: verify variance rules in `wasm/src/solver/subtype.rs`. Add targeted cases in `wasm/src/solver/subtype_tests.rs` for parameter contravariance and return-type covariance, then fix any inconsistencies.

## Task Queue
- [ ] If variance coverage is solid, add edge cases for optional/rest parameters and `this` parameters in assignability.

## Completed
- [x] (Move finished items here with brief notes and tests run.)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
