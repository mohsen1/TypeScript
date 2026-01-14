# Worker 5 Task List

## Squad: Binder - Global Scope & lib.d.ts Integration

## Current Task
- [ ] Debug why basic globals like `console`, `Promise`, `Array` fail to resolve in test cases
- [ ] Verify `lib_loader.rs` correctly merges `lib.d.ts` symbols into root `SymbolTable`

## Queue
- [ ] Audit `src/thin_binder.rs` to ensure `file_locals` are correctly populated from library context
- [ ] Fix module augmentation resolution (merging `interface Window` across files)
- [ ] Investigate TS2304 extra errors - reduce from 343 to <50
- [ ] Coordinate with Worker 6 on binding fixes

## Completed
- [x] Merge attempt - No commits to merge yet (Worker 5 at base commit ebd6cb201)

## Context
TS2304 (Cannot find name) is the #1 source of "Any" poisoning. When the Binder fails to find `Promise`, `console`, or `Array`, the Solver defaults to `Any`, suppressing all downstream errors.

### Key Files
- `src/lib_loader.rs` - lib.d.ts loading
- `src/thin_binder.rs` - binding logic
- `src/symbol.rs` - SymbolTable implementation

### Goal
Reduce TS2304 extra errors from 343 to <50 by fixing global scope binding.
