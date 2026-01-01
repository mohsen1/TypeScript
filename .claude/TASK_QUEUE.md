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

## Phase 5 - Type Checker (IN PROGRESS)

### Completed

- [x] **Type struct in Rust** (Phase 5.1)
  - TypeFlags and ObjectFlags modules
  - TypeId for type references
  - TypeArena with singleton caching
  - 14 intrinsic types pre-allocated

- [x] **Type variants** (Phase 5.1)
  - IntrinsicType, LiteralType, ObjectType
  - UnionType, IntersectionType
  - TypeParameter, ConditionalType, MappedType
  - IndexedAccessType, IndexType, TemplateLiteralType
  - TypeReference with type arguments

- [x] **CheckerState** (Phase 5.2)
  - Type caching (node→type, symbol→type)
  - Diagnostic collection
  - get_type_of_node() for inference
  - get_type_of_symbol() for symbol lookup

- [x] **Type assignability** (Phase 5.2)
  - is_type_assignable_to() with rules for:
    - any/unknown/never handling
    - Literal to base type widening
    - Union/intersection distribution

- [x] **Basic type inference** (Phase 5.2)
  - Literals: string, number, boolean, null
  - Type keywords: string, number, boolean, void, any, never, etc.
  - Union/Intersection types
  - Variable declarations (from initializer/annotation)

- [x] **Symbol type resolution** (Phase 5.2)
  - Link symbols to declarations
  - Resolve identifier types via symbol table
  - Type alias support

- [x] **Function type inference** (Phase 5.3)
  - FunctionType struct for function types
  - Parameter types and names
  - Return type inference
  - Function declarations, function types, type aliases
  - Optional parameters and rest parameters

### In Progress

- [ ] **Generic types**
  - Type parameters
  - Type arguments
  - Instantiation

### Priority: Medium

- [ ] **Object type checking**
  - Property access
  - Method signatures
  - Index signatures

- [ ] **Type narrowing**
  - Type guards (typeof, instanceof)
  - Control flow analysis

## Blocked

(none)

## Notes

- Phase 5.1-5.3 completed 2026-01-01
- 125 Rust tests passing (98 base + 27 checker)
- CheckerState with type inference, assignability, and function types working
- Arrow function expression parsing fully implemented
- Explored typescript-go for architectural patterns (documented in TYPE_CHECKER_MINDMAP.md)
