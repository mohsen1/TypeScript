# Solver Track C: The Bridge (Binder Integration)

## Mission
Convert AST nodes to TypeId representations. Bridge the gap between syntax (ThinNode) and semantics (TypeKey).

## Scope
**Files:** `src/solver/lower.rs`, integration with `src/thin_binder.rs`

**Independence:** MEDIUM - Needs stable TypeKey definitions from Track A. Heavy AST interaction.

## Current Status
🟡 **In Progress** - Phase 5 interface merging and declaration-space filtering added; BigInt literal test blocked by parser.

## Tasks

### Phase 1: Basic Type Lowering
- [x] Implement `lower_type_annotation(node: NodeIndex) -> TypeId`
  - Handle primitive types: `number`, `string`, `boolean`, `void`, `any`, `unknown`
  - Use interner to deduplicate: `interner.intern_type(TypeKey::Intrinsic(...))`
- [x] Implement literal types
  - String literals: `"hello"` -> `TypeKey::Literal(atom)`
  - Number literals: `42` -> special handling (store as Atom? or separate pool?)
  - Boolean literals: `true`, `false`
- [x] Tests for basic lowering
  - Test: `number` annotation -> Intrinsic(Number)
  - Test: `"hello"` -> Literal with correct Atom
  - Test: Verify deduplication (same type -> same TypeId)
  - [ ] Add BigInt literal type test once `123n` parses in type position

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
- [x] Tests for complex types
  - [x] Test: Object type with multiple properties
  - [x] Test: Nested objects
  - [x] Test: Union/intersection normalization
  - [x] Test: Tuple optional/rest/named elements

### Phase 3: Function Signatures
- [x] Implement function type lowering
  - `(x: string) => number` -> TypeKey::Function
  - Store signature: params (Vec<Param>), return type (TypeId)
- [x] Handle optional and rest parameters
  - `(x?: string)` -> Param { optional: true }
  - `(...args: string[])` -> Param { rest: true, type: Array<string> }
- [x] Tests for function types
  - [x] Test: Simple function signature
  - [x] Test: Optional parameters
  - [x] Test: Rest parameters
  - [x] Test: Overloaded signatures (Vec<Signature>)

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

### Phase 5: Interface Merging & Declaration Spaces
- [x] Implement interface merging
  - Multiple `interface Foo` declarations merge into one type
  - Merge properties, handle conflicts
- [ ] Implement declaration space separation
  - [x] Type space vs value space (handle same name for class/type)
  - [ ] Module augmentation support
- [x] Tests for merging
  - [x] Test: Two interface declarations merge
  - [x] Test: Conflicting property types error
  - [x] Test: Method overloads accumulate

### Phase 6: Integration
- [ ] Connect to binder
  - Binder provides SymbolId -> AST node mapping
  - Lower uses that to find type annotation nodes
- [ ] Implement `get_type_of_symbol(symbol: SymbolId) -> TypeId`
  - Check if already lowered (cache)
  - Otherwise, lower the symbol's type annotation
  - Store in cache for next lookup
- [ ] Add comprehensive tests
  - Test: Lowering full source file
  - Test: Cross-module type references
  - Test: Circular type references (handle gracefully)

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
