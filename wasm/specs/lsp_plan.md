
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
   - [x] Code action: remove unused import (6133)
   - [x] Code action: add missing property (2339, object literals + `this` in class)
   - [x] Add missing property: preserve single-line trailing commas
   - [x] Add missing property: element access string literals
   - [x] Code action: add missing import (2304, candidate-based)
   - [x] Feed project export candidates into code action context
   - [x] Expose wasm code actions context (diagnostics + import candidates)
   - [x] Missing import: surface default exports and re-exports
   - [x] Missing import: merge named imports into existing declarations
   - [x] Missing import: merge default imports into existing declarations
   - [x] Missing import: use import type in type positions (and skip type-only for values)
   - [x] Signature help: fix active parameter detection (between arguments)
   - [x] Signature help: pick overload by argument count
   - [x] Signature help: select constructor overloads for `new`
   - [x] Signature help: surface JSDoc documentation
   - [x] Signature help: attach @param docs to parameters
   - [x] Extract variable: avoid name collisions in scope
   - [x] Extract variable: avoid cross-scope extractions
   - [x] Extract variable: parenthesize comma expressions
   - [x] Extract variable: block TDZ declarations after insertion
   - [x] Extract variable: handle JSX tag TDZ references
   - [x] Extract variable: handle JSX attribute TDZ references
   - [x] Extract variable: handle JSX children TDZ references
   - [x] Rename: normalize private identifiers (`#name`)

2. **Extend AST Coverage** (if needed)
   - [x] Traverse template expressions in LSP resolver for references/completions
   - [x] Traverse JSX nodes in LSP resolver for references/completions
   - [x] Add more expression/type nodes in resolver (await/yield/as/tagged templates, type annotations)
   - [x] Bind and traverse destructuring patterns for definitions/references
   - [x] Bind class member bodies (methods/accessors/constructors) for local resolution
   - [x] Record class member declarations for definition lookups
   - [x] Avoid treating class members as lexical locals in LSP resolution
   - [x] Resolve class names within class scopes
   - [x] Bind class expressions in initializers for local resolution
   - [x] Find references for class names in class scopes/expressions
   - [x] Bind nested function/class expressions inside complex initializers
   - [x] Bind nested function/class expressions inside if conditions
   - [x] Bind loop/switch condition expressions for local resolution
   - [x] Bind export assignment expressions for local resolution
   - [x] Bind labeled/with statements for local resolution
   - [x] Hoist `var` declarations in LSP resolver scopes
   - [x] Traverse decorator expressions in binder/resolver
   - [x] Bind class static blocks for local resolution
   - [x] Add import/export handling for cross-file navigation

3. **Multi-File Support**
   - [x] Extend to handle cross-file references
   - [x] Implement project-wide find references (named/default imports)
   - [x] Namespace import member references (`import * as ns`)
   - [x] Re-export chains for named + export * (`export { foo } from`, `export * from`)
   - [x] Namespace re-exports (`export * as ns`) member mapping
   - [x] Resolve `.tsx`/`.d.ts`/`.mts`/`.cts` module specifiers

#### Testing

Latest run:
- `./wasm/test.sh` ✅

## Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh
```
