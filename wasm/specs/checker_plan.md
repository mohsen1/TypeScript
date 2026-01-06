
# Migration Plan: TypeScript Compiler → Rust via WebAssembly (Checker)

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**


# Files

src/solver/, src/thin_checker.rs, src/checker/ (legacy removal).

# Goal

Pass tests/cases/compiler.

## Tasks

Our focus is to make wasm checker complete


### 🚨 URGENT: Critical Architectural Fixes (MUST DO FIRST)

These issues block incremental compilation and fast LSP queries.

#### The "Transient Scope" Trap (Checker State)

**Problem:** The checker manually manages scope stacks inside checker logic:
```rust
// In ThinCheckerState
pub fn push_local_scope(&mut self) { ... }
pub fn add_local(&mut self, name: String, ...) { ... }
```

**Impact:** 
- Ties Checker to specific **traversal order**
- Cannot implement "Lazy Checking" (e.g., "Check function 'foo' right now because LSP asked")
- If you jump straight to `foo`, the scope_stack is empty
- **Result:** We will NEVER achieve incremental compilation or fast LSP queries with this design

**Action Required:**
- [ ] Checker must be **stateless regarding scope**
- [ ] Delete `local_scope_stack` from `CheckerContext`
- [ ] Query the Binder: `Binder::get_scope_for_node(node_id)` 
- [ ] Or ensure Binder resolved all identifiers to `SymbolId`s before Checker runs
- [ ] Checker asks: *"What is the symbol for 'x' at node 123?"* - not "what's on my stack"

#### TypeKey Refactor (Shared with Solver)

**Problem:** See SOLVER.md - `Arc<str>` in type keys destroys performance.

**Action Required:**
- [ ] Coordinate with Solver track to use `Atom` (u32) instead of `Arc<str>`
- [ ] All type operations must use interned strings

### Integrate Salsa for Incremental Queries

**Problem:** Manual `RwLock<HashMap>` interner blocks incremental compilation.

**Action Required:**
- [ ] **Add `salsa` crate** to `wasm/Cargo.toml` dependencies
- [ ] Define query interface using Salsa macros:
```rust
#[salsa::query_group(TypeDatabaseStorage)]
pub trait TypeDatabase {
    #[salsa::interned]
    fn intern_type(&self, key: TypeKey) -> TypeId;
    fn resolve_symbol_type(&self, symbol: SymbolId) -> TypeId;
    fn check_subtype(&self, sub: TypeId, sup: TypeId) -> bool;
}
```
- [ ] Replace `RwLock<HashMap>` with Salsa's `#[salsa::interned]`
- [ ] All solver/checker components accept `&dyn TypeDatabase`
- [ ] **Benefits:** automatic memoization, incremental recomputation, cycle detection

- 🔄 Move expression type computation to solver/operations.rs (incremental)
- 🔄 Use NodeView API instead of raw arena lookups (incremental)
- ⬜ Deprecate checker/types in favor of solver/types
- ⬜ Various missing error codes (see test failures)
- ... add more tasks (Ask Gemini when needed)



### Current Status (12,408 tests)
| Baseline | Compiler (100 sample) | Conformance | Crash Rate |
|----------|----------------------|-------------|------------|
| .errors.txt | **81.8%** (63/77 subset) | 33.8% (1,741/5,157) | 0.05% |
| .js emit | **60.5%** (46/76 subset) | ~3% | 0.05% |


## Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh
```
