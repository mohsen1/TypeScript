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

### Completed (Phase 5.9)

- [x] **Control flow type guard analysis**
  - TypeGuard enum: Typeof, Instanceof, Truthiness variants
  - get_type_guard_from_expression() extracts guards from conditions
  - get_type_guard_from_binary_expression() handles typeof, instanceof, null checks
  - negate_type_guard() for else branches
  - apply_type_guard() applies narrowing based on guard type
  - get_narrowed_type_at_flow() walks flow graph to apply guards
  - guard_applies_to_symbol() checks if guard applies to a specific symbol

### Completed (Phase 5.10)

- [x] **Class type checking**
  - Instance types created with create_class_type
  - Constructor types with construct signatures
  - resolved_return_type on signatures points to instance type
  - get_type_of_new_expression uses construct signature return type

### Completed (Phase 5.11)

- [x] **Index signature parsing**
  - IndexSignatureDeclaration struct in parser.rs
  - parse_index_signature_declaration() parses [key: type]: type
  - Parser test for interface with index signature

### Completed (Phase 5.12)

- [x] **Index signature type checking**
  - IndexInfo struct with key_type, value_type, is_readonly, declaration
  - Interface type construction extracts index signatures
  - Element access expressions use index info for type resolution
  - get_indexed_access_type() finds applicable index signature

### Completed (Phase 5.13)

- [x] **Array and tuple types**
  - ArrayTypeInfo struct with element_type
  - TupleTypeInfo struct with element_types
  - T[] syntax creates array types
  - Array<T> generic syntax creates array types
  - [T, U] tuple type syntax
  - type_to_string() support for array/tuple display

### Completed (Phase 5.14)

- [x] **Type assignability for arrays/tuples**
  - Array to array assignability (element types must be compatible)
  - Tuple to tuple assignability (same length, element-wise compatible)
  - Tuple to array assignability (all tuple elements must be compatible with array element type)

### Completed (Phase 5.15)

- [x] **Readonly array/tuple types**
  - `readonly T[]` syntax via TypeOperator node
  - `ReadonlyArray<T>` generic syntax
  - `readonly [T, U]` tuple syntax
  - is_readonly flag on ArrayTypeInfo and TupleTypeInfo
  - type_to_string shows readonly prefix
  - Assignability: readonly cannot be assigned to mutable

### Completed (Phase 5.16)

- [x] **Rest and optional tuple elements**
  - Optional elements: `[T, U?]` parsed as OptionalType nodes
  - Rest elements: `[T, ...U[]]` parsed as RestType nodes
  - has_optional_elements and has_rest_element flags on TupleTypeInfo
  - Element types extracted properly in checker

### Completed (Phase 5.17)

- [x] **Spread in array literals**
  - SpreadElement parsing in array literals
  - `is_array_element_start()` and `parse_array_element()` functions
  - `get_element_type_of_spread()` extracts element type from arrays/tuples
  - Type inference for `[...arr1, ...arr2]`

### Completed (Phase 5.18)

- [x] **Conditional type evaluation**
  - get_type_of_conditional_type() evaluates `T extends U ? X : Y`
  - Resolves to true_type when check_type is assignable to extends_type
  - Resolves to false_type otherwise
  - Distributes over union types in check position
  - Defers evaluation when check_type contains type parameters
  - type_to_string() support for conditional types
  - type_contains_type_parameter() helper for deferred evaluation

### Completed (Phase 5.19)

- [x] **Template literal types (basic)**
  - Parser support for template literal types in type positions
  - parse_template_literal_head() and parse_template_literal_type_spans()
  - get_type_of_template_literal_type() for type evaluation
  - create_template_literal_type() in TypeArena
  - type_to_string() support for template literal types
  - Simple templates work (`hello`)
  - Note: Complex substitutions need scanner state fixes

### Completed (Phase 5.20)

