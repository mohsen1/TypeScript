# Solver Track A: Core Logic (Generics & Inference)

## Mission
Implement the mathematical engine for type inference and generic instantiation. Focus on algorithms, not business rules.

## Scope
**Files:** `src/solver/infer.rs`, `src/solver/instantiate.rs`, `src/solver/operations.rs`

**Independence:** HIGH - Mostly interacts with TypeId and TypeInterner, minimal dependencies on other solver logic.

## Current Status
🟢 **Complete** - Integration tests cover union/optional inference plus application/object/tuple/index constraints.

## Tasks

### Phase 1: Inference Variables Foundation
- [x] Implement `InferenceContext` using `ena` unification table
  - [x] Create `InferenceVar` type (wraps ena's InferenceVariable)
  - [x] Implement `new_inference_var()` -> InferenceVar
  - [x] Implement `unify(var1, var2)` using ena's union-find
  - [x] Store inference type param names as Atom (no Arc<str> allocations)
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
  - [x] Use Atom keys in TypeSubstitution to avoid Arc<str> allocations
- [x] Tests for instantiation
  - [x] Test: `Array<T>` with T=number -> `Array<number>`
  - [x] Test: `Map<K,V>` with K=string, V=number
  - [x] Test: Nested generics

### Phase 3: Constraint Solving
- [x] Implement constraint system
  - [x] Lower/upper bounds tracked per inference var
  - [x] Constraints merged on var unification
  - [x] Store constraints in Vec by var id to avoid HashMap overhead
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
- [x] Add comprehensive test suite
  - [x] Test function call inference: `identity<T>(x: T) => x` with number
  - [x] Test array methods: `[1,2,3].map(x => x.toString())`
  - [x] Test generic class instantiation
  - [x] Test application parameter inference: `Promise<T>` with `Promise<number>`
  - [x] Test object property inference: `{ value: T }` with `{ value: string }`
  - [x] Test tuple element inference: `[T, T]` with `[number, number]`
  - [x] Test index signature inference: `{ [key: string]: T }` with `{ [key: string]: number }`
  - [x] Test union source inference: `{ value: T }` with `{ value: number } | { value: string }`
  - [x] Test optional union inference: `T | undefined` with `number`
  - [x] Test rest parameter inference: `(...args: T[])` with `number, string`
  - [x] Test default type params: `<T = string>(x?: T)` with no args
  - [x] Test default from prior param: `<T, U = T>(x: T)` with `number`
  - [x] Test constraint fallback: `<T extends number>(x?: T)` with no args
  - [x] Test constraint violation: `<T extends string>(x: T)` with `number`
  - [x] Test constraint from prior param: `<T, U extends T>(x: T, y: U)` with `string`

### Phase 5: Structural Constraints
- [x] Constrain Application args when bases match
- [x] Constrain object properties and index signatures
- [x] Constrain tuple elements
- [x] Constrain union members to target
- [x] Constrain optional union targets
- [x] Use Atom for discriminant property names in narrowing
- [x] Compare property access names via Atom to avoid resolve_atom churn
- [x] Store subtype failure property names as Atom for diagnostics
- [x] Avoid allocation when constraining non-nullish union targets
- [x] Respect type parameter shadowing in instantiation scopes
- [x] Use FxHashMap for substitution and inference maps
- [x] Infer generics for callable overload signatures
- [x] Validate rest parameter argument types in call resolution
- [x] Treat rest parameters as optional for min argument count
- [x] Validate generic call argument count and non-generic params
- [x] Validate generic calls after inference for defaulted params
- [x] Use FxHashSet in inference visited/dedup
- [x] Handle tuple rest parameters in call resolution
- [x] Constrain tuple rest elements during inference
- [x] Add tuple rest inference coverage for rest params and rest arguments
- [x] Infer variadic tuple type params from rest arguments
- [x] Recheck argument counts after inference for rest tuple defaults/constraints
- [x] Add defaulted rest tuple call coverage for count and optional behavior
- [x] Infer rest tuple type params inside tuple parameter types
- [x] Infer tuple rest placeholders from rest tuple arguments
- [x] Add tuple rest expansion subtyping coverage
- [x] Handle tuple rest expansion for tuple-to-array subtyping
- [x] Validate tuple-to-array bounds during inference resolution
- [x] Allow union upper bounds during inference resolution
- [x] Validate object and function bounds during inference resolution
- [x] Validate generic application bounds during inference resolution
- [x] Validate callable bounds during inference resolution
- [x] Allow object keyword bounds during inference resolution
- [x] Validate index signature bounds during inference resolution
- [x] Add negative index signature bounds coverage
- [x] Refine number index property checks during bounds validation
- [x] Match numeric literal name checks to TypeScript canonical `Number.toString` behavior
- [x] Add numeric literal name bounds coverage for exponent and non-canonical forms
- [x] Honor readonly properties and index signatures during bounds validation
- [x] Allow bivariant method parameter checks for bounds validation
- [x] Add readonly and method variance bounds coverage
- [x] Use assignability checks for bounds validation in inference resolution
- [x] Add assignability-based bounds coverage for bivariant function params
- [x] Infer index signature type params from object literal properties
- [x] Infer property type params from source index signatures
- [x] Allow index signatures to satisfy named properties in subtype checks
- [x] Use canonical numeric literal checks for index signature properties in subtyping
- [x] Add subtype tests for index signatures satisfying named properties
- [x] Cover non-canonical numeric names in index signature subtyping tests
- [x] Enforce index signature consistency during bounds validation

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
