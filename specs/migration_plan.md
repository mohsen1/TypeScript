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
- ✅ Error 2322: accessor type compatibility (getter return ⊆ setter param)
- ✅ Error 2511: abstract union type detection (type_contains_abstract_class)
- ✅ TypeQuery symbol resolution for abstract class detection through type aliases
- ✅ Union type resolution for abstract class detection (get_type_from_union_type)
- ✅ Error 2729: property used before initialization (this.X in initializers)
- ✅ Array method support in PropertyAccessEvaluator (map, filter, etc. no longer report 2339)
- ✅ Error 2416: property not assignable to same property in base type
- ✅ Error 2654: non-abstract class missing implementations for abstract members
- ✅ Error 2540: cannot assign to readonly property (via AST-level modifier check)

## Checker Refactoring (see specs/REFACTOR_CHECKER.md)
- ✅ Split ThinCheckerState into Context + specialized Checkers (expr, stmt, decl)
  - ✅ Created `checker/context.rs` with CheckerContext struct
  - ✅ Refactored ThinCheckerState to wrap CheckerContext
  - ✅ Created `checker/expr.rs` with ExpressionChecker struct
  - ✅ Created `checker/statements.rs` with StatementChecker struct
  - ✅ Created `checker/declarations.rs` with DeclarationChecker struct
- 🔄 Move expression type computation to solver/operations.rs (incremental)
- 🔄 Use NodeView API instead of raw arena lookups (incremental)
- ⬜ Deprecate checker/types in favor of solver/types

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

## Emitter Architecture Refactor (complete)
- ✅ Created `SourceWriter` abstraction for output generation with source map tracking
- ✅ Created `EmitContext` for transform-specific state management
- ✅ Refactored `ThinPrinter` to use `SourceWriter` (decouples output from AST traversal)
- ✅ UTF-16 column counting for correct source map positions
- ✅ Refactored `ThinPrinter` to use `EmitContext` for all transform state
- ✅ Separated arrow function ES5/native emit paths
- ✅ Converted `thin_emitter.rs` to directory module (`thin_emitter/mod.rs`)
- ✅ Marked fields/methods `pub(super)` for future submodule splitting
- ⏳ Split `emit_node` into modules - structure ready, splitting deferred until needed

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
| .errors.txt | **80.5%** (62/77 subset) | 33.8% (1,741/5,157) | 0.05% |
| .js emit | **60.5%** (46/76 subset) | ~3% | 0.05% |

### Work Process

⚠️ **CRITICAL: For each task, ALWAYS:**
1. **BEFORE**: `node scripts/ask-gemini.mjs "How should I implement [task]?"` - get guidance
2. **IMPLEMENT**: Write code, run tests. add tests
3. **AFTER**: `node scripts/ask-gemini.mjs --review wasm/src/[file].rs` - get review

This catches design issues early and ensures consistent code quality.


### Checker Architecture Cleanup (see specs/REFACTOR_CHECKER.md)

1. ✅ Refactor ThinCheckerState to use CheckerContext (wraps `ctx: CheckerContext<'a>`)
2. ✅ Create ExpressionChecker in checker/expr.rs
3. ✅ Create specialized checker modules (statements.rs, declarations.rs)
4. 🔄 Move expression type computation to solver (incremental, as features are added)
5. 🔄 Use NodeView API consistently (incremental, as code is touched)

### Type Checker Errors - Recently Completed
- ✅ Error 2416: Property not assignable to same property in base type
- ✅ Error 2654: Non-abstract class missing implementations for abstract members
- ✅ Error 2540: Cannot assign to readonly property
- ✅ Error 2676: Accessor abstract consistency (fixed: skip type check for abstract accessors)
- ✅ Error 1253: Abstract in non-abstract class
- ✅ Error 2355: Function type annotations

THEN continue working on baseline type checker work
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
