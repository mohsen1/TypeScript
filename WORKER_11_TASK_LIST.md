# Worker 11 Task List

## Squad: Solver (Semantics)

## Current Task
- [ ] Improve TS2322 (Type not assignable) detection in wasm/src/solver

## Queue
- [ ] Audit solve_subtype logic for missing strictness
- [ ] Find patterns where TS2322 should fire but doesn't
- [ ] Implement stricter type compatibility checks

## Completed
(none yet)

## Context
TS2322 is being missed. The solver needs to be meaner (stricter) to match tsc. Better to be too strict than unsound.
