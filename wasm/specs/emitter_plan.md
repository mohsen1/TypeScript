
# Migration Plan: TypeScript Compiler → Rust via WebAssembly (Emitter)

**See [EMITTER_ACHIEVEMENTS.md](./EMITTER_ACHIEVEMENTS.md) for comprehensive summary.**

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

### Goal
100% accurate JS output and Source Maps for supported features.

## Current Status

Note:
**Further baseline improvement requires checker/binder track** (type-checking capabilities).

## What's Next?
1. Integrate LoweringPass into public API (lib.rs exports) [done]
2. Implement remaining directive handlers (arrow, async, modules) [partial: arrow/async done; AMD/UMD/System wrappers added]
3. Deprecate old API, make transforms required [partial: emit/emitModern now use two-phase pipeline; LoweringPass handles CommonJS auto-detect]
4. Expand transform system to more node types
5. Implement System/AMD/UMD module formats (if needed) [done]
6. Public API integration (export LoweringPass) [done - covered by #1]
7. Deprecate inline transform logic (breaking change)
8. More unit test if there are gaps [added parity tests; auto-detect/export-assignment coverage]
9. Emit ES6 class heritage clauses (extends) [done]
10. Class ES5 transform: emit try/throw statements and parenthesized expressions [done]
11. CommonJS export star (export * from) support [done]


## Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh

# Real-world benchmarks
./wasm/bench.sh real_world_bench
```
