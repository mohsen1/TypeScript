# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Anvil for the Anvil squad (output: emitter, transforms, cli, lsp).

Status: Active
Priority: 4

## Current Assignment
- Add an end-to-end compile test for a small multi-file generic library (interfaces, generics, conditional types) in `wasm/src/cli/driver_tests.rs`; assert no diagnostics/panics and outputs written; run `./wasm/test.sh`.

## Task Queue
- [ ] Create a multi-file fixture (e.g., `src/index.ts`, `src/types.ts`) in the temp dir with imports/exports and generic constraints.
- [ ] Assert JS output exists for each input file and (if enabled) map files are emitted.
- [ ] If compile fails, isolate the panic/diagnostic and add the minimal regression coverage.

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize emitter fidelity (ES5 downleveling/source maps) before new features.
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: <description>` or `[wasm] cli: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
