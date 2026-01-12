# Worker 5 Task List

## Current Task
- [ ] **BINDER-3: Fix module augmentation resolution**
  - Trace how `interface Window` is merged across files
  - Implement proper symbol merging for global augmentations
  - Handle `declare global` blocks correctly
  - Add tests for multi-file interface merging

## Queue
- [ ] **BINDER-4: Eliminate false positive TS2304 errors**
  - Run conformance tests and identify remaining TS2304 issues
  - Categorize: missing globals vs. legitimate errors
  - Fix the root causes of false positives
  - Goal: reduce TS2304 extra errors to < 50

## Completed
(none yet)