- [x] **Mapped type evaluation**
  - get_type_of_mapped_type() for type evaluation
  - create_mapped_type() in TypeArena
  - type_to_string() support for mapped types
  - Deferred evaluation (mapped types stored unevaluated)

### Completed (Phase 5.21)

- [x] **Infer types in conditional types**
  - Node::InferType handling in get_type_of_node_worker
  - get_type_of_infer_type() creates type parameter
  - Type alias declarations now set up type parameter scope
  - `type UnwrapPromise<T> = T extends Promise<infer U> ? U : T`

### Completed (Phase 5.22)

- [x] **Keyof type evaluation (basic)**
  - get_keyof_type() for extracting keys from types
  - create_index_type() in TypeArena
  - keyof any = string | number | symbol
  - keyof T (type parameter) = Index type
  - keyof unknown = never
  - Note: Full object literal keyof needs member tracking improvements

### Completed (Phase 5.23)

- [x] **Keyof object types (full)**
  - Extract keys from type literals and interfaces
  - get_symbol checks local_symbols first to avoid ID collision

- [x] **Mapped type instantiation**
  - Evaluate mapped types when applied to concrete types
  - get_keys_from_type for extracting property names
  - Proper type parameter scope handling

- [x] **Infer type pattern matching**
  - infer_from_type for pattern matching against source types
  - type_contains_infer to detect infer type parameters
  - Proper scope handling: infer types added to type_parameter_scope
  - Function type return inference: preserved parent scope

- [x] **Template literal types (full)**
  - Fixed scanner rescan: update parser's current_token after re_scan_template_token
  - Fixed infinite loop: break on EOF in parse_template_literal_type_spans
  - Added bounds checking in substring() and re_scan_template_token()
  - Note: Full instantiation (e.g., `\`hello ${"world"}\`` → "hello world") not yet complete

### Completed (Phase 5.25)

- [x] **Intersection type simplification**
  - Flatten nested intersections
  - Remove duplicates (X & X = X)
  - Handle never (X & never = never)
  - Handle unknown (X & unknown = X)
  - Return single type if only one remains

### Completed (Phase 5.26)

- [x] **Template literal type instantiation**
  - Evaluate template literals with concrete string/number/boolean/bigint types
  - Concatenate static texts with stringified literal values
  - Defer evaluation when substitution contains type parameters

### Completed (Phase 5.27)

- [x] **Union type simplification**
  - Flatten nested unions
  - Remove duplicates (X | X = X)
  - Handle never (X | never = X)
  - Handle any (X | any = any)
  - Return single type if only one remains

### Completed (Phase 5.28)

- [x] **Distributive conditional types**
  - Auto-detect naked type parameters in check position
  - Mark conditionals as distributive when check type is naked
  - Distribute over union types during instantiation

### Completed (Phase 5.29)

- [x] **Index access on arrays/tuples**
  - get_indexed_access_type handles Type::Array (returns element_type)
  - get_indexed_access_type handles Type::Tuple (returns specific element or union)
  - Union type support for indexed access
  - Number literal index returns specific tuple element
  - Number type index returns union of all tuple elements

### Next Up

- [ ] **Property access on unions** (blocked - needs interface type resolution fix)
  - Get common property type across all union members
  - Handle optional properties
  - Note: test_property_access_on_union has infinite loop, needs investigation

## Blocked

- **Property access on unions**: test_property_access_on_union hangs, likely due to infinite loop in interface type resolution

## Notes

- Phase 5.1-5.29 completed 2026-01-03
- 215 Rust tests passing (1 ignored)
- CheckerState with type inference, assignability, function types, generics, object types, type narrowing, generic call inference, control flow type guards, and index signatures
- Parser now handles assignment expressions, unary operators (typeof, void, delete, await), and call expressions with type arguments
- Binder flow analysis infrastructure for if/while statements
- Explored typescript-go for architectural patterns (documented in TYPE_CHECKER_MINDMAP.md)
