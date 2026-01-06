
# Migration Plan: TypeScript Compiler → Rust via WebAssembly (Emitter)

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

### Goal
100% accurate JS output and Source Maps.

## Tasks

Our focus is to make wasm emitter complete

- ⏳ Split `emit_node` into modules - structure ready, splitting deferred until needed
- ✅ CommonJS exports - **COMPLETED**
  - ✅ Added `__esModule` marker (correctly excludes `export =` cases)
  - ✅ Added enum and namespace support to export collection
  - ✅ Fixed `exports.X = void 0;` initialization emission
    - Root cause: Parser wraps exports in EXPORT_DECLARATION nodes
    - collect_export_names now checks export_clause field
  - ✅ Suppress `exports.X = X;` when file has `export =`
    - Added has_export_assignment flag to ModuleTransformState
    - Detect and set flag in emit_source_file
    - Check flag in emit_export_declaration_commonjs and declaration emitters
    - Now correctly emits only initialization when `export =` present
- 🚧 Comment preservation in emit - **PARTIALLY DONE** (all comments preserved, but baseline regressed 61.8% → 53.9%)
- ✅ Parse error tolerance - **COMPLETED**
  - Added `emit_expression()` function that emits `void 0` for error/unknown nodes
  - Updated all expression emitters to use `emit_expression()` instead of `emit()`
  - Ensures syntactically valid JavaScript even with parse errors (e.g., `var x = void 0;` instead of `var x =;`)
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
