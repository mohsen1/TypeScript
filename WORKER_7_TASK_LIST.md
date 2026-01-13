# Worker 7 Task List

## Squad: Binder Squad (Builtins)

## Current Task
- [ ] Debug why basic globals like `console`, `Array`, `Promise` still fail to resolve in some cases

## Queue
- [ ] Add detailed logging to `resolve_identifier_symbol` in thin_checker.rs
- [ ] Trace symbol table lookup chain: local -> module -> global
- [ ] Verify lib.d.ts injection populates correct symbol IDs for builtins
- [ ] Test with minimal examples that should resolve to global types

## Completed
(Previous phase work archived)

## Context
- **Goal:** Ensure all standard library globals resolve correctly
- **Key files:** `wasm/src/thin_binder.rs`, `wasm/src/thin_checker.rs`, `wasm/src/lib_loader.rs`
- **Impact:** Unresolved globals cause Any fallback which suppresses downstream errors
