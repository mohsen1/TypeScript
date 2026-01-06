# Solver Track A: Core Logic (Generics & Inference)

## Mission
Implement the mathematical engine for type inference and generic instantiation. Focus on algorithms, not business rules.

## Scope
**Files:** `src/solver/infer.rs`, `src/solver/instantiate.rs`, `src/solver/operations.rs`

**Independence:** HIGH - Mostly interacts with TypeId and TypeInterner, minimal dependencies on other solver logic.

## Current Status
🟡 **In Progress** - Inference foundation, instantiation, and constraint tests are in place; integration and occurs-check remain.

## Tasks

### Phase 1: Inference Variables Foundation
- [x] Implement `InferenceContext` using `ena` unification table
  - [x] Create `InferenceVar` type (wraps ena's InferenceVariable)
  - [x] Implement `new_inference_var()` -> InferenceVar
  - [x] Implement `unify(var1, var2)` using ena's union-find
- [ ] Add tests for basic unification
  - [x] Test: `unify(T, number)` then resolve T -> number
  - [x] Test: `unify(T, U)` then `unify(U, string)` -> both resolve to string
  - [ ] Test: Circular unification detection (occurs-check)

### Phase 2: Generic Instantiation
- [ ] Implement `instantiate_generic(type: TypeId, args: &[TypeId]) -> TypeId`
  - [x] Handle `Array<T>` + `[number]` -> `Array<number>`
  - [ ] Handle `Promise<T>` instantiation (needs generic `Ref` args)
  - [x] Cache instantiations to avoid duplicates
- [x] Add substitution logic
  - [x] Walk type structure replacing type parameters with concrete types
  - [ ] Handle nested generics: `Map<K, Array<V>>` (needs generic `Ref` args)
- [ ] Tests for instantiation
  - [x] Test: `Array<T>` with T=number -> `Array<number>`
  - [ ] Test: `Map<K,V>` with K=string, V=number
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
- [ ] Expose public API for other solver modules
  - `infer_call_signature(fn_type, args) -> TypeId`
  - `infer_generic_function(fn, args) -> TypeId`
- [ ] Add comprehensive test suite
  - Test function call inference: `identity<T>(x: T) => x` with number
  - Test array methods: `[1,2,3].map(x => x.toString())`
  - Test generic class instantiation

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
