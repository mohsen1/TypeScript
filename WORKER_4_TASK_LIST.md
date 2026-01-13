# Worker 4 Task List

## Squad: CFA Squad (TS2564 Focus)

## Current Task
- [ ] Review `wasm/src/checker/control_flow.rs` and `flow_analyzer.rs` for definite assignment tracking gaps

## Queue
- [ ] Identify class property initialization patterns not being detected
- [ ] Fix constructor flow analysis to track property assignments through method calls
- [ ] Add test cases for complex initialization patterns (conditional, super calls, helper methods)

## Completed
- [x] Analyze TS2564 missing errors (Property has no initializer) - 413 occurrences still missing

## Context
- **Goal:** TS2564 is #1 missing error (413 occurrences) - fix edge cases
- **Key files:** `wasm/src/checker/control_flow.rs`, `wasm/src/checker/flow_analyzer.rs`, `wasm/src/thin_checker.rs`
- **Status:** Flow Graph Side-Table is implemented - edge cases need work
