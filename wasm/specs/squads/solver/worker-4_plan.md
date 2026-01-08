# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Solver for the Solver squad. Focus on type inference and constraint solving.

Status: Active
Priority: 4

## Current Assignment
- [EM: Assign initial task]

## Task Queue
- [ ] [EM: Add queued tasks]

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/solver-4`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
