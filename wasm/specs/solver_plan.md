# Solver Track Plan (Core Type Engine)

## Mission
High-performance semantic solver and inference engine aligned with WASM architecture.

## Scope
Files: `wasm/src/solver/*`, `wasm/src/interner.rs` (string interning), `wasm/specs/SOLVER.md`.

## Current Status
- Status: Active.
- Lowering, inference, compat, and diagnostics are implemented and tested.
- TypeKey is POD with side-table IDs (lists/shapes/conditional/mapped/template); QueryDatabase/QueryCache entry points exist.
- Sharded string interner is Arc-backed; solver hot paths use `resolve_atom_ref` to avoid per-lookup allocations.
- Future work: performance tuning (allocation churn, lookup hot paths) and richer built-in type behavior as needed.

## Highest-Impact Next Tasks
- [x] Make TypeKey POD with side-table slices
  - Replace Vec-heavy variants with index slices stored in interner side tables.
  - Use SmallVec for tiny lists where beneficial.
- [x] Move conditional/mapped types to side-table IDs
  - Remove Box allocations from `TypeKey` and intern the payloads separately.
- [x] Shard the global string interner
  - Replace single `Interner` map with sharded buckets or DashMap.
  - Keep Atom stable and thread-safe.
- [x] Reduce atom resolution allocations in solver hot paths
  - Add `resolve_atom_ref` and Arc-backed sharded interner storage.
  - Use it for numeric/property checks to avoid string cloning.
- [x] Incremental/query layer prototype
  - Implement a Salsa-backed TypeDatabase or query wrapper.
  - Thread through `lower`, `evaluate`, `infer`, `subtype` entry points.
- [x] Diagnostics depth improvements
  - Extend union mismatch reporting in `solver/diagnostics.rs`.
  - Avoid resolve_atom churn when not needed.
- [x] Assignability cleanup
  - Route all assignability through CompatChecker; remove TODO in `SubtypeChecker::is_assignable_to`.
  - Add tests for strict/unsound toggles.
- [x] Benchmarks
  - Add microbench for subtype/evaluate/infer to `./wasm/bench.sh`.
- [x] Introduce SmallVec (or stack-first buffers) for short union/intersection/member lists
  - Reduce Vec churn in interner normalization and hot-path unions.
- [x] Add fast-path property lookup for large object shapes
  - Cached per-shape map in TypeInterner; wired into subtype/infer/property access.
- [x] Expand array/tuple method inference beyond `any` placeholders
  - Added method signatures for map/filter/concat/at/reduce and iterator helpers.
- [x] Add microbench for property lookup and union/intersection normalization
  - Added benchmarks in solver_bench for cached property lookup and normalization.
- [x] Reduce Vec churn in hot union/intersection paths
  - Added union2/union3/intersection2 helpers and switched common two-member unions to stack-first buffers.
- [x] Add coverage for union/intersection normalization with unknown
  - Assert unknown dominates unions and is identity for intersections.
- [x] Allow empty array assignability to optional tuples
  - Treat never[] as assignable to tuples with only optional elements.
- [x] Include undefined for optional tuple index access
  - Tuple indexing now unions optional elements with undefined.
- [x] Resolve tuple rest index element types
  - Tuple indexing now returns rest element types instead of rest containers.
- [x] Expand keyof for tuples with rest tuples
  - Tuple keyof now includes indices from rest tuple expansions.
- [x] Reject non-integer/negative tuple indices
  - Tuple index access now yields undefined for negative or fractional indices.
  - String numeric tuple keys validate i64 parsing and reject non-integer values.
- [x] Add mapped type coverage for primitive keyof (number)
  - Validate mapped types over `keyof number` produce expected boolean properties.
- [x] Add mapped type coverage for primitive keyof (boolean/symbol)
  - Validate mapped types over `keyof boolean` and `keyof symbol` produce expected properties.
- [x] Add mapped type coverage for primitive keyof (bigint)
  - Validate mapped types over `keyof bigint` produce expected properties.
- [x] Enforce tuple/array assignment rule
  - Allow tuple-to-array; reject array-to-tuple except empty arrays to optional/empty tuples.
  - Added compat-layer assignability coverage for tuple/array rules.
- [x] Respect type parameter constraints in overlap checks
  - Binary comparison overlap now considers generic constraints for disjoint primitives.
- [x] Add overlap coverage for unconstrained/union-constrained type params
  - Binary comparison allows unconstrained generics and rejects disjoint constrained unions.
- [x] Add overlap coverage for any/unknown/never/template literals
  - Binary comparison handles top/bottom types and template literals.
- [x] Add index signature property consistency coverage
  - Source properties must satisfy target index signatures even with source index signatures.
- [x] Support split accessor property variance
  - Getter types are covariant; setter types are contravariant in property assignability.
- [x] Add constructor void exception coverage for construct signatures
  - Construct signatures returning values are assignable to `new () => void`.
- [x] Add best common type inference for array literals
  - Prefer a supertype element when all entries are assignable to it.
- [x] Add primitive boxing assignability coverage
  - Allow primitive -> wrapper interface, reject wrapper -> primitive.
- [x] Add unique symbol assignability coverage
  - Unique symbols are nominal and only subtype themselves plus `symbol`.
- [x] Add coverage for global Function type assignability
  - Callables assignable to Function; Function not assignable to specific signatures.
- [x] Add mapped type key remapping coverage (`as never`)
  - Remapped keys returning `never` are filtered from mapped outputs.
- [x] Add keyof contravariance coverage for intersections
  - `keyof (A & B)` unions keys and stays assignable from `keyof A`.

## Success Criteria
- Solver remains lock-contention free in parallel builds.
- TypeKey representation is allocation-light and cache-friendly.
- Benchmarks show steady improvements toward TS-Go parity.
