
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

#### The "Transient Scope" Trap (Checker State) - ⏳ IN PROGRESS

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

**Progress (2026-01-06):**

**Phase 1: Binder Infrastructure (Completed)**
- [x] Added `Scope` and `ScopeId` structures to `binder.rs`
- [x] Updated `ThinBinderState` with persistent scope system:
  - `pub scopes: Vec<Scope>` - Persistent scopes for querying
  - `pub node_scope_ids: FxHashMap<u32, ScopeId>` - Maps AST nodes to scopes
  - `current_scope_id: ScopeId` - Tracks current scope during binding
- [x] Implemented `resolve_identifier(arena, node_idx) -> Option<SymbolId>` API
  - Enables stateless checking by querying scope info without traversal order
  - Walks up AST to find enclosing scope, then walks scope chain
- [x] Integrated persistent scope management into binding:
  - `enter_scope` / `exit_scope` now maintain both legacy and persistent scopes
  - Symbols added to persistent scope table during binding
  - File-level scope created as root persistent scope

**Phase 2: Checker Integration (Completed)**
- [x] Updated `get_type_of_identifier` in `thin_checker.rs`:
  - Now calls `binder.resolve_identifier()` FIRST (stateless approach)
  - Falls back to legacy `lookup_local()` for compatibility
  - Maintains both code paths during transition
- [x] Successfully compiles with new stateless approach
- [x] Binder tests pass (13/13) - scope resolution working correctly
- [x] Changes committed and pushed to `checker-track`

**Architecture Achievement:**
✅ **Stateless checking is now enabled!**
- Checker can resolve identifier types without scope stack dependency
- No longer tied to traversal order
- Foundation for lazy checking and incremental compilation is complete

**Next Phase (Future Work):**
- [ ] Remove legacy `lookup_local()` fallback once fully validated
- [ ] Remove `local_scope_stack` entirely from `CheckerContext`
- [ ] Implement lazy function checking for LSP
- [ ] Measure and document performance improvements
- [ ] Add comprehensive tests for stateless resolution

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
