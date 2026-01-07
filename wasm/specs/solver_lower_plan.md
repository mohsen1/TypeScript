# Solver Track C: The Bridge (Binder Integration)

## Mission
Convert AST nodes to TypeId representations. Bridge the gap between syntax (ThinNode) and semantics (TypeKey).

## Scope
**Files:** `src/solver/lower.rs`, integration with `src/thin_binder.rs`

**Independence:** MEDIUM - Needs stable TypeKey definitions from Track A. Heavy AST interaction.

## Current Status
🟡 **In Progress** - Declaration space separation complete; comprehensive tests expanded; typeof value queries supported.

## Tasks

### Phase 1: Basic Type Lowering
- [x] Implement `lower_type_annotation(node: NodeIndex) -> TypeId`
  - Handle primitive types: `number`, `string`, `boolean`, `void`, `any`, `unknown`
  - Use interner to deduplicate: `interner.intern_type(TypeKey::Intrinsic(...))`
- [x] Implement literal types
  - String literals: `"hello"` -> `TypeKey::Literal(atom)`
  - Number literals: `42` -> special handling (store as Atom? or separate pool?)
  - Boolean literals: `true`, `false`
  - Hex/binary/octal numeric literals map to numeric literal types
  - BigInt literals normalize base prefixes to decimal
  - BigInt literals with base prefixes are tokenized in the scanner
  - Numeric separators are accepted for numeric and bigint literals
  - [x] Scanner flags invalid numeric separators for diagnostics
  - [x] Parser emits diagnostics for invalid numeric separators (6188/6189)
  - [x] Diagnostics highlight the invalid numeric separator position
- [x] Parse and lower `unique symbol` type operator
- [x] Tests for basic lowering
  - Test: `number` annotation -> Intrinsic(Number)
  - Test: `"hello"` -> Literal with correct Atom
  - Test: Verify deduplication (same type -> same TypeId)
  - [x] Add BigInt literal type test once `123n` parses in type position
  - [x] Test: Negative numeric and bigint literal types
  - [x] Test: Hex/binary/octal numeric literal types
  - [x] Test: Hex/binary/octal bigint literal types (normalized)
  - [x] Test: Numeric/bigint literal separators across bases
  - [x] Test: `unique symbol` type operator lowering
  - [x] Test: `keyof` type operator lowering
  - [x] Test: `readonly` type operator lowering

### Phase 2: Complex Type Structures
- [x] Implement object type lowering
  - `{ name: string, age: number }` -> TypeKey::Object
  - Store properties in side table (Vec<Property>)
  - Property: `{ name: Atom, type: TypeId, optional: bool }`
- [x] Implement array and tuple types
  - `string[]` -> TypeKey::Array(element)
  - `[string, number]` -> TypeKey::Tuple(Vec<TupleElement>)
  - Optional/rest/named tuple elements captured in TupleElement flags
- [x] Implement union and intersection types
  - `string | number` -> TypeKey::Union(Slice<TypeId>)
  - `A & B` -> TypeKey::Intersection(Slice<TypeId>)
- [x] Bind infer parameters when lowering conditional types
- [x] Tests for complex types
  - [x] Test: Object type with multiple properties
  - [x] Test: Nested objects
  - [x] Test: Union/intersection normalization
  - [x] Test: Tuple optional/rest/named elements
  - [x] Test: `Array<T>` type reference lowers to array element type
  - [x] Test: `ReadonlyArray<T>` type reference lowers to readonly array
  - [x] Test: `Array<T>` type reference respects resolver shadowing
  - [x] Test: `ReadonlyArray<T>` type reference respects resolver shadowing
  - [x] Test: Conditional type with infer (including constraint)
  - [x] Test: Conditional infer binding in true/false branches

### Phase 3: Function Signatures
- [x] Implement function type lowering
  - `(x: string) => number` -> TypeKey::Function
  - Store signature: params (Vec<Param>), return type (TypeId)
  - [x] Type predicates lower to boolean/void return types
  - [x] Preserve type predicate metadata in function/call signatures
  - [x] Treat `this` parameters as separate `this_type` (not in params list)
- [x] Handle optional and rest parameters
  - `(x?: string)` -> Param { optional: true }
  - `(...args: string[])` -> Param { rest: true, type: Array<string> }
- [x] Tests for function types
  - [x] Test: Simple function signature
  - [x] Test: Optional parameters
  - [x] Test: Rest parameters
  - [x] Test: Overloaded signatures (Vec<Signature>)
  - [x] Test: Type predicates with `this` parameter
  - [x] Test: `asserts this` predicate without type
  - [x] Test: `asserts x` predicate without type (captures predicate metadata)
  - [x] Test: Call signature type predicates in type literals
  - [x] Test: `this` parameter stored separately for function and call signatures
  - [x] Test: `this` parameter variance in subtype checks
  - [x] Test: Contextual typing for callable signatures (params/return/this)
  - [x] Test: Contextual typing skips `this` parameter when indexing args
  - [x] Test: Contextual typing unions callable overloads
  - [x] Test: Contextual typing for variable initializers with annotations
  - [x] Test: Contextual typing selects overloads by call arity
  - [x] Test: Checker lowers generic function type annotations with type params
  - [x] Test: Checker lowers generic function declarations with type params

