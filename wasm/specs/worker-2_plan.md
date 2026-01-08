# Worker 2 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 2

## Current Assignment
- Solver hardening: stress test conditional type evaluation in `wasm/src/solver/evaluate.rs`. Add or extend tests in `wasm/src/solver/evaluate_tests.rs` for distributive conditionals over unions and fix any mismatches.

## Task Queue
- [ ] If conditionals already pass, add coverage for nested/distributive conditionals with `extends` and `infer` positions.

## Completed
- [x] (Move finished items here with brief notes and tests run.)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
