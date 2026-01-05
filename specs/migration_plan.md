# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

---

# ✅ COMPLETED

- **Phase 0-5**: Scanner, Parser, Binder, Solver (~77,000 LOC, 1041 tests)
- **Phase 6**: Emitter (ES5 transforms, source maps, .d.ts) - **39.5% JS baseline**
- **Phase 7**: Language Service (60%) - go-to-def, find refs, completions

## Recent Emitter Improvements
- ✅ Fixed baseline comparison script to extract JS portion correctly
- ✅ Instance property initializers → `this.x = value;` in constructor
- ✅ Distinguish `implements` vs `extends` in heritage clauses

## Emitter TODOs (for JS baseline 80%+)
- ⬜ Class inheritance - emit base class name (not just `_super`)
- ⬜ CommonJS exports (`"use strict"`, `module.exports`, `exports.X`)
- ⬜ Parse error tolerance (some tests skipped)

## Language Service TODOs (40% remaining)
- ⬜ Formatting engine
- ⬜ Code fixes/refactorings

---

# 🎯 CURRENT FOCUS: Phase 8 - Baseline Compatibility

**Goal**: Match TypeScript's test baselines for `tests/cases/compiler`.

### Current Status
| Baseline | Pass Rate | Notes |
|----------|-----------|-------|
| .errors.txt | **63.6%** (49/77) | Focus area |
| .js emit | **39.5%** (30/76) | ES5 IIFE emit, class transforms |

### Next Steps
1. ✅ Export assignment validation (2309, 2304)
2. ⬜ Return type validation (2355)
3. ⬜ Parser semantic errors (1128, 1248)
4. ⬜ Class member resolution (this.x vs x vs ClassName.x)
5. ⬜ RelatedInformation (point to definition sites)

### Blockers Analysis (28 failing tests)
- **Parser errors** (1005, 1068, 1128, 1248): Error recovery gaps
- **Type errors** (2339, 2355, 2511): Property access, return type, abstract unions
- **Accessor errors** (1183, 6234, 18045): Accessor-specific validation

---

# Phase 9: Full Rust Mode ⬜

- ⬜ Remove TypeScript fallbacks
- ⬜ Performance benchmarks vs tsc and tsc-go
- ⬜ Memory usage optimization


# Phase 10: Prepare for Release ⬜

## Strategy
- **Track upstream**: Mirror TypeScript releases (5.x → 6.x)
- **Language features**: 100% compatible—no less, no more
- **API/CLI**: Match tsc behavior; extra flags allowed (e.g., `--wasm-threads`)

## TODOs
- ⬜ Upstream sync workflow (track `microsoft/TypeScript` releases)
- ⬜ Compatibility test suite (run against TS test baselines on each release)
- ⬜ Version alignment (match TS version numbers, e.g., `5.7.0-rust`)
- ⬜ CLI parity audit (`tsc --help` flags, exit codes, output format)
- ⬜ API compatibility layer (programmatic API matches `typescript` npm)
- ⬜ Packaging for npm (`@aspect/tsc-rust` or similar)
- ⬜ Packaging for cargo (`tsc-rust` crate)
- ⬜ Pre-built WASM binaries for major platforms
- ⬜ CI/CD release pipeline (GitHub Actions)
- ⬜ Documentation (migration guide, API docs, README)
- ⬜ Branding & naming
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
