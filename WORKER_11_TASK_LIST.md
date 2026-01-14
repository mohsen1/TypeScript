# Worker 11 Task List

**Branch:** worker-11
**Target branch:** rust
**Focus:** Semantic Solver Implementation (Phase 7.5)

---

## Task Queue

### Current Task
**Task 3:** Implement AST Type Lowering (The Bridge)
- [ ] Implement `lower_type(db, node: NodeIndex) -> Type` query
- [ ] Handle primitives: `SyntaxKind::StringKeyword` -> `Intrinsic::String`
- [ ] Handle literals: Extract text from scanner -> `Literal::String`
- [ ] Handle interfaces: Iterate members, recursively call `lower_type`, return `TypeKey::Object`
- [ ] Handle union types: Flatten and normalize
- [ ] Handle function types: Extract parameters and return type
- [ ] Add tests for lowering in `wasm/src/solver/lower_tests.rs`
- [ ] **Run:** `./wasm/test.sh` to verify
- [ ] **Run:** `./wasm/differential-test/run-conformance.sh --all` and analyze report

**Ready for Merge:** No

---

### Completed Tasks

**Task 1:** Implement TypeKey normalization infrastructure ✓
- [x] Add `salsa`, `ena`, `indexmap`, `bitflags` dependencies to `wasm/Cargo.toml`
- [x] Create `wasm/src/solver/` module structure (mod.rs, db.rs, jar.rs, type_id.rs, type_key.rs, lower.rs, logic.rs, infer.rs) - Already existed
- [x] Define `TypeKey` enum with variants - Already existed in `types.rs`
- [x] Implement `TypeKey::object()` constructor that sorts properties by Atom - Already existed in `intern.rs`
- [x] Implement `TypeKey::union()` constructor that flattens nested unions and sorts by ID - Already existed in `intern.rs`
- [x] Tests for normalization already exist in `intern_tests.rs`

**Task 2:** Implement Salsa Type Interner ✓
- [x] TypeId wrapper exists in `types.rs`
- [x] TypeDatabase trait exists in `db.rs` with all required methods
- [x] QueryDatabase trait extends TypeDatabase with caching layer
- [x] TypeInterner implementation with sharded storage in `intern.rs`
- [x] Comprehensive tests for interning deduplication in `intern_tests.rs` and `db_tests.rs`
- Note: Implementation uses manual query system instead of Salsa (Salsa requires nightly Rust)

---

### Pending Tasks

**Task 4:** Implement Core Subtyping Logic
- [ ] Implement `solve_subtype(db, sub: Type, sup: Type) -> bool` query
- [ ] Handle primitive subtyping: `String <: String`, `Never <: T`, `T <: Unknown`
- [ ] Handle structured objects: Iterate sup properties, binary search in sub, recurse
- [ ] Handle unions: `Union <: T` (ALL parts), `T <: Union` (ANY part)
- [ ] Handle intersections with distributivity rules
- [ ] Implement function variance (contravariant parameters, covariant returns)
- [ ] Add cycle detection tests: `interface A { x: A }; interface B { x: B }; A <: B?`
- [ ] **Run:** `./wasm/test.sh` to verify
- [ ] **Run:** `./wasm/differential-test/run-conformance.sh --all` and analyze report

**Task 5:** Implement Inference and Unification
- [ ] Define `InferenceContext` wrapper around `ena::InPlaceUnificationTable`
- [ ] Implement `instantiate`: Replace `TypeKey::Generic(T)` with `InferenceVar(?0)`
- [ ] Implement `unify(a, b)`: If var, point; if both concrete, call `solve_subtype`
- [ ] Implement bounds checking: `L <: α <: U`
- [ ] Implement contextual typing (reverse inference from expected type)
- [ ] Add tests for generic inference
- [ ] **Run:** `./wasm/test.sh` to verify
- [ ] **Run:** `./wasm/differential-test/run-conformance.sh --all` and analyze report

**Task 6:** Implement Conditional Types and Meta-Types
- [ ] Add `TypeKey::Conditional { check_type, extends_type, true_branch, false_branch }`
- [ ] Implement conditional evaluation with speculative subtyping check
- [ ] Implement distributivity for naked type parameters over unions
- [ ] Handle deferred conditionals for unresolved generics
- [ ] Implement mapped types: `{ [K in Keys]: Transform<K> }`
- [ ] Implement index access types: `T[K]`
- [ ] Add tests for conditional and mapped types
- [ ] **Run:** `./wasm/test.sh` to verify
- [ ] **Run:** `./wasm/differential-test/run-conformance.sh --all` and analyze report

**Task 7:** Compatibility Layer for TypeScript Quirks
- [ ] Implement `solve_subtype` public API with "Lawyer" layer
- [ ] Handle `any` short-circuit (subtype and supertype of everything)
- [ ] Implement bivariant function parameters for legacy mode
- [ ] Implement excess property checking (freshness) for object literals
- [ ] Handle void exception: `() => void` matches `() => string`
- [ ] Add tests for compatibility behaviors
- [ ] **Run:** `./wasm/test.sh` to verify
- [ ] **Run:** `./wasm/differential-test/run-conformance.sh --all` and analyze report

**Task 8:** Solver Integration and Performance
- [ ] Connect solver to existing `thin_checker.rs` as incremental replacement
- [ ] Benchmark memory usage vs Legacy Checker
- [ ] Implement rayon parallelism for file-level type checking
- [ ] Optimize TypeFlags for fast rejection (is_truthy, has_object_structure)
- [ ] Add error propagation: `TypeKey::Error` with poison pill semantics
- [ ] **Run:** `./wasm/differential-test/run-conformance.sh --all` and analyze report
- [ ] **Run:** `./wasm/bench.sh` for performance validation

---

## Progress Notes
- Branch is clean and synced with `origin/rust`
- Tasks 1 and 2 complete: TypeKey normalization and TypeInterner already implemented
- solver/ module has comprehensive implementation with 3239 passing tests
- Working on Task 3: AST Type Lowering (The Bridge)
- All work stays within `wasm/` directory per architecture rules
- Each task includes conformance testing to track progress
