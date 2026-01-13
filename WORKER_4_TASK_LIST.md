# Worker 4 Task List

## Current Task
- [ ] Fix Global Scope and lib.d.ts injection
  - Locate `wasm/crates/swc_typescript/src/binder/global_scope.rs`
  - Ensure `lib.d.ts` symbols are merged into root SymbolTable
  - Verify console, Promise, Array, Object are resolvable
  - Add tests for global symbol resolution

## Queue
- [ ] Debug why global symbols show as "Cannot find name" (TS2304)
  - Add logging to symbol resolution path
  - Verify SymbolTable lookup chain reaches global scope
  - Check for scope chain breaks
  - Target: Reduce TS2304 extra errors to <50
- [ ] Fix global symbol type binding
  - Ensure global types (PromiseConstructor, ArrayConstructor) are correctly typed
  - Verify type parameters are bound
  - Test with generic global types

## Completed
(none yet)
