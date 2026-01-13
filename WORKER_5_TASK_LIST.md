# Worker 5 Task List

## Squad: CFA Squad (TS2454 Focus)

## Current Task
- [ ] Review narrowing logic in `wasm/src/solver/narrowing.rs` for over-aggressive unassigned detection

## Queue
- [ ] Identify patterns where our CFA thinks variable is unassigned but tsc accepts it
- [ ] Fix flow analysis for loops, try/catch, and conditional assignments
- [ ] Add test cases for patterns that cause false positive TS2454

## Completed
- [x] Analyze TS2454 extra errors (Variable used before being assigned) - 225 false positives

## Context
- **Goal:** Reduce TS2454 extra errors (225 false positives)
- **Key files:** `wasm/src/checker/control_flow.rs`, `wasm/src/solver/narrowing.rs`, `wasm/src/checker/reachability_analyzer.rs`
- **Status:** CFA framework is in place - need to fix over-triggering
