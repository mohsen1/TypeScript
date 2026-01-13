# Worker 15 Task List

## Squad: CFA (Control Flow Analysis)

## Current Task
- [ ] Fix TS2454 (Variable used before assignment) extra errors (225) in wasm/src/cfa

## Queue
- [ ] Analyze patterns where TS2454 fires incorrectly (false positives)
- [ ] Fix flow graph to correctly track assignments through control flow
- [ ] Handle try/catch/finally assignment tracking

## Completed
(none yet)

## Context
TS2454 has 225 extra errors (false positives). The CFA is being too strict in some cases. Need to fix flow analysis to match tsc behavior.
