# Solver Track Plan (Core Type Engine)

## Mission
High-performance semantic solver and inference engine aligned with WASM architecture.

## Scope
Files: `wasm/src/solver/*`, `wasm/src/interner.rs` (string interning), `wasm/specs/SOLVER.md`.

## Current Status
- Lowering, inference, compat, and diagnostics are implemented and tested.
- TypeDatabase trait exists, TypeInterner is sharded.
- Remaining work is performance and incremental architecture.

## Highest-Impact Next Tasks
- [ ] Make TypeKey POD with side-table slices
  - Replace Vec-heavy variants with index slices stored in interner side tables.
  - Use SmallVec for tiny lists where beneficial.
- [ ] Shard the global string interner
  - Replace single `Interner` map with sharded buckets or DashMap.
  - Keep Atom stable and thread-safe.
- [ ] Incremental/query layer prototype
  - Implement a Salsa-backed TypeDatabase or query wrapper.
  - Thread through `lower`, `evaluate`, `infer`, `subtype` entry points.
- [ ] Diagnostics depth improvements
  - Extend union mismatch reporting in `solver/diagnostics.rs`.
  - Avoid resolve_atom churn when not needed.
- [ ] Assignability cleanup
  - Route all assignability through CompatChecker; remove TODO in `SubtypeChecker::is_assignable_to`.
  - Add tests for strict/unsound toggles.
- [ ] Benchmarks
  - Add microbench for subtype/evaluate/infer to `./wasm/bench.sh`.

## Success Criteria
- Solver remains lock-contention free in parallel builds.
- TypeKey representation is allocation-light and cache-friendly.
- Benchmarks show steady improvements toward TS-Go parity.
