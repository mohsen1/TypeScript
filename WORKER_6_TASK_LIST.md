# Worker 6 Task List

## Squad: Binder Squad (Global Scope)

## Current Task
- [ ] Review `wasm/src/thin_binder.rs` global scope initialization

## Queue
- [ ] Verify `lib.d.ts` symbols are correctly merged into root SymbolTable
- [ ] Add logging to symbol resolution path to track where lookups fail
- [ ] Fix scope chain to properly reach global scope for built-in types

## Completed
- [x] Debug why TS2304 is BOTH missing (116) AND extra (343) - analyze binder symbol resolution

## Context
- **Goal:** Reduce TS2304 extra errors from 343 to <50
- **Key files:** `wasm/src/thin_binder.rs`, `wasm/src/lib_loader.rs`
- **Critical:** TS2304 causes error poisoning - Solver defaults to Any when Binder fails
