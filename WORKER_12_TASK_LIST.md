# Worker 12 Task List

## Squad: Solver (Semantics)

## Current Task
- [ ] Implement TS7006 (Implicit Any) detection in wasm/src/solver

## Queue
- [ ] Find where parameters/variables should have implicit any warnings
- [ ] Implement noImplicitAny checking in the solver
- [ ] Add tests for implicit any detection

## Completed
(none yet)

## Context
TS7006 (Implicit Any) is being missed. When --noImplicitAny is enabled, parameters without type annotations should error.
