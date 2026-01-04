# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. The migration follows the "Strangler Fig"
pattern: Rust components progressively replace TypeScript modules while the
compiler remains fully functional at every step.

## Guiding Principles

1. **Never Break the Build** – Every commit must pass `hereby runtests-parallel`.
2. **Iterate in Small Slices** – One function, one module at a time.
3. **Test Before & After** – Existing test suite is the source of truth.
4. **Performance Parity First** – Match TS speed before optimizing.

---

# 🚀 PRIORITY ZERO: DESIGN FOR SPEED

**Goal: Beat TypeScript-Go in performance.**

Stop "porting" TypeScript line-by-line. Start **architecting for the hardware**.

## Phase 0.1: Thin Nodes (2-3x Parser Speedup)

Current `Node` enum is sized to largest variant (~200+ bytes). This destroys cache locality.

### TODO
- [ ] Refactor `Node` enum to use 8-byte headers + indices into type-specific arrays
- [ ] Target: `sizeof(Node)` ≤ 16 bytes (8 nodes per 64-byte cache line)
- [ ] Split `NodeArena` into separate vectors per node type

```rust
#[repr(C)]
pub struct NodeHeader {
    kind: SyntaxKind,  // u16
    flags: NodeFlags,  // u16
    data_index: u32,   // index into specific Vec
}
```

## Phase 0.2: Zero-Allocation Scanner (Massive Memory Reduction)

Current scanner does `self.source[...].to_string()` = malloc per token.

### TODO
- [ ] Scanner returns `&str` slices of source, never `String`
- [ ] Integrate `Interner` directly into `Scanner`
- [ ] All identifiers become `Atom` (u32) - O(1) string comparison
- [ ] Zero heap allocations during parsing

## Phase 0.3: Arena-Based Type Checker (O(1) Cleanup)

### TODO
- [ ] Apply "Thin" pattern to `Type` enum (currently huge)
- [ ] Use `bumpalo` or `typed-arena` for Type objects
- [ ] All allocations for `check` go into single arena
- [ ] Deallocation = reset pointer (no `Drop` overhead)

## Phase 0.4: Parallelism (Fearless Concurrency)

### TODO
- [ ] Parse files in parallel with `Rayon`
- [ ] Pipeline: Parse → Bind (parallel) → Merge symbols (sequential) → Check bodies (parallel)
- [ ] Check function bodies in parallel (local inference doesn't affect global scope)

## Phase 0.5: SIMD Scanning (Advanced)

### TODO
- [ ] Use portable SIMD for whitespace/identifier scanning
- [ ] Reference: swc, oxc scanner implementations

## Why Rust Beats Go

| Feature | TypeScript-Go | Rust (This Project) |
|---------|---------------|---------------------|
| Memory | Heap + GC | Arena (O(1) free) |
| Data Layout | Pointers (scattered) | Contiguous arrays (cache-friendly) |
| Strings | GC Strings | Interned Atoms (u32) |
| Concurrency | Goroutines | Rayon (no races on AST) |

---

# REMAINING WORK

## Phase 6: Emitter (60% Complete)

| Metric | Value |
|--------|-------|
| Lines of Code | ~4,500 |
| Tests | 65+ |

### TODO
- [x] Declaration file emission (node filtering, export visibility, type-only imports)
- [x] ES2015+ transforms: arrow function → function expression
- [~] Async/await transforms (helper detection done, AST rewriting pending)
- [~] Generator transforms (helper detection done, state machine pending)
- [~] Module transforms (helper detection done, import/export rewriting pending)

## Phase 7: Language Service (50% Complete)

| Metric | Value |
|--------|-------|
| Lines of Code | ~1,300 |
| Tests | 3 |

### TODO
- [x] Signature help (basic implementation)
- [ ] Context-aware completions (member completions, type completions)
- [ ] Cross-file navigation support
- [ ] Formatting engine
- [ ] Code fixes and refactorings

## Phase 8: Full Rust Mode

- [ ] Remove TypeScript fallbacks
- [ ] Performance optimization pass
- [ ] Memory usage optimization
- [ ] WASM interface optimization (replace JSON with binary protocol)

---

# PROGRESS SUMMARY

| Phase | Component | Lines | Tests | Status |
|-------|-----------|-------|-------|--------|
| 0 | Infrastructure | - | - | ✅ Done |
| 1 | Utilities | ~300 | 21 | ✅ Done |
| 2 | Scanner | ~2,500 | 22 | ✅ Done |
| 3 | Parser | ~5,000 | 100+ | ✅ Done |
| 4 | Binder | ~1,900 | 20+ | ✅ Done |
| 5 | Type Checker | ~23,500 | 485 | ✅ 99% |
| 6 | Emitter | ~4,800 | 80+ | 🟡 70% |
| 7 | Language Service | ~1,500 | 3 | 🟡 55% |
| 8 | Full Rust Mode | - | - | ⬜ Pending |

**Total Rust Code**: ~38,300 lines
**Total Tests**: 590 passing
**Overall Progress**: ~85% of full compiler functionality

---

# MILESTONES

## 2026-01-04: Architecture Fixes
- Fixed 4 BLOCKER/CRITICAL issues from Gemini architecture review
- String literal escaping, optional properties, enum nominal typing, function arity
- 577 tests passing

## 2026-01-04: Language Service Core
- Go-to-definition, find references, rename, quick info all working
- NodeSymbolMap for local symbol resolution
- Diagnostics API implemented

## 2026-01-03: Emitter Foundation
- Source map support (VLQ encoding, inline/external)
- Comment preservation
- JS transform scaffolding (ES2015, async, modules)

## 2026-01-02: Type Checker Complete
- 23,500 lines, 485 tests
- Full structural typing, generics, control flow analysis
- JSX support, discriminated unions, exhaustiveness checking

## 2026-01-01: Parser & Binder Complete
- Full AST with 130+ node types
- Symbol table with scope chain and control flow graph
- All TypeScript syntax supported
