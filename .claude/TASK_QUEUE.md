# Task Queue

## Phase 3 - Parser: COMPLETE ✓

All parser tasks finished:
- Scanner: 100%
- Parser: ~98%
- Integration: Complete
- 81 Rust tests + 19 TS tests passing

## Phase 4 - Binder: COMPLETE ✓

All Phase 4 tasks finished:

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

- [x] **Flow analysis setup**
  - FlowFlags, FlowNodeId, FlowNode, FlowNodeArena
  - Control flow graph structures ready

- [x] **TypeScript integration**
  - bindSourceFile and getBindingResult in ParserState
  - wasm.ts interface updated
  - 10 binder verification tests passing

## Phase 5 - Type Checker (Next)

### Priority: High

- [ ] **Type struct in Rust**
  - TypeFlags
  - TypeArena

- [ ] **Basic type checking**
  - Primitive types
  - Object types
  - Function types

- [ ] **Type inference**
  - Variable inference
  - Return type inference

### Priority: Medium

- [ ] **Generic types**
  - Type parameters
  - Type arguments

- [ ] **Union/Intersection types**
  - Type narrowing
  - Type guards

## Blocked

(none)

## Notes

- Phase 4 completed 2026-01-01
- 98 Rust tests + 10 binder tests passing
- Binder creates symbols for: variables, functions, classes, interfaces, type aliases, enums, namespaces, imports
