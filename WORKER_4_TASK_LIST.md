# Worker 4 Task List - Binder Squad

## Current Task
- [ ] **BIND-10: Fix symbol lookup order**
  - Ensure correct scope chain: local -> module -> global
  - Handle `import { x }` vs `let x` shadowing correctly
  - Test scope chain traversal

## Queue
- [ ] **BIND-15: Fix window and DOM symbol resolution**
  - Ensure DOM types from lib.d.ts are accessible
  - Test: `window.alert("hi")` should not produce TS2304
  - Verify `document`, `navigator`, etc. resolve correctly

## Completed
- [x] **BIND-7: Test TS2304 fixes**
  - Created test file for all previously failing global symbols
  - Verified `console`, `Promise`, `Array`, `Object`, `String`, `Number` resolve correctly
  - Goal: Reduce TS2304 extra errors from 702 to <50
- [x] **BIND-4: Fix Global SymbolTable initialization**
  - Modified `Binder::new` to inject lib symbols into root scope
  - Loaded `lib.d.ts` types from `stdlib/lib.d.ts`
  - Ensured symbols are merged correctly at module level
  - Tested `console.log("hello")` no longer produces TS2304
  - Added `LibContext` struct to `thin_binder.rs`
  - Added `inject_lib_symbols` method to copy lib file symbols into `file_locals`
  - Modified `bind_source_file` in `lib.rs` to inject lib symbols after binding
  - Tracks symbol arenas for cross-file resolution
- [x] **BIND-1: Audit current global scope implementation**
  - Read `wasm/src/binder/scope.rs` to understand current `SymbolTable`
  - Identified how `lib.d.ts` symbols are (not) loaded
  - Found where global symbols like `console`, `Array`, `Promise` should be defined
  - Documented the gap in `docs/binder_gap_analysis.md`
