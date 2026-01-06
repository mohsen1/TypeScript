
# Migration Plan: TypeScript Compiler → Rust via WebAssembly (Emitter)

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

### Goal
100% accurate JS output and Source Maps.

## Tasks

Our focus is to make wasm emitter complete

- ⏳ Split `emit_node` into modules - structure ready, splitting deferred until needed
- ⏳ CommonJS exports - **IN PROGRESS** (~11 tests)
  - ✅ Added `__esModule` marker (correctly excludes `export =` cases)
  - ✅ Added enum and namespace support to export collection
  - ✅ Fixed `exports.X = void 0;` initialization emission
    - Root cause: Parser wraps exports in EXPORT_DECLARATION nodes
    - collect_export_names now checks export_clause field
  - ⬜ **REMAINING**: Suppress `exports.X = X;` when file has `export =`
    - Currently emits both `exports.C = void 0;` AND `exports.C = C;`
    - Should only emit initialization when `export =` present
- ⬜ Comment preservation in emit - ~3 tests
- ⬜ Parse error tolerance (some tests skipped) - ~2 tests
- ... add more tasks (Ask Gemini when needed)



### Current Status (12,408 tests)
| Baseline | Compiler (100 sample) | Conformance | Crash Rate |
|----------|----------------------|-------------|------------|
| .errors.txt | **81.8%** (63/77 subset) | 33.8% (1,741/5,157) | 0.05% |
| .js emit | **61.8%** (47/76 subset) ↑ | ~3% | 0.05% |


## Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh
```
