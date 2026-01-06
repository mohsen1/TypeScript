# Solver Track A: Core Logic (Generics & Inference)

## Mission
Implement the mathematical engine for type inference and generic instantiation. Focus on algorithms, not business rules.

## Scope
**Files:** `src/solver/infer.rs`, `src/solver/unify.rs`, `src/solver/instantiate.rs`

**Independence:** HIGH - Mostly interacts with TypeId and TypeInterner, minimal dependencies on other solver logic.

## Current Status
🟢 **Ready to Start** - TypeKey refactor is complete, APIs are stable.

## Tasks

### Phase 1: Inference Variables Foundation
- [ ] Implement `InferenceContext` using `ena` unification table
  - Create `InferenceVar` type (wraps ena's InferenceVariable)
  - Implement `new_inference_var()` -> InferenceVar
  - Implement `unify(var1, var2)` using ena's union-find
- [ ] Add tests for basic unification
  - Test: `unify(T, number)` then resolve T -> number
  - Test: `unify(T, U)` then `unify(U, string)` -> both resolve to string
  - Test: Circular unification detection

### Phase 2: Generic Instantiation
- [ ] Implement `instantiate_generic(type: TypeId, args: &[TypeId]) -> TypeId`
  - Handle `Array<T>` + `[number]` -> `Array<number>`
  - Handle `Promise<T>` instantiation
  - Cache instantiations to avoid duplicates
- [ ] Add substitution logic
  - Walk type structure replacing type parameters with concrete types
  - Handle nested generics: `Map<K, Array<V>>`
- [ ] Tests for instantiation
  - Test: `Array<T>` with T=number -> `Array<number>`
  - Test: `Map<K,V>` with K=string, V=number
  - Test: Nested generics

### Phase 3: Constraint Solving
- [ ] Implement constraint system
  - `add_constraint(var: InferenceVar, bound: TypeId, kind: BoundKind)`
  - BoundKind: Upper (extends) vs Lower (super)
  - Store constraints in Vec<(InferenceVar, TypeId, BoundKind)>
- [ ] Implement `resolve_constraints() -> Result<(), Error>`
  - Check for conflicts (upper bound not assignable to lower bound)
  - Finalize inference variables to concrete types
- [ ] Tests for constraints
  - Test: `T extends string` -> T can be string literal
  - Test: Conflicting bounds error
  - Test: Multiple bounds intersection

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
