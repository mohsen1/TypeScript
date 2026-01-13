# Worker 15 Task List

## Squad: Solver Squad (Lawyer Layer)

## Current Task
- [ ] Review `wasm/src/solver/lawyer.rs` for TypeScript quirks handling (function bivariance, void return exceptions)

## Queue
- [ ] Audit function bivariance rules - methods should be bivariant, functions contravariant in strict mode
- [ ] Fix void return type exceptions (functions returning anything can be assigned to void-returning type)
- [ ] Test "this" parameter type checking
- [ ] Verify excess property checking is properly strict for object literals

## Completed
(Previous phase work archived)

## Context
- **Goal:** The "Lawyer" layer handles TypeScript's intentional deviation from sound type theory
- **Key files:** `wasm/src/solver/lawyer.rs`, `wasm/src/solver/subtype.rs`
- **Reference:** specs/SOLVER.md describes the Lawyer vs Judge pattern
