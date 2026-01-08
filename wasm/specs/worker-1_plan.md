# Worker 1 Plan

## Mission
Execute the highest-impact tasks assigned by the manager across all areas (solver, checker, emitter, CLI, LSP).

Status: Active
Priority: 1

## Current Assignment
- Solver hardening: focus on inference from contextual types in `wasm/src/solver/infer.rs`. Add a focused test in `wasm/src/solver/infer_tests.rs` that currently fails (contextual function inference or circular constraint) and implement the minimal fix.

## Task Queue
- [ ] If contextual inference is already correct, target circular `extends` constraints and add a regression test.

## Completed
- [x] (Move finished items here with brief notes and tests run.)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md` when applicable.
- Use Docker for Rust tests (`./wasm/test.sh`), never `cargo test` directly.
- Update this plan after each task and keep it accurate.
