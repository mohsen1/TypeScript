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

# REMAINING WORK

## Before going back to work


1. Split test out of source files into a separate tests directory in was dir. 
2. Without going to deep into a rabbithole find opportunities for splitting large files into smaller ones in was directory

## Phase 6: Emitter (60% Complete)

| Metric | Value |
|--------|-------|
| Lines of Code | ~4,500 |
| Tests | 65+ |

### TODO
- [ ] Declaration file emission (node filtering, export visibility, type-only imports)
- [ ] ES2015+ transforms (class→prototype, arrow→function, destructuring)
- [ ] Async/await → Promise chains
- [ ] Generator → state machine
- [ ] Module transforms (ES→CommonJS, ES→AMD/UMD)

## Phase 7: Language Service (50% Complete)

| Metric | Value |
|--------|-------|
| Lines of Code | ~1,300 |
| Tests | 3 |

### TODO
- [ ] Signature help
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
| 6 | Emitter | ~4,500 | 65+ | 🟡 60% |
| 7 | Language Service | ~1,300 | 3 | 🟡 50% |
| 8 | Full Rust Mode | - | - | ⬜ Pending |

**Total Rust Code**: ~38,300 lines
**Total Tests**: 577 passing
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
