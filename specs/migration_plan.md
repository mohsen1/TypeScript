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

**Total**: ~74,350 Rust LOC | 1006 tests passing

## Solver (specs/SOLVER.md) ✅
Complete implementation in `wasm/src/solver/`:
- Type interning (O(1) equality via TypeId)
- Semantic subtyping with coinductive recursion
- Full type lowering (typeof, keyof, this, conditional, mapped, infer)
- Generic instantiation and constraint-based inference
- Contextual typing, discriminated union narrowing
- Lazy diagnostics with structured args

## Phase 6: Emitter (75%)
- ✅ Declaration emit, ES2015+ transforms, CommonJS, async/await
- 🟡 Partial: Generator transforms

## Phase 7: Language Service (60%)
- ✅ Go-to-definition, find references, completions, signature help, cross-file navigation
- ⬜ Formatting engine, code fixes/refactorings

---

# 🎯 CURRENT FOCUS: Phase 8 - Baseline Compatibility

**Goal**: Match TypeScript's test baselines for `tests/cases/compiler`.

## Test Pass Rates
| Category | Pass Rate |
|----------|-----------|
| compiler | 99.9% (6389/6393) |
| conformance | 99.98% (5654/5655) |
| fourslash | 0% (not started) |

## Baseline Comparison (First 100 tests)
| Baseline | Pass Rate | Blockers |
|----------|-----------|----------|
| .errors.txt | **32.5%** (25/77) | Missing function validation, parser errors |
| .js emit | 0% | Emitter format mismatch |

## Completed
- ✅ Class member validation (2389-2391) for method/constructor overloads
- ✅ Parser error code infrastructure

## Missing Error Codes
**High Priority**:
- 2389, 2391 - Top-level function overload validation (not just class methods)
- 2369 - Parameter type errors
- 2414 - Class member modifiers

**Parser Validation**:
- 1005, 1068, 1128, 1440 - Parser semantic errors

## Next Steps
1. ⬜ Function validation errors (2389-2391) for top-level function declarations
2. ⬜ Add parser semantic errors (1068, 1128, 1440) in ThinParser
3. ⬜ Error elaboration ("...because property 'x' has type 'string' not 'number'")
4. ⬜ RelatedInformation (point to definition sites)
5. ⬜ Match emitter output format

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
