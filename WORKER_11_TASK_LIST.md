# Worker 11 Task List

## Squad: Solver Squad (TS2322 Focus)

## Current Task
- [ ] Analyze remaining TS2322 (Type not assignable) missing errors - improve detection

## Queue
- [ ] Review `wasm/src/solver/subtype.rs` for cases where mismatches are incorrectly accepted
- [ ] Audit structural type checking in `wasm/src/solver/compat.rs`
- [ ] Add stricter checks for object literal excess property detection
- [ ] Test complex union/intersection assignability edge cases

## Completed
(Previous phase work archived - ANY fallback work complete)

## Context
- **Goal:** Convert "Missing TS2322" into "Exact Match" or "Extra TS2322" (better to be too strict than unsound)
- **Key files:** `wasm/src/solver/subtype.rs`, `wasm/src/solver/compat.rs`, `wasm/src/thin_checker.rs`
- **Note:** Previous phase completed ANY->ERROR fallback changes
