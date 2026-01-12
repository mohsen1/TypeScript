# Worker 4 Task List - Binder Squad

## Current Task
- [ ] **BIND-1: Audit current global scope implementation**
  - Read `src/binder/scope.rs` to understand current `SymbolTable`
  - Identify how `lib.d.ts` symbols are (not) loaded
  - Find where global symbols like `console`, `Array`, `Promise` should be defined
  - Document the gap in `docs/binder_gap_analysis.md`

## Queue
- [ ] **BIND-4: Fix Global SymbolTable initialization**
  - Modify `Binder::new` to inject lib symbols into root scope
  - Load `lib.d.ts` types from `stdlib/lib.d.ts`
  - Ensure symbols are merged correctly at module level
  - Test: `console.log("hello")` should not produce TS2304
- [ ] **BIND-7: Test TS2304 fixes**
  - Create test file for all previously failing global symbols
  - Verify `console`, `Promise`, `Array`, `Object`, `String`, `Number` resolve correctly
  - Goal: Reduce TS2304 extra errors from 702 to <50

## Completed
(none yet)
