# Worker 14 Task List

## Squad: Solver Squad (Generic Inference)

## Current Task
- [ ] Review generic type inference in `wasm/src/solver/infer.rs` for edge cases

## Queue
- [ ] Fix complex generic constraint inference
- [ ] Test conditional type inference (T extends U ? X : Y)
- [ ] Improve mapped type inference accuracy
- [ ] Add conformance tests for generic function calls with partial inference

## Completed
(Previous phase work archived)

## Context
- **Goal:** Generic inference must match tsc behavior for type parameter constraints
- **Key files:** `wasm/src/solver/infer.rs`, `wasm/src/solver/instantiate.rs`
- **Note:** Previous phase completed mapped types and index access types
