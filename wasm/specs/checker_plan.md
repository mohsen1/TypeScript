
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

- ✅ Fix the Solver Stack Overflow Risk (COMPLETED)
    Added depth tracking to SubtypeChecker:
    - Added `depth: u32` field to struct
    - Initialize to 0 in both constructors
    - Check `depth > 100` after fast paths, return Provisional
    - Increment before recursion, decrement after
    - All tests pass (593/593)
- 🔄 Move expression type computation to solver/operations.rs (incremental)
- 🔄 Use NodeView API instead of raw arena lookups (incremental)
- ⬜ Deprecate checker/types in favor of solver/types
- ⬜ Symbol type checking (errors 2403, 2554)
- ⬜ Property access from index signature (error 4111)
- ⬜ Ambient module patterns (errors 2305, 5061, 2819)
- ⬜ Various missing error codes (see test failures)
- ⬜ Fix tuple subtyping logic (CRITICAL from Gemini review)
    - Currently too permissive: allows extra elements in source
    - TypeScript: `[number, string]` is NOT assignable to `[number]`
    - Need to check if target has rest element, reject extra source elements if not
- ⬜ Fix function parameter variance (MAJOR from Gemini review)
    - Currently bivariant (legacy mode), should be contravariant (strict mode)
    - Consider making strictFunctionTypes the default
- ⬜ Remove unused ref_cache field (MINOR from Gemini review)
    - Currently marked #[allow(dead_code)], not implemented
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
