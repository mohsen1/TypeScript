# Worker 5 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 5

## Current Assignment
- [ ] Inference edge cases: implement `this`-parameter inference and non-distributive optional tuple/property inference in `wasm/src/solver/infer.rs` and `wasm/src/solver/evaluate.rs`; add tests in `wasm/src/solver/infer_tests.rs` and `wasm/src/solver/evaluate_tests.rs` (convert TODOs for optional property/tuple inference); run `./wasm/test.sh`.

## Task Queue
- [ ] Add tests for function `this`-parameter inference (contextual typing + call-site inference) in `wasm/src/solver/infer_tests.rs`.
- [ ] Convert TODOs in `wasm/src/solver/evaluate_tests.rs` for optional property inference (missing vs `undefined`) and optional tuple element inference (undefined inclusion).
- [ ] Follow up with remaining `infer` placeholder vs `never` TODOs in solver tests that block non-distributive inference.

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness (inference/conditional/subtype) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-5`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
