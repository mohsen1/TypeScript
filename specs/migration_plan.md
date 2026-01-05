# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

---

# ✅ COMPLETED

- **Phase 0-5**: Scanner, Parser, Binder, Solver (~77,000 LOC, 1041 tests)
- **Phase 6**: Emitter (ES5 transforms, source maps, .d.ts) - 31% JS baseline
- **Phase 7**: Language Service (60%) - go-to-def, find refs, completions

## Emitter TODOs (for JS baseline 80%+)
- ⬜ Class inheritance (`__extends` helper for `extends`)
- ⬜ CommonJS exports (`module.exports`, `exports.X`)
- ⬜ Parse error tolerance (13 tests skipped)

## Language Service TODOs (40% remaining)
- ⬜ Formatting engine
- ⬜ Code fixes/refactorings

---

# 🎯 CURRENT FOCUS: Phase 8 - Baseline Compatibility

**Goal**: Match TypeScript's test baselines for `tests/cases/compiler`.

### Current Status
| Baseline | Pass Rate | Notes |
|----------|-----------|-------|
| .errors.txt | **61.0%** (47/77) | Focus area |
| .js emit | 0% | Baselines use ES5, we emit ES6+ |

### Next Steps
1. ⬜ Export assignment validation (2309)
2. ⬜ Return type validation (2355)
3. ⬜ Parser semantic errors (1128, 1248)
4. ⬜ Class member resolution (this.x vs x vs ClassName.x)
5. ⬜ RelatedInformation (point to definition sites)

### Blockers Analysis (30 failing tests)
- **Parser errors** (1005, 1068, 1128, 1248): Error recovery gaps
- **Module errors** (2304, 2309): Export assignment validation
- **Type errors** (2339, 2355, 2511): Property access, return type, abstract unions
- **Accessor errors** (1183, 6234, 18045): Accessor-specific validation

---

# Phase 9: Full Rust Mode ⬜

- ⬜ Remove TypeScript fallbacks
- ⬜ Performance benchmarks vs tsc and tsc-go
- ⬜ Memory usage optimization

---

# Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh
```
