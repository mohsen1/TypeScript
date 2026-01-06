
# Migration Plan: TypeScript Compiler → Rust via WebAssembly (LSP)

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

## Tasks

Our focus is to make wasm Language Service Protocol (LSP) complete

- go-to-def
- find refs
- completions
- ... add more tasks (Ask Gemini when needed)



### Current Status 

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
