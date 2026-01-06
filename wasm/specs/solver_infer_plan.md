# Solver Track A: Core Logic (Generics & Inference)

## Mission
Implement the mathematical engine for type inference and generic instantiation. Focus on algorithms, not business rules.

## Scope
**Files:** `src/solver/infer.rs`, `src/solver/instantiate.rs`, `src/solver/operations.rs`

**Independence:** HIGH - Mostly interacts with TypeId and TypeInterner, minimal dependencies on other solver logic.

## Current Status
🟢 **Complete** - Integration tests cover array mapping inference plus application/object constraints.

## Tasks

### Phase 1: Inference Variables Foundation
- [x] Implement `InferenceContext` using `ena` unification table
  - [x] Create `InferenceVar` type (wraps ena's InferenceVariable)
  - [x] Implement `new_inference_var()` -> InferenceVar
  - [x] Implement `unify(var1, var2)` using ena's union-find
- [x] Add tests for basic unification
  - [x] Test: `unify(T, number)` then resolve T -> number
  - [x] Test: `unify(T, U)` then `unify(U, string)` -> both resolve to string
  - [x] Test: Circular unification detection (occurs-check)

### Phase 2: Generic Instantiation
- [x] Implement `instantiate_generic(type: TypeId, args: &[TypeId]) -> TypeId`
  - [x] Handle `Array<T>` + `[number]` -> `Array<number>`
  - [x] Handle `Promise<T>` instantiation (generic `Application` args)
  - [x] Cache instantiations to avoid duplicates
- [x] Add substitution logic
  - [x] Walk type structure replacing type parameters with concrete types
  - [x] Handle nested generics: `Map<K, Array<V>>` (generic `Application` args)
- [x] Tests for instantiation
  - [x] Test: `Array<T>` with T=number -> `Array<number>`
  - [x] Test: `Map<K,V>` with K=string, V=number
  - [x] Test: Nested generics

### Phase 3: Constraint Solving
- [x] Implement constraint system
  - [x] Lower/upper bounds tracked per inference var
  - [x] Constraints merged on var unification
- [x] Implement `resolve_constraints() -> Result<(), Error>`
  - [x] Check for conflicts (upper bound not assignable to lower bound)
  - [x] Finalize inference variables to concrete types
- [x] Tests for constraints
  - [x] Test: `T extends string` -> T can be string literal
  - [x] Test: Conflicting bounds error
  - [x] Test: Multiple bounds intersection

### Phase 4: Integration
- [x] Expose public API for other solver modules
  - [x] `infer_call_signature(fn_type, args) -> TypeId`
  - [x] `infer_generic_function(fn, args) -> TypeId`
- [ ] Add comprehensive test suite
  - [x] Test function call inference: `identity<T>(x: T) => x` with number
  - [x] Test array methods: `[1,2,3].map(x => x.toString())`
  - [x] Test generic class instantiation
  - [x] Test application parameter inference: `Promise<T>` with `Promise<number>`
  - [x] Test object property inference: `{ value: T }` with `{ value: string }`

### Phase 5: Structural Constraints
- [x] Constrain Application args when bases match
- [x] Constrain object properties and index signatures
- [x] Constrain tuple elements

## Architecture Notes
- Use `ena` crate for union-find data structure
- All types are TypeId (u32 index), no String allocation
- Inference is demand-driven: only instantiate when needed
- Cache results in TypeInterner to deduplicate

## Success Criteria
- `cargo test solver::infer` passes all tests
- Can infer type arguments for generic functions
- Can resolve conditional types (future, but foundation ready)
- Zero TypeScript-specific business logic in this module
