
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
   - [x] Traverse JSX nodes in LSP resolver for references/completions
   - Add more expression types as needed (template literals, JSX, etc.)
   - [x] Add import/export handling for cross-file navigation

3. **Multi-File Support**
   - [x] Extend to handle cross-file references
   - [x] Implement project-wide find references (named/default imports)
   - [x] Namespace import member references (`import * as ns`)
   - [x] Re-export chains for named + export * (`export { foo } from`, `export * from`)
   - [ ] Namespace re-exports (`export * as ns`) member mapping

#### Testing

Latest run:
- `./wasm/test.sh` ❌ (fails in `src/emitter_transform_integration_tests.rs` for CommonJS auto-detect; pre-existing on rust)

## Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh
```
