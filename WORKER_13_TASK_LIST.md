# Worker 13 Task List

## Squad: Solver Squad (Subtyping Edge Cases)

## Current Task
- [ ] Audit `wasm/src/solver/subtype.rs` for edge cases that deviate from tsc behavior

## Queue
- [ ] Fix nullable type subtyping (`T | null` vs `T | undefined` vs `T | null | undefined`)
- [ ] Review literal type widening rules
- [ ] Test readonly array vs mutable array assignability
- [ ] Add conformance tests for complex subtyping scenarios

## Completed
(none yet)

## Context
- **Goal:** Match tsc subtyping semantics exactly
- **Key files:** `wasm/src/solver/subtype.rs`, `wasm/src/solver/compat.rs`
- **Impact:** Incorrect subtyping leads to both missing and extra errors
