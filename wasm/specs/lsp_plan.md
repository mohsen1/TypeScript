
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

- go-to-def
- find refs
- completions
- ... add more tasks (Ask Gemini when needed)



### Current Status 

- You don't have an LSP module yet. Create it to allow Track 3 to start.

    ```bash
    mkdir -p wasm/src/lsp
    touch wasm/src/lsp/mod.rs
    # Add "pub mod lsp;" to lib.rs
    ```

- More
...to be filled...

## Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh
```
