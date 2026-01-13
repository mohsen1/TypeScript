# Worker 6 Task List

## Squad: Binder (CRITICAL)

## Current Task
- [ ] Fix module augmentation resolution in wasm/src/binder - merging interface Window across files

## Queue
- [ ] Trace how declaration merging should work for interfaces
- [ ] Implement or fix interface merging across module boundaries
- [ ] Add tests for module augmentation patterns

## Completed
(none yet)

## Context
Module augmentation (like extending Window interface) doesn't work correctly. This breaks many real-world TypeScript patterns.
