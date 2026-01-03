# Rust Migration Performance Opportunities

This document outlines advanced performance optimization opportunities for the TypeScript Rust/WASM migration, building upon the lessons learned from the Go migration and incorporating state-of-the-art techniques from modern Rust-based tooling (Oxc, Biome, rust-analyzer) and recent academic research.

## 1. Memory Management & Allocation

### Arena Allocation (Bump Allocation)
*   **Concept:** Use a single, contiguous block of memory for all objects with the same lifetime (e.g., all AST nodes for a file).
*   **Implementation:** Use crates like `bumpalo` or a custom allocator like `oxc_allocator`.
*   **Benefit:** Allocation is a simple pointer increment. Deallocation is a single operation (resetting the pointer). This eliminates the overhead of the system allocator and significantly improves cache locality.
*   **Reference:** Oxc uses this to achieve 10-100x speedups over traditional JS-based tools.

### Zero-Copy AST
*   **Concept:** AST nodes should store references (`&str` or `Span`) back to the original source buffer rather than allocating new strings for identifiers or literals.
*   **Implementation:** Use lifetimes to ensure the source buffer outlives the AST. For cross-file references, use string interning (see below).
*   **Benefit:** Drastically reduces memory allocations and pressure on the garbage collector (in WASM) or heap.

### String Interning (Atoms)
*   **Concept:** Store unique strings once in a global or thread-local table and refer to them via a small integer ID or a pointer-sized "Atom".
*   **Implementation:** Crates like `string-cache` or `compact_str`.
*   **Benefit:** Fast equality checks (pointer/ID comparison) and reduced memory footprint for common identifiers like `prototype`, `length`, etc.

## 2. Data-Oriented Design (DOD)

### Index-Based References
*   **Concept:** Replace 64-bit pointers with 32-bit indices (`u32`) into a central array or arena.
*   **Implementation:** Instead of `Box<Node>`, use `NodeId(u32)`.
*   **Benefit:** Reduces the size of every node by 4 bytes (on 64-bit systems). Smaller nodes mean more nodes fit in a CPU cache line, leading to fewer cache misses.

### Struct-of-Arrays (SoA)
*   **Concept:** Instead of an array of large structs, use multiple arrays of primitive fields.
*   **Implementation:** If you frequently iterate over only one field of a `Symbol` (e.g., its `flags`), store all `flags` in a contiguous `Vec<SymbolFlags>`.
*   **Benefit:** Maximizes cache efficiency for specific compiler phases that only need a subset of data.

### Field Reordering & Bit-Packing
*   **Concept:** Order struct fields to minimize padding and use bit-fields for booleans/enums.
*   **Implementation:** Use `#[repr(C)]` or `#[repr(packed)]` carefully, or simply order fields from largest to smallest.
*   **Benefit:** Reduces the memory footprint of the most common objects (Nodes, Symbols, Types).

## 3. Architectural Patterns

### Query-Based Incrementality (Salsa)
*   **Concept:** Model the compiler as a set of pure functions (queries) whose results are cached.
*   **Implementation:** Use the `salsa` crate (used by `rust-analyzer`).
*   **Benefit:** Enables extremely fine-grained incremental compilation. When a user types a character, only the affected queries are re-evaluated, making the IDE experience near-instant.

### Resilient & Recoverable Parsing
*   **Concept:** The parser should never "give up" on an error. It should produce a "Green Tree" (CST) that includes "Bogus" nodes for invalid syntax.
*   **Implementation:** Follow the `rowan` or `Biome` architecture.
*   **Benefit:** Essential for IDE features. It allows the compiler to provide type-checking and completions even in the presence of syntax errors.

## 4. Parallelism & Concurrency

### Phase-Level Parallelism
*   **Concept:** While TypeScript project references handle project-level parallelism, Rust can parallelize *within* a project.
*   **Implementation:** Use `rayon` to parallelize independent tasks like:
    *   Parsing multiple files in parallel.
    *   Binding symbols across different modules.
    *   Emitting JS/DTS files.
*   **Benefit:** Fully utilizes multi-core CPUs even for single-project builds.

### Work-Stealing Schedulers
*   **Concept:** Use a work-stealing scheduler to balance the load of type-checking tasks.
*   **Benefit:** Prevents "long tail" issues where one thread is stuck on a massive file while others are idle.

## 5. Low-Level Optimizations

### SIMD-Accelerated Lexing
*   **Concept:** Use SIMD instructions (SSE, AVX, NEON) to scan for whitespace, comments, and token boundaries.
*   **Implementation:** Inspired by `simdjson`.
*   **Benefit:** Can speed up the lexing phase by 3-5x.

### Fast Hashing (xxh3 / FxHash)
*   **Concept:** Use non-cryptographic, high-performance hash functions for HashMaps.
*   **Implementation:** `rustc-hash` (FxHash) or `xxhash-rust`.
*   **Benefit:** Significant speedup for symbol lookups and type caching.

## 6. Recommended Tooling & Crates

*   **`bumpalo`**: Fast arena allocation.
*   **`rayon`**: Easy data parallelism.
*   **`salsa`**: Incremental computation framework.
*   **`rowan`**: Library for lossless syntax trees (Red/Green trees).
*   **`compact_str`**: Memory-efficient string representation.
*   **`hashbrown`**: High-performance HashMap implementation (now in std, but often faster to use directly).
*   **`parking_lot`**: Faster, more compact synchronization primitives than `std::sync`.

## 7. Scientific Literature & Further Reading

*   **"Three Architectures for a Responsive IDE"**: Blog post by matklad (author of rust-analyzer) detailing the query-based approach.
*   **"Literature survey on improving type checker efficiency" (M Staal, 2023)**: Covers modern techniques for type-checking performance without changing the language.
*   **"TaskProf2: A Parallelism Profiler" (PLDI 2019)**: Useful for identifying serialization bottlenecks in parallel compilers.
