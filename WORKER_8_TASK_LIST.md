# Worker 8 Task List

## Squad: Binder Squad (Module Augmentation)

## Current Task
- [ ] Fix module augmentation resolution - merging `interface Window` across files

## Queue
- [ ] Ensure module augmentations update existing declarations correctly
- [ ] Test with multiple files augmenting same global interface
- [ ] Fix namespace merging behavior for nested namespaces
- [ ] Verify exported symbols are visible to augmentations

## Completed
(Previous phase work archived)

## Context
- **Goal:** Module augmentation is critical for lib.d.ts to work correctly
- **Key files:** `wasm/src/thin_binder.rs`, `wasm/src/binder.rs`
- **Impact:** Incorrect augmentation leads to TS2304 and error poisoning
