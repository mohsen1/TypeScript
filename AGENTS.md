# Zang TypeScript Compiler - Agent Guidelines

## Project Overview

Zang is a high-performance TypeScript compiler written in Rust, designed to be a drop-in replacement for the official TypeScript compiler with improved performance, especially for type checking.

## Architecture Principles

### 1. Lock-Free Type Resolution

The type checker uses lock-free patterns to avoid deadlocks during recursive type resolution:

- **Resolution Stack**: Track cycles via a resolution stack instead of locks
- **Atomic Status Transitions**: Use atomic operations for cache entry states (Pending → Resolving → Resolved/Failed)
- **Snapshot-Based Lookups**: Return copies of cached data, not live references
- **No Locks During Recursion**: Never hold a lock while making recursive calls

### 2. Arena Allocation

Use arena allocation for AST nodes and types:

- `bumpalo` crate for fast bump allocation
- Lifetime-tied allocations for automatic cleanup
- Reduced allocation overhead for deeply nested structures

### 3. String Interning

All identifier strings should be interned:

- Use `StringInterner` from `zang_core`
- Pre-intern TypeScript keywords at startup
- Use `InternedString` for all identifier references

### 4. Parallel Processing

Leverage parallelism where possible:

- `rayon` for parallel file processing
- `dashmap` for concurrent caches
- Lock-free algorithms for shared state

## Crate Structure

```
crates/
├── zang_core/      # Core types: Arena, StringInterner, Span, Symbol
├── zang_parser/    # Lexer and Parser producing AST
├── zang_checker/   # Type checker and binder
└── zang_wasm/      # WASM bindings for browser/Node.js
```

## Code Style

### Naming Conventions

- Use TypeScript-compatible names where applicable (e.g., `SyntaxKind`, `TypeFlags`)
- Prefix internal types with module name when exported
- Use snake_case for Rust, but preserve TypeScript semantics

### Error Handling

- Use `thiserror` for error type definitions
- Return `Option<T>` for operations that may not find a value
- Return `Result<T, E>` for operations that may fail with an error
- Never panic in library code

### Testing

- Unit tests in same file with `#[cfg(test)]` module
- Integration tests in `tests/` directory
- Benchmark tests using `criterion`

## Common Tasks

### Adding a New AST Node

1. Define the struct in `zang_parser/src/ast.rs`
2. Add the `SyntaxKind` variant
3. Update the parser in `zang_parser/src/parser.rs`
4. Add visitor methods if needed

### Adding a New Type

1. Define in `zang_core/src/types.rs` or `zang_checker/src/types.rs`
2. Add type checking logic in `zang_checker/src/checker.rs`
3. Update type resolution in the evaluator

### Adding WASM Bindings

1. Add the function in `zang_wasm/src/lib.rs`
2. Use `#[wasm_bindgen]` attribute
3. Convert Rust types to JSON for JS interop
4. Add TypeScript type definitions

## Performance Guidelines

1. **Avoid Allocations in Hot Paths**: Use arena allocation or pre-allocated buffers
2. **Cache Aggressively**: Type resolution results should be cached
3. **Lazy Evaluation**: Don't compute types until needed
4. **Batch Operations**: Process multiple files in parallel when possible

## Compatibility

The compiler should maintain compatibility with TypeScript's type system:

- Parse all valid TypeScript syntax
- Report errors at the same locations as tsc
- Produce identical type checking results

## Building

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Build WASM
./scripts/build-wasm.sh

# Run benchmarks
cargo bench
```

## Contributing

1. Create a feature branch
2. Write tests for new functionality
3. Ensure `cargo clippy` passes
4. Ensure `cargo fmt` has been run
5. Submit a pull request with a clear description
