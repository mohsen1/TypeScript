# Worker 6 Task List

## Current Task
- [ ] **BINDER-5: Fix Error Poisoning from unresolved symbols**
  - Find where the Solver defaults to `Any` on unknown symbols
  - Change behavior to preserve `Unknown` or emit error
  - Ensure unresolved names don't suppress downstream type checking
  - Add test: `const x: string = unknownSymbol.shouldError`

## Queue
- [ ] **BINDER-6: Coordinate with CFA and Solver squads**
  - Share findings about symbol resolution issues
  - Help debug tests that involve binding + flow analysis
  - Document integration points between phases
  - Run full conformance suite after fixes

## Completed
(none yet)
