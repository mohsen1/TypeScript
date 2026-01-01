# Task Queue

## Phase 3 - Parser: COMPLETE ✓

All parser tasks finished:
- Scanner: 100%
- Parser: ~98%
- Integration: Complete
- 81 Rust tests + 19 TS tests passing

## Phase 4 - Binder: IN PROGRESS

### Completed ✓

- [x] **Symbol struct in Rust**
  - File: `wasm/src/binder.rs`
  - Port Symbol from TypeScript
  - Add SymbolFlags

- [x] **Symbol table**
  - SymbolArena for symbol allocation
  - SymbolTable for name lookup

- [x] **Scope management**
  - Block scope (push/pop)
  - Function scope (parameters)
  - Module scope (namespaces)

- [x] **Declaration merging**
  - Interface merging
  - Namespace merging
  - Class + namespace merging

- [x] **WASM exposure**
  - createBinder factory
  - JSON serialization for symbols

### Priority: High (Next)

- [ ] **Flow analysis setup**
  - Control flow graph
  - Narrowing framework

- [ ] **TypeScript integration**
  - bindWithRustBinder in binder.ts
  - Test with real TS files

## Blocked

(none)

## Notes

- Phase 4 started 2026-01-01
- 94 Rust tests passing
- Binder creates symbols for: variables, functions, classes, interfaces, type aliases, enums, namespaces
