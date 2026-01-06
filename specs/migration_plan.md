# Migration Plan: TypeScript Compiler → Rust via WebAssembly

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

## Session log

see SESSION_LOG.md -- always amended with each session's work

# ✅ COMPLETED

- **Phase 0-5**: Scanner, Parser, Binder, Solver (~77,000 LOC, 1041 tests)
- **Phase 6**: Emitter (ES5 transforms, source maps, .d.ts) - **59.2% JS baseline**
- **Phase 7**: Language Service (60%) - go-to-def, find refs, completions

## Recent Checker Improvements
- ✅ Callable interface type lowering (function-like interfaces)
- ✅ Function type annotations for error 2355
- ✅ Error 2676: accessor abstract consistency (get/set must both be abstract or both non-abstract)
- ✅ Error 1253: abstract members in non-abstract class
- ✅ Accessor body checks in ambient contexts (1183)
- ✅ Error 1248: const keyword on class members (moved from parser to checker)

## Code Cleanup
- ✅ Deleted ~35k lines of dead code (old fat-node parser, emitter, checker, services)

## Recent Emitter Improvements
- ✅ Fixed baseline comparison script to extract JS portion correctly
- ✅ Instance property initializers → `this.x = value;` in constructor
- ✅ Distinguish `implements` vs `extends` in heritage clauses
- ✅ Parameter properties (`public x, private y`) → `this.x = x; this.y = y;`
- ✅ Constructor overloads: only emit implementation, skip signatures
- ✅ Single-line empty block detection (preserve `{ }` vs `{\n}`)
- ✅ Class extends: emit base class name, _super parameter, derived constructor with _super.apply
- ✅ Combined getter/setter pairs into single `Object.defineProperty` calls
- ✅ Skip abstract accessors in emit
- ✅ Source order emit for methods/accessors
- ✅ Declare variable skip (`declare const foo: number;` → empty)
- ✅ Arrow function `this` capture (`var _this = this;`) for base and derived classes
- ✅ Destructuring transform (`let { x } = obj;` → `var _a = obj, x = _a.x;`)

## Emitter TODOs (for JS baseline 80%+)
- ⬜ CommonJS exports (`"use strict"`, `module.exports`, `exports.X`) - ~11 tests
- ⬜ Comment preservation in emit - ~3 tests
- ⬜ Parse error tolerance (some tests skipped) - ~2 tests

## Language Service TODOs (40% remaining)
- ⬜ Formatting engine
- ⬜ Code fixes/refactorings
- ⬜ Incremental Builds

---

# 🎯 CURRENT FOCUS: Phase 8 - Baseline Compatibility

**Goal**: 100% match on TypeScript's test baselines.

### Current Status (12,408 tests)
| Baseline | Compiler (100 sample) | Conformance | Crash Rate |
|----------|----------------------|-------------|------------|
| .errors.txt | **79.2%** (61/77 subset) | 33.8% (1,741/5,157) | 0.05% |
| .js emit | **59.2%** (45/76 subset) | ~3% | 0.05% |

### Work Process

⚠️ **CRITICAL: For each task, ALWAYS:**
1. **BEFORE**: `node scripts/ask-gemini.mjs "How should I implement [task]?"` - get guidance
2. **IMPLEMENT**: Write code, run tests. add tests
3. **AFTER**: `node scripts/ask-gemini.mjs --review wasm/src/[file].rs` - get review

This catches design issues early and ensures consistent code quality.

### Next Steps

**Type Checking (25 failing tests)**
1. ✅ Export assignment validation (2309, 2304)
2. ✅ Setter parameter validation (1052, 1053)
3. ✅ Return type validation (2355) - function must return a value (basic types, function types)
4. ✅ Abstract class instantiation (2511) - basic case (union types need more work)
5. ✅ Static member access from instance (2662) - `foo` → "Did you mean 'C.foo'?"
6. ✅ Abstract property in constructor (2715) - `this.abstractProp` in ctor
7. ⬜ Property used before initialization (2729) - needs dataflow analysis
8. ⬜ Accessor return type inference (7023) - implicit any in getter

**Parser Semantic Errors**

8. ⬜ Declaration expected (1128) - after certain tokens
9. ✅ Const modifier on class members (1248) - `const` invalid on properties
10. ✅ Accessor body in ambient context (1183) - no body in declare class/interface/type
11. ✅ Abstract in non-abstract class (1253) - abstract members need abstract class
12. ⬜ Accessor in ambient context ES5 (18045) - `accessor` keyword needs ES5+

