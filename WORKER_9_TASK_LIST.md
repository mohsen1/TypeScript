# Worker 9 Task List

## Squad: Binder Squad (Symbol Lookup)

## Current Task
- [ ] Audit symbol table lookup chain in `wasm/src/thin_binder.rs` for breaks in scope traversal

## Queue
- [ ] Fix cases where block-scoped declarations (let/const) shadow incorrectly
- [ ] Verify import/export symbol visibility in importing modules
- [ ] Test with circular imports to ensure no infinite loops or missing symbols
- [ ] Add validation to detect orphaned symbols or broken links

## Completed
(Previous phase work archived)

## Context
- **Goal:** Symbol lookup must correctly traverse: local -> module -> global
- **Key files:** `wasm/src/thin_binder.rs`, `wasm/src/thin_checker.rs`
- **Impact:** Broken lookup chain causes TS2304 errors
