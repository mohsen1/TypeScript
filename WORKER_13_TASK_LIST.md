# Worker 13 Task List

## Squad: Solver (Semantics)

## Current Task
- [ ] Implement function parameter bivariance handling

## Queue
- [ ] Implement void return type exceptions
- [ ] Handle other TypeScript-specific subtyping quirks

## Completed
- [x] Implement the "Lawyer" layer from specs/SOLVER.md for TypeScript quirks
  - Enhanced documentation with SOLVER.md Section 8 references
  - Added FreshnessTracker for excess property checking
  - Added TypeScriptQuirks struct documenting 9 quirks
  - Added 12 new tests for FreshnessTracker and TypeScriptQuirks

## Context
TypeScript has semantic quirks like function bivariance and void return exceptions. These need special handling in the solver to match tsc behavior.
