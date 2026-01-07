---
name: worker-assignment-orchestrator
description: Assigns highest-impact tasks to worker plans using Project Direction and current plan state.
---

# Worker Assignment Orchestrator

Use this when the manager needs to assign or rebalance work across the five generic workers.

## Inputs to read
- `wasm/README.md` (Project Direction section only)
- `wasm/specs/worker-*_plan.md`
- Domain backlogs for candidates: `wasm/specs/emitter_plan.md`, `wasm/specs/cli_plan.md`, `wasm/specs/lsp_plan.md`, `wasm/specs/checker_plan.md`, `wasm/specs/solver_plan.md`

## Workflow
1. Read Project Direction and extract the top 3 priorities.
2. Read all worker plans and note:
   - `Status`, `Priority`, current assignment, and blockers.
3. Pull 1-3 highest-impact tasks from domain backlogs that align with Project Direction.
4. Assign tasks so coverage spans multiple fronts; avoid duplicating work across workers.
5. Update each worker plan:
   - Set `Current Assignment` to a concrete, testable task.
   - Add 1-3 items in `Task Queue`.
   - Adjust `Priority` if a worker needs to be bumped.
   - Keep `Status` as `Active` unless stopping the worker.

## Status hygiene
- `Status: Active` means the worker is expected to work now.
- `Status: Complete` only if you are intentionally stopping a worker (rare).
- Ensure each active worker has a clear next step and a test requirement.

## Output expectations
- Assignments are specific (file + behavior + test).
- Each worker has a clear stopping condition.
- No worker is idle.