### Phase 4: Generic Types
- [x] Implement type parameter lowering
  - `<T>` -> create TypeKey::TypeParameter(name: Atom, constraint: Option<TypeId>)
  - `<T extends string>` -> store constraint
- [x] Implement generic type references
  - `Array<T>` where T is a type parameter
  - Track type parameter scope (which generic declaration)
- [x] Tests for generics
  - [x] Test: Generic function declaration
  - [x] Test: Generic class/interface
  - [x] Test: Constrained type parameters
  - [x] Test: Generic type reference preserves type arguments in checker

### Phase 5: Interface Merging & Declaration Spaces
- [x] Implement interface merging
  - Multiple `interface Foo` declarations merge into one type
  - Merge properties, handle conflicts
- [x] Implement declaration space separation
  - [x] Type space vs value space (handle same name for class/type)
  - [x] Module augmentation support
- [x] Tests for merging
  - [x] Test: Two interface declarations merge
  - [x] Test: Conflicting property types error
  - [x] Test: Method overloads accumulate

### Phase 6: Integration
- [x] Connect to binder
  - Binder provides SymbolId -> AST node mapping
  - Lower uses that to find type annotation nodes
- [x] Implement `get_type_of_symbol(symbol: SymbolId) -> TypeId`
  - Check if already lowered (cache)
  - Otherwise, lower the symbol's type annotation
  - Store in cache for next lookup
- [x] Lower typeof type queries with type arguments into applications
- [x] Add comprehensive tests
  - [x] Test: Interface index signature property access (4111)
  - [x] Test: Explicit property bypasses 4111
  - [x] Test: Union with index signature triggers 4111
  - [x] Test: Lowering full source file
  - [x] Test: Cross-module type references
  - [x] Test: Circular type references (handle gracefully)
  - [x] Test: typeof value references in interfaces (qualified and unqualified)
  - [x] Test: typeof lowering uses value resolver (type vs value space)
  - [x] Test: typeof type query with type arguments
  - [x] Test: checker preserves type arguments on typeof type queries
  - [x] Test: checker element access returns array element type
  - [x] Test: checker element access returns tuple element types
  - [x] Element access uses literal index nodes for tuple lookup
  - [x] Contextual typing for array literals respects tuple element expectations
  - [x] Test: checker element access handles string literal properties
  - [x] Test: checker element access handles numeric string indices
  - [x] Test: checker element access respects index signatures
  - [x] Test: checker element access reports missing index signature (7053)
  - [x] Test: checker element access unions literal key types
  - [x] Test: checker element access uses literal key types from identifiers
  - [x] Test: checker element access unions numeric literal tuple indices
  - [x] Test: checker element access reports nullish object (2532)
  - [x] Test: checker element access optional chain unions undefined
  - [x] Test: checker element access unions mixed string/number literal keys
  - [x] Test: checker element access union string index requires signature
  - [x] Test: checker element access union string/number index requires signature
  - [x] Checker: element access assignment reports readonly properties
  - [x] Checker: union readonly assignment checks any union member
  - [x] Test: checker element access assignment reports readonly property (literal key)
  - [x] Test: checker element access assignment respects readonly index signatures
  - [x] Checker: interface extension compatibility respects readonly/optional properties
  - [x] Checker: interface extension compatibility resolves method type parameters
  - [x] Test: interface extension detects readonly property mismatch (2430)
  - [x] Test: interface extension detects optional property mismatch (2430)
  - [x] Test: interface extension accepts matching generic method signatures
- [x] Parser: fix interface member parsing to track progress by token position (avoid skipping consecutive identifiers)
- [x] Perf: avoid extra String clone when lowering negative bigint literals
- [x] Perf: avoid extra allocation when normalizing bigint leading zeros
- [x] Perf: skip separator stripping for base-prefixed numeric literals
- [x] Perf: skip separator stripping for base-prefixed bigint literals

## Architecture Notes
- All strings MUST go through interner.intern_string() -> Atom
- Never allocate String in hot path
- Use side tables (Vec) for variable-size data (object properties, union members)
- TypeKey itself must remain Copy (no heap allocations)
- Cache lowered types: SymbolId -> TypeId map

## Success Criteria
- `cargo test solver::lower` passes all tests
- Can lower all TypeScript type syntax to TypeKey
- Zero string allocations during lowering (only Atoms)
- Handles interface merging correctly
- Integrates cleanly with binder's SymbolId system
