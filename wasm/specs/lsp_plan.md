
# Migration Plan: TypeScript Compiler → Rust via WebAssembly (LSP)

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**


# Files

src/lsp/ (New), src/thin_binder.rs

# Goal

"Go to Definition" and "Find References".

## Tasks

Our focus is to make wasm Language Service Protocol (LSP) complete

- [x] **Code Action: Organize Imports** (Sort-only first)
- [x] **Diagnostic Integration**: Surface Checker errors in LSP
- [x] **Multi-File Context**: Create `Project` struct to hold multiple source files

#### Remaining Optimizations
   - All major LSP performance optimizations complete!
   - Future: Consider incremental re-parsing on edits

3. **Add More LSP Features**
   - Code actions (complete implementation)

2. **Extend AST Coverage** (if needed)
   - [x] Traverse template expressions in LSP resolver for references/completions
   - Add more expression types as needed (template literals, JSX, etc.)
   - Add import/export handling for cross-file navigation

3. **Multi-File Support**
   - Extend to handle cross-file references
   - Implement project-wide find references

#### Testing

All tests pass (691/691):
```bash
./wasm/test.sh  # ✅ All pass
```

## Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh
```
