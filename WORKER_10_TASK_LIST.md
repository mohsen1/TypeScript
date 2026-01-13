# Worker 10 Task List

## Squad: Binder Squad (Error Propagation)

## Current Task
- [ ] Ensure Binder propagates Error type instead of Any when symbols cannot be resolved

## Queue
- [ ] Audit all remaining `TypeId::ANY` fallbacks in binder code paths
- [ ] Change remaining error-case fallbacks to `TypeId::ERROR` or `TypeId::UNKNOWN`
- [ ] Verify error propagation doesn't cause cascading false positives
- [ ] Test that downstream errors are revealed when Any poisoning is stopped

## Completed
(Previous phase work archived)

## Context
- **Goal:** Stop Any poisoning - propagate errors instead of silencing them
- **Key files:** `wasm/src/thin_binder.rs`, `wasm/src/thin_checker.rs`
- **Impact:** Error propagation should reveal missing TS2322/TS7006 errors
