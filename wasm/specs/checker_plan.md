
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

-  Fix the Solver Stack Overflow Risk
    In `src/solver/subtype.rs`, add a depth check:

    ```rust
    // Add to SubtypeChecker struct
    depth: u32,

    // In check_subtype
    if self.depth > 100 {
        return SubtypeResult::Provisional; // Or Error
    }
    self.depth += 1;
    // ... check ...
    self.depth -= 1;
```
- 🔄 Move expression type computation to solver/operations.rs (incremental)
- 🔄 Use NodeView API instead of raw arena lookups (incremental)
- ⬜ Deprecate checker/types in favor of solver/types
- ⬜ Symbol type checking (errors 2403, 2554)
- ⬜ Property access from index signature (error 4111)
- ⬜ Ambient module patterns (errors 2305, 5061, 2819)
- ⬜ Various missing error codes (see test failures)
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
