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

### Completed (Phase 5.4)

- [x] **Generic types**
  - Type parameters with proper symbols
  - Type arguments via TypeReference handling
  - Type instantiation with mapper

### In Progress (Phase 5.5)

- [x] **Object type checking**
  - Property access via get_property_type
  - Object literals infer properties
  - Type literals with members
  - Interface member resolution
  - Method signatures as function types

### Completed (Phase 5.6)

- [x] **Type narrowing**
  - typeof guards: narrow_type_by_typeof, narrow_type_by_typeof_negation
  - Nullable guards: get_non_nullable_type, get_type_with_facts
  - Union type filtering based on type flags

### Completed (Phase 5.7)

- [x] **Generic call expression inference**
  - Type parameter scoping for function signatures
  - Explicit type arguments: `identity<number>(42)`
  - Type argument inference from argument types
  - Type instantiation for return types
  - Parser support for call expressions with type arguments: `expr<T>(args)`
  - Element access expression parsing: `expr[index]`

### Completed (Phase 5.8)

- [x] **instanceof type guards**
  - narrow_type_by_instanceof() narrows to target class type
  - narrow_type_by_instanceof_negation() excludes target from union
  - could_be_instanceof() filters out primitive types
  - is_definitely_instanceof() uses assignability

### Next Up

- [ ] **Control flow based type narrowing**
  - Use flow nodes created by binder
  - Narrow types based on if/while conditions

- [ ] **Class type checking**
  - Instance types
  - Constructor types

## Blocked

(none)

## Notes

- Phase 5.1-5.8 completed 2026-01-01
- 163 Rust tests passing
- CheckerState with type inference, assignability, function types, generics, object types, type narrowing, and generic call inference
- Parser now handles assignment expressions, unary operators (typeof, void, delete, await), and call expressions with type arguments
- Binder flow analysis infrastructure for if/while statements
- Explored typescript-go for architectural patterns (documented in TYPE_CHECKER_MINDMAP.md)