**Advanced Diagnostics**

13. ⬜ RelatedInformation - point to definition sites for context
14. ⬜ Accessor diagnostic hints (6234) - "did you mean to call it?"

### Blockers Analysis (26 failing tests)
| Category | Codes | Tests | Notes |
|----------|-------|-------|-------|
| Parser errors | 1005, 1128 | 4 | Error recovery gaps (1068, 1248 done) |
| Type errors | 2339, 2355, 2511 | 10 | Property access, returns, abstract (2662 done) |
| Accessor errors | 6234, 18045 | 3 | Hints, ES5 target (1183 done) |
| Abstract members | 2729, 2416, 2540 | 3 | Abstract property handling (2715 done) |

### Emit TODOs (40.8% failing → target 80%+)
| Feature | Tests | % | Notes |
|---------|-------|---|-------|
| **Modules** | 1,935 | 33% | `import`/`export` → CommonJS/ESM |
| **let/const** | 205 | 4% | Block scoping → `var` for ES5 |
| **Arrow functions** | 159 | 3% | `=>` → `function` for ES5 |
| **Class fields** | 404 | 7% | Static fields, private `#` |
| **Namespace** | 168 | 3% | IIFE wrapping |
| **Enums** | 145 | 2% | Enum object emit |
| **Decorators** | 98 | 2% | `__decorate` helper |
| **Async/await** | 88 | 2% | `__awaiter` helper |
| **for-of** | 39 | 1% | Iterator downlevel |
| **Spread/rest** | 24 | <1% | `__spread`/`__rest` helpers |
| **Generators** | 26 | <1% | `__generator` helper |

### Type Checking TODOs (top missing codes)
| Code | Count | Description |
|------|-------|-------------|
| TS5107 | 323 | Option requires value |
| TS2322 | 306 | Type not assignable |
| TS2339 | 138 | Property does not exist |
| TS6133 | 123 | Declared but never used |
| TS2304 | 102 | Cannot find name |
| TS2345 | 100 | Argument type mismatch |
| TS2300 | 96 | Duplicate identifier |
| TS2307 | 58 | Cannot find module |
| TS2741 | 47 | Missing property |
| TS7006 | 43 | Implicit any parameter |
| TS1128 | 35 | Declaration expected |

### Quick Wins
- 🔄 TS2322/2345: Improve type assignability checks (accessor type compatibility in progress)
- ⬜ TS2339: Property lookup on union/intersection types  
- ⬜ TS2304: Module resolution, global declarations
- ⬜ TS2300: Duplicate detection in binder
- ⬜ Modules: Start with `export {}` and named exports

---

# Phase 9: Finishing up all TODOs ⬜
- ⬜ 100% baseline in all aspects
- ⬜ all todos left from previous phases
- ⬜ Benchmarking: Create a benchmark suite (e.g., parsing a large library like three.js or typescript itself) to measure actual throughput (MB/s).
- ⬜ Incremental Builds: The ThinNode architecture allows for efficient incremental reparsing, but the "diffing" logic isn't visible yet.
- ⬜ todos in code
- ⬜ missing unit tests and test coverage. aim for near 100% coverage of rust code

# Phase 10: Full Rust Mode ⬜

- ⬜ Remove TypeScript fallbacks
- ⬜ Delete old CheckerState (checker/state.rs) - use only ThinCheckerState
- ⬜ Delete old ParserState (parser_impl.rs) - use only ThinParser
- ⬜ Port Language Service to use ThinNode/ThinChecker APIs
- ⬜ Performance benchmarks vs tsc and tsc-go
- ⬜ Memory usage optimization


# Phase 11: Prepare for Release ⬜

## Strategy
- **Track upstream**: Mirror TypeScript releases (5.x → 6.x)
- **Language features**: 100% compatible—no less, no more
- **API/CLI**: Match tsc behavior; extra flags allowed (e.g., `--wasm-threads`)

## TODOs
- ⬜ Own repo mohsen1/tsc-rust
- ⬜ Upstream sync workflow (track `microsoft/TypeScript` releases)
- ⬜ Compatibility test suite (run against TS test baselines on each release)
- ⬜ Version alignment (match TS version numbers, e.g., `tsc-rust@5.7.0`)
- ⬜ CLI parity audit (`tsc --help` flags, exit codes, output format)
- ⬜ API compatibility layer (programmatic API matches `typescript` npm)
- ⬜ Packaging for npm (`@mohsen1/tsc-rust` or similar)
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
