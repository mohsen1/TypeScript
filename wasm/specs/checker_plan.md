
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

### Immediate Priorities
- [ ] **Fix Atom Refactor Compilation Errors** (High Priority)
  - Fix `solver/lower.rs` (intern strings from AST)
  - Fix `solver/diagnostics.rs` (resolve atoms for error messages)
  - Fix `solver/subtype.rs` (property name comparisons)
- [ ] **Define TypeDatabase Trait** (Preparation for Salsa)

**Status:** 🚧 **Broken Build**. The `TypeKey` refactor (String -> Atom) is half-finished.
**Context:** You changed `TypeKey` to use `Atom` (u32) instead of `Arc<str>`, which broke the solver logic that expects strings. This is the **highest priority** task in the entire repo.

**Step 1 (Fix Build):**
*   **Goal:** Fix ~40 compilation errors in `wasm/src/solver/`.
*   **Focus:**
    *   `solver/lower.rs`: Update `lower_literal_type` and `lower_identifier_type` to use `interner.intern_string()`.
    *   `solver/diagnostics.rs`: Update `TypeFormatter` to resolve Atoms back to strings using `interner.resolve_atom()` before printing.
    *   `solver/intern.rs`: Ensure `TypeInterner` exposes a thread-safe `resolve_atom` method.

**Step 2 (The "Salsa Gap"):**
*   **Goal:** Define the `TypeDatabase` trait to prepare for incremental compilation.
*   **Action:** Create `wasm/src/solver/db.rs`. Define the query interface so the solver stops accessing raw data structures directly.





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
