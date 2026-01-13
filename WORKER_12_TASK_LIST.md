# Worker 12 Task List

## Squad: Solver Squad (TS7006 Focus)

## Current Task
- [ ] Analyze missing TS7006 (Implicit Any parameter) errors - ensure detection in strict mode

## Queue
- [ ] Review function parameter type checking in `wasm/src/thin_checker.rs`
- [ ] Verify strict mode flag properly enables implicit any detection
- [ ] Fix cases where function parameters without type annotations don't emit TS7006
- [ ] Add test cases for implicit any in callbacks, arrow functions, and method parameters

## Completed
(none yet)

## Context
- **Goal:** TS7006 is key for strict mode compliance
- **Key files:** `wasm/src/thin_checker.rs`, `wasm/src/solver/infer.rs`
- **Impact:** Missing TS7006 means we're too permissive with untyped parameters
