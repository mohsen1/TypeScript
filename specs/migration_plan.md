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

**Goal**: 100% match on TypeScript's test baselines.

### Current Status (12,408 tests)
| Baseline | Compiler | Conformance | Crash Rate |
|----------|----------|-------------|------------|
| .errors.txt | 38.6% (2,070/5,360) | 33.8% (1,741/5,157) | 0.05% |
| .js emit | 3.3% (194/5,837) | ~3% | 0.05% |

### Work Process

⚠️ **CRITICAL: For each task, ALWAYS:**
1. **BEFORE**: `node scripts/ask-gemini.mjs "How should I implement [task]?"` - get guidance
2. **IMPLEMENT**: Write code, run tests. add tests
3. **AFTER**: `node scripts/ask-gemini.mjs --review wasm/src/[file].rs` - get review

This catches design issues early and ensures consistent code quality.

### Next Steps

#### First thing first.
The analysis identifies several modules that are effectively dead code. While some "old" components (like `checker/`, `binder.rs`, `parser_impl.rs`) are technically reachable via the legacy `createParser` API, other components have no public entry points in `lib.rs` or are disconnected from the execution graph.

delete this section once done

<details>
### Dead Code Report

The following modules are compiled but never used in the production pipeline (wasm public API):

1.  **`emitter.rs` (Old Emitter)**
    *   **Status**: Dead.
    *   **Reason**: Works on "fat" `Node` AST. The legacy `ParserState` exposed in `lib.rs` has no `emit()` method. The new `ThinParser` uses `thin_emitter.rs` (working on `ThinNode` AST).
    *   **Files**: `emitter.rs`, `emitter_tests.rs`.

2.  **`services/` (Language Service)**
    *   **Status**: Dead.
    *   **Reason**: Although `pub mod services` exists, there is no `createLanguageService` or similar factory exposed in `lib.rs`. The logic is unreachable from the WASM boundary. It also depends on the legacy `checker` which is being phased out.
    *   **Files**: `services/mod.rs`, `services_tests.rs`.

3.  **`transforms/` (Fat Node Transformers)**
    *   **Status**: Dead.
    *   **Reason**: These implement the `Transformer` trait for "fat" `Node` ASTs. Since the legacy parser has no emit/transform pipeline exposed, these run on nothing. The new `ThinPrinter` performs on-the-fly output generation using specific helpers (like `class_es5.rs`), bypassing this transformation pipeline entirely.
    *   **Files**:
        *   `transforms/async_gen.rs`
        *   `transforms/class.rs` (The `ClassTransformer` struct, distinct from `class_es5.rs` which is **live**)
        *   `transforms/es2015.rs`
        *   `transforms/generators.rs`
        *   `transforms/modules.rs`
        *   `transforms/async_emitter.rs`
        *   `transforms/generator_emitter.rs`

4.  **`declarations_emitter.rs`**
    *   **Status**: Potentially Dead / Unused.
    *   **Reason**: While it works on `ThinNodeArena` (the new architecture), it is not wired into `ThinParser` or any other public API in `lib.rs`. `ThinParser` only exposes `emit()` and `emitModern()`, which use `ThinPrinter`.

### Summary of "Old" vs "New" Pipeline

| Component | Legacy (Fat Node) | Modern (Thin Node) | Status |
| :--- | :--- | :--- | :--- |
| **Parser** | `parser_impl.rs` | `thin_parser.rs` | Legacy is **Live** (via `createParser`), Modern is **Live**. |
| **Binder** | `binder.rs` | `thin_binder.rs` | Legacy is **Live** (via `ParserState`), Modern is **Live**. |
| **Checker** | `checker/` | `thin_checker.rs` + `solver/` | Legacy is **Live** (via `ParserState`), Modern is **Live**. |
| **Emitter** | `emitter.rs` | `thin_emitter.rs` | **Legacy is DEAD**. |
| **LSP** | `services/` | *(None)* | **Legacy is DEAD**. |

### Recommendation

To clean up the codebase, you can safely delete:
1.  `emitter.rs` and its tests.
2.  `services/` directory.
3.  `transforms/` directory (except for `class_es5.rs` and `namespace_es5.rs`, which are used by `thin_emitter.rs`).

</details>

**Type Checking (25 failing tests)**
1. ✅ Export assignment validation (2309, 2304)
2. ✅ Setter parameter validation (1052, 1053)
3. ✅ Return type validation (2355) - function must return a value (basic types)
4. ✅ Abstract class instantiation (2511) - basic case (union types need more work)
5. ✅ Static member access from instance (2662) - `foo` → "Did you mean 'C.foo'?"
6. ✅ Abstract property in constructor (2715) - `this.abstractProp` in ctor
7. ⬜ Property used before initialization (2729) - needs dataflow analysis
8. ⬜ Accessor return type inference (7023) - implicit any in getter

**Parser Semantic Errors**

8. ⬜ Declaration expected (1128) - after certain tokens
9. ✅ Const modifier on class members (1248) - `const` invalid on properties
10. ✅ Accessor body in ambient context (1183) - no body in declare class
11. ⬜ Accessor in ambient context ES5 (18045) - accessors need ES5+

**Advanced Diagnostics**

12. ⬜ RelatedInformation - point to definition sites for context
13. ⬜ Accessor diagnostic hints (6234) - "did you mean to call it?"

### Blockers Analysis (26 failing tests)
| Category | Codes | Tests | Notes |
|----------|-------|-------|-------|
| Parser errors | 1005, 1128 | 4 | Error recovery gaps (1068, 1248 done) |
| Type errors | 2339, 2355, 2511 | 10 | Property access, returns, abstract (2662 done) |
| Accessor errors | 6234, 18045 | 3 | Hints, ES5 target (1183 done) |
| Abstract members | 2729, 2416, 2540 | 3 | Abstract property handling (2715 done) |

### Emit TODOs (96.7% failing)
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
- ⬜ TS2322/2345: Improve type assignability checks
- ⬜ TS2339: Property lookup on union/intersection types  
- ⬜ TS2304: Module resolution, global declarations
- ⬜ TS2300: Duplicate detection in binder
- ⬜ Modules: Start with `export {}` and named exports

---

# Phase 9: Finishing up all TODOs ⬜

- ⬜ 100% baseline in all aspects
- ⬜ all todos left from previous phases
- ⬜ todos in code
- ⬜ missing unit tests

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
