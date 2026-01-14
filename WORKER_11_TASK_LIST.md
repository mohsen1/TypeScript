# Worker 11 Task List

**Branch:** worker-11
**Target branch:** rust
**Focus:** Semantic Solver Implementation (Phase 7.5)

---

## Task Queue

### Current Task
**Task 1:** Implement TypeKey normalization infrastructure
- [ ] Add `salsa`, `ena`, `indexmap`, `bitflags` dependencies to `wasm/Cargo.toml`
- [ ] Create `wasm/src/solver/` module structure (mod.rs, db.rs, jar.rs, type_id.rs, type_key.rs, lower.rs, logic.rs, infer.rs)
- [ ] Define `TypeKey` enum with variants: `Intrinsic`, `Literal`, `Object`, `Union`, `Intersection`, `Ref`, `Conditional`, `InferenceVar`
- [ ] Implement `TypeKey::object()` constructor that sorts properties by Atom
- [ ] Implement `TypeKey::union()` constructor that flattens nested unions and sorts by ID
- [ ] Add comprehensive tests for normalization in `wasm/src/solver/type_key_tests.rs`
- [ ] **Run:** `./wasm/test.sh` to verify
- [ ] **Run:** `./wasm/differential-test/run-conformance.sh --all` and analyze report

**Ready for Merge:** No

---

### Pending Tasks

**Task 2:** Implement Salsa Type Interner
- [ ] Define `#[salsa::interned]` struct `Type` in `wasm/src/solver/jar.rs`
- [ ] Create `TypeJar` with queries: `Type`, `lower_type`, `solve_subtype`, `check_expression`, `resolve_symbol`
- [ ] Define `Db` trait with `salsa::DbWithJar<TypeJar>`
- [ ] Implement TypeId wrapper and type flag system
- [ ] Add tests for interning deduplication
- [ ] **Run:** `./wasm/test.sh` to verify
- [ ] **Run:** `./wasm/differential-test/run-conformance.sh --all` and analyze report

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

## Completed Tasks
*None yet - Worker 11 just initialized*

---

## Progress Notes
- Branch is clean and synced with `origin/rust`
- Focused on Phase 7.5: Semantic Solver implementation
- All work stays within `wasm/` directory per architecture rules
- Each task includes conformance testing to track progress
