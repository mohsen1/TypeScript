# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

## Session log

see SESSION_LOG.md -- always amended with each session's work

---

# 🎯 CURRENT FOCUS: Phase 8 - Baseline Compatibility

**Goal**: 100% match on TypeScript's test baselines.

### Current Status (100-file samples)
| Baseline | Compiler | Conformance | Crash Rate |
|----------|----------|-------------|------------|
| .errors.txt | **77.9%** (60/77) | 23.3% (21/90) | 0% |
| .js emit | **60.5%** (46/76) | 17.0% (15/88) | 0% |

### Work Process

⚠️ **CRITICAL: For each task, ALWAYS:**
1. **BEFORE**: `node scripts/ask-gemini.mjs "How should I implement [task]?"` - get guidance
2. **IMPLEMENT**: Write code, run tests, add tests
3. **AFTER**: `node scripts/ask-gemini.mjs --review wasm/src/[file].rs` - get review

---

# ✅ COMPLETED PHASES

## Phase 0-5: Core Compiler (~77,000 LOC, 1041 tests)
- Scanner, Parser, Binder, Solver

## Phase 6: Emitter
- ✅ ES5 class transform (IIFE with prototype methods)
- ✅ ES5 arrow function transform (function + `_this` capture)
- ✅ ES5 enum transform (IIFE with reverse mapping)
- ✅ ES5 namespace transform (IIFE with qualified names)
- ✅ ES5 async/await transform (`__awaiter`/`__generator` helpers)
- ✅ ES5 private fields transform (WeakMap pattern)
- ✅ ES5 destructuring transform (temp vars)
- ✅ ES5 block scoping (let/const → var)
- ✅ CommonJS module transform (preamble, require, exports)
- ✅ Source map support (SourceWriter with UTF-16 columns)
- ✅ EmitContext for transform state management

## Phase 7: Language Service (60%)
- go-to-def, find refs, completions

---

# ⬜ REMAINING WORK

## Emitter TODOs (to reach 80%+ JS baseline)

**High Priority:**
1. ⬜ Symbol property emit (ES5SymbolProperty tests failing)
2. ⬜ Ambient declaration handling (skip `declare enum/namespace`)
3. ⬜ Export assignment emit (`export =` and `export default`)
4. ⬜ System.register module format

**Medium Priority:**
5. ⬜ Decorators (`__decorate` helper)
6. ⬜ `for-of` iterator downlevel
7. ⬜ Spread/rest (`__spread`/`__rest` helpers)
8. ⬜ Comment preservation in emit

**Low Priority:**
9. ⬜ Parse error tolerance
10. ⬜ JSDoc preservation

## Checker TODOs (to reach 80%+ errors baseline)
- ⬜ Symbol type checking (errors 2403, 2554)
- ⬜ Property access from index signature (error 4111)
- ⬜ Ambient module patterns (errors 2305, 5061, 2819)
- ⬜ Various missing error codes (see test failures)

## Language Service TODOs (40% remaining)
- ⬜ Formatting engine
- ⬜ Code fixes/refactorings
- ⬜ Incremental builds

---

# Phase 9: Polish ⬜
- ⬜ 100% baseline compatibility
- ⬜ Benchmarking suite (three.js, typescript itself)
- ⬜ Near 100% Rust test coverage

# Phase 10: Full Rust Mode ⬜
- ⬜ Remove TypeScript fallbacks
- ⬜ Delete old CheckerState, ParserState
- ⬜ Performance benchmarks vs tsc and tsc-go

# Phase 11: Release ⬜
- ⬜ Own repo (mohsen1/tsc-rust)
- ⬜ NPM + Cargo packaging
- ⬜ CI/CD pipeline
- ⬜ Documentation

---

# Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs
node scripts/baseline-test-rust.mjs conformance

# Build WASM
./wasm/build-wasm.sh
```
