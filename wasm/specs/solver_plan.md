# Solver Track Plan (Core Type Engine)

## Mission
High-performance semantic solver and inference engine aligned with WASM architecture.

## Scope
Files: `wasm/src/solver/*`, `wasm/src/interner.rs` (string interning), `wasm/specs/SOLVER.md`.

## Current Status
- Lowering, inference, compat, and diagnostics are implemented and tested.
- TypeKey is POD with side-table IDs (lists/shapes/conditional/mapped/template); QueryDatabase/QueryCache entry points exist.
- Sharded string interner is Arc-backed; solver hot paths use `resolve_atom_ref` to avoid per-lookup allocations.
- Remaining work is performance tuning (allocation churn, lookup hot paths) and richer built-in type behavior.

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

## Success Criteria
- Solver remains lock-contention free in parallel builds.
- TypeKey representation is allocation-light and cache-friendly.
- Benchmarks show steady improvements toward TS-Go parity.
