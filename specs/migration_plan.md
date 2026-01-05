# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

---

# ✅ COMPLETED WORK

## Phase 0: Performance Architecture ✅
- **ThinNode**: 16 bytes/node (13x cache improvement from 208B)
- **Zero-Alloc Scanner**: Atom interning, zero-copy accessors
- **Parallelism**: Rayon-based parallel parsing/binding/checking
- **Lazy Diagnostics**: Deferred string formatting, zero waste
- **Solver Operations**: Pure type logic, structured results

## Phases 1-5: Core Compiler ✅
| Component | Lines | Status |
|-----------|-------|--------|
| Utilities | ~300 | ✅ |
| Scanner | ~2,500 | ✅ |
| Parser (ThinParser) | ~11,300 | ✅ 100% pass rate (4483/4483) |
| Binder (ThinBinder) | ~2,900 | ✅ |
| Type Checker + Solver | ~29,300 | ✅ 99% |

**Total**: ~74,350 Rust LOC | 1009 tests passing

## Solver (specs/SOLVER.md) ✅
Complete implementation in `wasm/src/solver/`:
- Type interning (O(1) equality via TypeId)
- Semantic subtyping with coinductive recursion
- Full type lowering (typeof, keyof, this, conditional, mapped, infer)
- Generic instantiation and constraint-based inference
- Contextual typing, discriminated union narrowing
- Lazy diagnostics with structured args

## Phase 7: Language Service (60%)
- ✅ Go-to-definition, find references, completions, signature help, cross-file navigation
- ⬜ Formatting engine, code fixes/refactorings

---

# 🎯 CURRENT FOCUS: Parallel Tracks

## Track A: Phase 8 - Baseline Compatibility

**Goal**: Match TypeScript's test baselines for `tests/cases/compiler`.

### Test Pass Rates
| Category | Pass Rate |
|----------|-----------|
| compiler | 99.9% (6389/6393) |
| conformance | 99.98% (5654/5655) |
| fourslash | 0% (not started) |

### Baseline Comparison (First 100 tests)
| Baseline | Pass Rate | Blockers |
|----------|-----------|----------|
| .errors.txt | **48.1%** (37/77) | Missing parser errors, error elaboration |
| .js emit | 0% | Emitter format mismatch |

### Completed
- ✅ Class/function overload validation (2389, 2390, 2391)
- ✅ Parser error code infrastructure
- ✅ Parser semantic errors (1068, 1440) for class members
- ✅ Parameter property validation (2369)
- ✅ Declare class parsing (skip impl checks for ambient)
- ✅ Numeric method name support (0(), 1(), etc.)

### Next Steps
1. ⬜ Parser semantic errors (1128, additional coverage)
2. ⬜ Error elaboration ("...because property 'x' has type...")
3. ⬜ RelatedInformation (point to definition sites)
4. ⬜ Interface/function type parameter property checks

---

## Track B: Phase 6 - Emitter Completion (75% → 100%)

**Goal**: Complete emitter with performance-first approach.

### Phase 6.1: Study & Exploration (Use Gemini)
Before implementation, analyze for performance opportunities:
- ⬜ **Benchmark current emit** - measure throughput (bytes/sec)
- ⬜ **Profile hot paths** - string building, whitespace, source maps
- ⬜ **Study TypeScript emitter** - identify simplification opportunities
- ⬜ **Gemini review** - ask for emit architecture recommendations
- ⬜ **Explore alternatives**: rope data structures, streaming output, SIMD text processing

### Phase 6.2: Generator Transforms
- ⬜ `function*` syntax and `yield` expressions
- ⬜ State machine generation for ES5 target
- ⬜ Iterator protocol compliance

### Phase 6.3: Output Format Matching
- ⬜ Match TypeScript baseline whitespace/semicolons
- ⬜ Source map accuracy
- ⬜ Declaration file formatting

### Key Files
| Purpose | Location |
|---------|----------|
| ThinEmitter | `wasm/src/thin_emitter.rs` |
| Transforms | `wasm/src/transforms/` |
| Generator transforms | `wasm/src/transforms/async_gen.rs` |

---

# Phase 9: Full Rust Mode ⬜

- ⬜ Remove TypeScript fallbacks
- ⬜ Performance benchmarks vs tsc and tsc-go
- ⬜ Memory usage optimization
- ⬜ WASM interface optimization (binary protocol)

---

# WASM Strategy

## WasmGC (Garbage Collection) 🟢 **Ready Now**
Use `externref` via `wasm-bindgen` for the Language Service API:
```rust
// Instead of JSON serialization:
pub fn get_completions() -> js_sys::Array { ... }
```
- Removes serialization bottleneck at API boundary
- Low risk, high payoff

## Wasm64 (Memory64) 🔴 **Not Ready**
- Experimental flag required in most runtimes
- `wasm-bindgen` friction with 64-bit pointers/BigInt
- **Not needed yet**: ThinNode (16 bytes) fits ~250M nodes in 4GB
- **Decision**: Keep u32 indices, revisit in 12-18 months

---

# Quick Reference

## Commands
```bash
# Tests (Docker)
./wasm/test.sh

# TypeScript
npx hereby runtests-parallel

# Baseline comparison
node scripts/baseline-test-rust.mjs
```

## Key Files
| Purpose | Location |
|---------|----------|
| This plan | specs/migration_plan.md |
| Session log | [specs/SESSION_LOG.md](SESSION_LOG.md) |
| Solver spec | specs/SOLVER.md |
| Unsoundness rules | specs/TS_UNSOUNDNESS_CATALOG.md |
| Architecture | specs/WASM_ARCHITECTURE.md |
| Solver code | wasm/src/solver/ |
| ThinParser | wasm/src/parser/thin_parser.rs |
| ThinChecker | wasm/src/checker/thin_checker.rs |
| Baseline comparison | scripts/baseline-test-rust.mjs |
