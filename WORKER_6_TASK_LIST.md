# Worker 6 Task List

## Squad: Binder (CRITICAL)

## Current Task
- [ ] Fix ambient module declarations (declare module "foo")

## Queue
- [ ] Debug console/Array resolution failures in complex scenarios
- [ ] Verify module augmentation fix handles all patterns (global, module, namespace)
- [ ] Fix module resolution for @types packages

## Completed
- [x] Fix module augmentation resolution - preserve global_augmentations through parallel binding
- [x] Trace how TypeScript handles declaration merging across files
- [x] Implement symbol merging for interface augmentations

## Context
Module augmentation (like extending Window interface) now works correctly for global_augmentations. Need to continue fixing ambient module declarations.

---

## Implementation Summary

### Module Augmentation Fix
**Problem:** `interface Window` extensions across files were not being merged correctly.

**Solution:** Modified parallel binding to preserve `global_augmentations` through the binding process.

**Changes:**
- `wasm/src/parallel.rs`: Preserve global_augmentations during parallel binding
- `wasm/src/thin_binder.rs`: Handle global augmentation symbols correctly
- `wasm/src/cli/driver.rs`: Pass augmentation context to binder

**Impact:** Fixes a major source of TS2304 errors where augmented interfaces were not resolvable.
