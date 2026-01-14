# Worker 4 Task List

## Squad: Binder (CRITICAL)

## Current Task (Assigned by EM-1)
- [ ] **CRITICAL PATH:** Complete ambient module declarations fix (declare module "foo")
  - Investigate how ambient modules are currently bound
  - Ensure `declare module "foo"` blocks create proper module symbols
  - Fix any issues with string literal module names
  - Add tests for ambient module scenarios
  - Run conformance to measure TS2304 reduction

## Queue (High Priority - TS2304 is #1 blocker)
- [ ] Implement module augmentation resolution (merging `interface Window` across files)
- [ ] Debug console/Array resolution failures in complex scenarios
- [ ] Target: Reduce TS2304 extra errors from 343 to <50

## Completed
- [x] Investigate lib.d.ts loading and symbol merging into root SymbolTable
- [x] Trace how lib.d.ts symbols (Array, Promise, console) are supposed to be injected
- [x] Fix symbol table merging to correctly expose global types
- [x] Add tests verifying lib.d.ts globals are resolvable

## Context
TS2304 (Cannot find name) is the #1 source of error poisoning. When the Binder fails to resolve Promise, the Solver defaults to Any, suppressing all downstream errors.

---

## Implementation Summary

### Problem
After merging lib.d.ts symbols into `file_locals`, the checker's `get_symbol()` method could not resolve lib symbols because they were stored in the lib binder's arena, not the local binder's arena.

### Solution
Modified `ThinBinderState` to:
1. Store lib binders in a new `lib_binders: Vec<Arc<ThinBinderState>>` field
2. During `merge_lib_symbols()`, store references to lib binders
3. Update `get_symbol()` to check lib binders automatically if symbol not found locally

### Changes Made
- `wasm/src/thin_binder.rs`:
  - Added `lib_binders` field to `ThinBinderState` struct
  - Updated `new()`, `reset()`, `from_bound_state()`, `from_bound_state_with_scopes()` to initialize/reset `lib_binders`
  - Updated `merge_lib_symbols()` to store lib binders
  - Updated `get_symbol()` to check lib binders when symbol not found locally

- `wasm/src/lib_loader.rs`:
  - Added `test_get_symbol_resolves_lib_symbols()` test to verify the fix

### Verification
All lib_loader tests pass:
- `test_merge_lib_symbols`
- `test_load_default_lib_dts`
- `test_bind_with_lib_symbols`
- `test_get_symbol_resolves_lib_symbols` (new)

### Impact
This fix ensures that when the checker calls `get_symbol()` with a lib symbol ID (e.g., for `Promise`, `Array`, `console`), it will correctly resolve to the Symbol object from the lib binder, eliminating the TS2304 errors caused by the previous mismatch.
