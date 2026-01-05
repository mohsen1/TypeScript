# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

## Session log

see SESSION_LOG.md -- always amended with each session's work

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
| .errors.txt | **66.2%** (51/77) | Focus area |
| .js emit | **36.8%** (28/76) | Fixed baseline comparison bug |

### Next Steps

**Type Checking (26 failing tests)**
1. ✅ Export assignment validation (2309, 2304)
2. ✅ Setter parameter validation (1052, 1053)
3. ⬜ Return type validation (2355) - function must return a value
4. ⬜ Abstract class instantiation (2511) - cannot create instance of abstract
5. ⬜ Static member access from instance (2662) - `this.staticProp` in static method
6. ⬜ Abstract property validation (2715, 2729) - abstract in constructor
7. ⬜ Accessor return type inference (7023) - implicit any in getter

**Parser Semantic Errors**

8. ⬜ Declaration expected (1128) - after certain tokens
9. ⬜ Const modifier on class members (1248) - `const` invalid on properties
10. ⬜ Accessor body in ambient context (1183) - no body in declare class
11. ⬜ Accessor in ambient context ES5 (18045) - accessors need ES5+

**Advanced Diagnostics**

12. ⬜ RelatedInformation - point to definition sites for context
13. ⬜ Accessor diagnostic hints (6234) - "did you mean to call it?"

### Blockers Analysis (26 failing tests)
| Category | Codes | Tests | Notes |
|----------|-------|-------|-------|
| Parser errors | 1005, 1068, 1128, 1248 | 6 | Error recovery gaps |
| Type errors | 2339, 2355, 2511, 2662 | 12 | Property access, returns, abstract |
| Accessor errors | 1183, 6234, 18045 | 4 | Ambient context, hints |
| Abstract members | 2715, 2729, 2416, 2540 | 4 | Abstract property handling |

---

# Phase 9: Finishing up all TODOs ⬜

- ⬜ 100% baseline in all aspects
- ⬜ all todos left from previous phases
- ⬜ todos in code
- ⬜ missing unit tests

# Phase 10: Full Rust Mode ⬜

- ⬜ Remove TypeScript fallbacks
- ⬜ Performance benchmarks vs tsc and tsc-go
- ⬜ Memory usage optimization


# Phase 11: Prepare for Release ⬜

## Strategy
- **Track upstream**: Mirror TypeScript releases (5.x → 6.x)
- **Language features**: 100% compatible—no less, no more
- **API/CLI**: Match tsc behavior; extra flags allowed (e.g., `--wasm-threads`)

## TODOs
- ⬜ Upstream sync workflow (track `microsoft/TypeScript` releases)
- ⬜ Compatibility test suite (run against TS test baselines on each release)
- ⬜ Version alignment (match TS version numbers, e.g., `@mohsen1/typescript@5.7.0`)
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
