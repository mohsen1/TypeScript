Migration Plan: TypeScript Compiler ➜ Rust via WebAssembly
==========================================================

Vision
------
Incrementally rewrite the entire TypeScript compiler and type checker in Rust,
compiled to WebAssembly for seamless Node.js/browser interop. The migration
follows the "Strangler Fig" pattern: Rust components progressively replace
TypeScript modules while the compiler remains fully functional at every step.

Guiding Principles
------------------
1. **Never Break the Build** – Every commit must pass `hereby runtests-parallel`.
2. **Iterate in Small Slices** – One function, one module at a time.
3. **Test Before & After** – Existing test suite is the source of truth; add Rust unit tests.
4. **Performance Parity First** – Match TS speed before optimizing.
5. **Feature Flags** – New Rust paths can be toggled off if regressions appear.
6. **Document Everything** – Each migrated module gets a section in this plan.

==============================================================================
PHASE 0: INFRASTRUCTURE (COMPLETE)
==============================================================================

0.1 Toolchain Setup
-------------------
- [x] Install Rust stable via rustup (1.92.0)
- [x] Install wasm-pack (0.13.1)
- [x] Verify `cargo build --target wasm32-unknown-unknown` works

0.2 Crate Scaffold
------------------
- [x] Create `wasm/` crate at repo root
- [x] Configure `Cargo.toml` with `crate-type = ["cdylib", "rlib"]`
- [x] Add `wasm-bindgen = "0.2"` dependency
- [x] Export trivial `add(a, b)` function

0.3 Build Integration
---------------------
- [x] Add `build-wasm` task to `Herebyfile.mjs`
- [x] Wire into `generateLibs` dependency chain
- [x] Output to `built/local/wasm/`
- [x] Implement `needsUpdate` for incremental builds

0.4 TypeScript Bridge
---------------------
- [x] Create `src/compiler/wasm.ts` abstraction layer
- [x] Implement lazy dynamic `require()` to avoid bundler issues
- [x] Call `wasmAdd(2, 2)` from `src/tsc/tsc.ts`
- [x] Verify: `node built/local/tsc.js --version` prints `[WASM] 2 + 2 = 4`

==============================================================================
PHASE 1: UTILITIES & DATA STRUCTURES
==============================================================================
Target: Migrate pure, stateless utility functions that have no dependencies on
the rest of the compiler. These are safe to port first because they can be
tested in isolation.

1.1 String Utilities
--------------------
- [ ] `src/compiler/core.ts` → `getStringComparer`, `compareCaseInsensitive`
- [ ] `src/compiler/path.ts` → `combinePaths`, `getDirectoryPath`, `normalizePath`
- [ ] Test: Run path resolution tests against both TS and Rust implementations
- [ ] Benchmark: Ensure Rust version is not slower

1.2 Collections
---------------
- [ ] Implement Rust equivalents for `Map`, `Set`, `MultiMap` patterns
- [ ] Port `createMap`, `forEach`, `some`, `every`, `find` utilities
- [ ] Use `wasm-bindgen` to expose iterators to JS

1.3 Text Manipulation
---------------------
- [ ] `src/compiler/scanner.ts` → character classification (`isWhiteSpace`, `isDigit`, etc.)
- [ ] Unicode handling (may leverage Rust's superior unicode support)
- [ ] String interning / symbol table

Verification Gate: All existing compiler tests pass with Rust utilities enabled.

==============================================================================
PHASE 2: SCANNER / LEXER
==============================================================================
Target: The scanner is the first major compiler component—converts source text
into tokens. It's largely self-contained and performance-critical.

2.1 Token Definitions
---------------------
- [ ] Define `SyntaxKind` enum in Rust (mirror `src/compiler/types.ts`)
- [ ] Implement `Token` struct with span information
- [ ] Export token constants via wasm-bindgen

2.2 Core Scanner
----------------
- [ ] Port `createScanner()` logic to Rust
- [ ] Implement `scan()`, `getToken()`, `getTokenPos()`, `getTokenText()`
- [ ] Handle all JavaScript/TypeScript token types
- [ ] Support JSX scanning mode

2.3 Integration
---------------
- [ ] Create `RustScanner` wrapper in `src/compiler/scanner.ts`
- [ ] Add feature flag: `--useRustScanner`
- [ ] Run scanner tests with both implementations
- [ ] Benchmark: Target 2x speedup for large files

Verification Gate: `tests/cases/compiler/*.ts` produce identical token streams.

==============================================================================
PHASE 3: PARSER
==============================================================================
Target: Convert source tokens into AST. This is tightly coupled with the
scanner and shares data structures with the type checker.

3.1 AST Node Definitions
------------------------
- [ ] Define all `Node` types in Rust (100+ node kinds)
- [ ] Implement `NodeFlags`, `ModifierFlags`, `TransformFlags`
- [ ] Design memory layout for efficient wasm<->JS transfer
- [ ] Consider using `serde` for AST serialization

3.2 Parser Core
---------------
- [ ] Port `parseSourceFile()` entry point
- [ ] Implement statement parsing (`parseStatement`, `parseDeclaration`)
- [ ] Implement expression parsing (`parseExpression`, `parseBinaryExpression`)
- [ ] Handle automatic semicolon insertion (ASI)

3.3 JSX & Decorators
--------------------
- [ ] Port JSX parsing (`parseJsxElement`, `parseJsxExpression`)
- [ ] Port decorator parsing
- [ ] Experimental syntax support

3.4 Integration
---------------
- [ ] Create `RustParser` wrapper calling into wasm
- [ ] Feature flag: `--useRustParser`
- [ ] Roundtrip test: parse → emit → parse must be identical

Verification Gate: All parser baselines match.

==============================================================================
PHASE 4: BINDER
==============================================================================
Target: Walk the AST and create symbol table, establishing scope and name
resolution.

4.1 Symbol Table
----------------
- [ ] Implement `Symbol` struct in Rust
- [ ] Port `SymbolFlags` and symbol creation logic
- [ ] Handle declaration merging

4.2 Scope Management
--------------------
- [ ] Implement scope chain (block, function, module, global)
- [ ] Port `bindSourceFile()` traversal
- [ ] Handle hoisting rules

4.3 Flow Analysis Setup
-----------------------
- [ ] Create control flow graph nodes
- [ ] Port flow container logic
- [ ] Prepare for type narrowing in checker

Verification Gate: Symbol resolution tests pass.

==============================================================================
PHASE 5: TYPE CHECKER (THE BIG ONE)
==============================================================================
Target: The heart of TypeScript—structural type checking, inference, and
diagnostics. This is ~50% of the compiler complexity.

Strategy: Migrate in layers, starting with primitive type operations and
building up to full inference.

5.1 Type Representation
-----------------------
- [ ] Define `Type` enum in Rust (ObjectType, UnionType, etc.)
- [ ] Implement `TypeFlags` and type predicates
- [ ] Port intrinsic types (string, number, boolean, etc.)
- [ ] Handle type aliases and references

5.2 Subtype & Assignability
---------------------------
- [ ] Port `isTypeRelatedTo()` core logic
- [ ] Implement structural compatibility checks
- [ ] Handle variance (covariance, contravariance)
- [ ] Port excess property checks

5.3 Type Inference
------------------
- [ ] Implement inference context
- [ ] Port `inferTypes()` and constraint solving
- [ ] Handle generic instantiation
- [ ] Contextual typing

5.4 Control Flow Analysis
-------------------------
- [ ] Port type narrowing logic
- [ ] Implement assertion functions
- [ ] Handle discriminated unions
- [ ] Exhaustiveness checking

5.5 Diagnostics
---------------
- [ ] Port error message generation
- [ ] Implement related information spans
- [ ] Match exact error codes and messages

Verification Gate: `tests/baselines/reference/*.types` match exactly.

==============================================================================
PHASE 6: EMITTER
==============================================================================
Target: Generate JavaScript/declaration files from the AST.

6.1 Printer
-----------
- [ ] Port AST → text printing logic
- [ ] Handle formatting and whitespace
- [ ] Source map generation

6.2 Transformers
----------------
- [ ] Port downlevel transforms (ES2015 → ES5, etc.)
- [ ] Module system transforms (ESM ↔ CJS)
- [ ] JSX transform

6.3 Declaration Emit
--------------------
- [ ] Port `.d.ts` generation
- [ ] Handle visibility and export pruning

Verification Gate: All emit baselines match.

==============================================================================
PHASE 7: LANGUAGE SERVICE
==============================================================================
Target: IDE features—completions, hover, go-to-definition, etc.

7.1 Completions
---------------
- [ ] Port completion entry generation
- [ ] Symbol filtering and ranking

7.2 Quick Info / Hover
----------------------
- [ ] Port display parts generation
- [ ] Type-to-string rendering

7.3 Navigation
--------------
- [ ] Go to definition
- [ ] Find all references
- [ ] Rename support

Verification Gate: fourslash tests pass.

==============================================================================
PHASE 8: FULL RUST MODE
==============================================================================
Target: TypeScript compiler is 100% Rust. The TypeScript source in `src/` is
only used for tests and legacy compatibility.

8.1 Standalone Binary
---------------------
- [ ] Create native `tsc` binary (no Node.js required)
- [ ] CLI argument parsing in Rust
- [ ] File system abstraction

8.2 Performance Optimization
----------------------------
- [ ] Profile and optimize hot paths
- [ ] Implement parallel type checking
- [ ] Memory usage optimization

8.3 Compatibility Mode
----------------------
- [ ] Maintain wasm build for Node.js users
- [ ] Ensure identical behavior between native and wasm builds

==============================================================================
TESTING STRATEGY
==============================================================================

Continuous Verification
-----------------------
At every step, the following must pass:

1. `npx hereby runtests-parallel` – Full test suite
2. `npx hereby baseline-accept` – Only if intentional changes
3. Manual smoke test: compile a real-world project (e.g., vscode)

Feature Flags
-------------
Each Rust component has a runtime toggle:
- `--useRustScanner`
- `--useRustParser`
- `--useRustChecker`

This allows A/B testing and safe rollback.

Benchmark Suite
---------------
Track performance at each phase:
- Compile time for `src/compiler/**/*.ts` (self-compile)
- Memory usage
- Startup latency

Differential Testing
--------------------
For migrated components, run both TS and Rust versions and assert identical output.

==============================================================================
GIT WORKFLOW
==============================================================================

Commit Frequently
-----------------
- **Commit after every passing test run** – Small, atomic commits are easier to
  bisect and revert.
- **Never commit broken code** – If tests fail, fix before committing.
- **One logical change per commit** – Don't mix refactoring with new features.

Commit Cadence
--------------
Aim for commits at these checkpoints:

1. After adding a new Rust function (even if not yet wired to TS)
2. After wiring Rust function to TS bridge
3. After tests pass with new Rust code enabled
4. After fixing any regressions
5. After updating documentation/plan

Commit Message Format
---------------------
```
[wasm] <component>: <short description>

- Detail 1
- Detail 2

Tests: npx hereby runtests-parallel ✓
```

Examples:
```
[wasm] scanner: port isWhiteSpaceLike to Rust

- Added character classification in wasm/src/scanner.rs
- Exposed via wasm-bindgen
- TS bridge calls Rust version when --useRustScanner

Tests: npx hereby runtests-parallel ✓
```

```
[wasm] infra: add needsUpdate check for wasm build

- Herebyfile now skips wasm-pack if sources unchanged
- Speeds up incremental builds

Tests: npx hereby local ✓
```

Branch Strategy
---------------
- `main` – Always stable, tests passing
- `wasm/<phase>-<component>` – Feature branches for each migration slice
- Merge to main only after full test suite passes
- Squash small fixup commits before merging

Pre-Commit Checklist
--------------------
Before every commit:
```bash
npx hereby local                    # Build passes
npx hereby runtests-parallel        # Tests pass
node built/local/tsc.js --version   # Smoke test
git add -A && git commit -m "..."   # Commit
```

==============================================================================
PROGRESS LOG
==============================================================================

[2026-01-01] Phase 0 Complete
-----------------------------
- Toolchain: rustc 1.92.0, wasm-pack 0.13.1
- Crate: `wasm/` with `wasm-bindgen`
- Build: `hereby local` produces `built/local/wasm/`
- Bridge: `src/compiler/wasm.ts` + `src/tsc/tsc.ts` integration
- Verification: `node built/local/tsc.js --version` outputs `[WASM] 2 + 2 = 4`

[2026-01-01] Phase 1.1 Complete - String Utilities
--------------------------------------------------
- Ported `compareStringsCaseSensitive` to Rust
- Ported `compareStringsCaseInsensitive` to Rust
- Ported `compareStringsCaseInsensitiveEslintCompatible` to Rust
- Ported `equateStringsCaseSensitive` and `equateStringsCaseInsensitive`
- Added Rust `Comparison` enum matching TypeScript
- 4 Rust unit tests passing
- Commit: `97292d8aa`

[2026-01-01] Phase 1.2 Complete - Path Utilities
------------------------------------------------
- Ported `isAnyDirectorySeparator` to Rust
- Ported `normalizeSlashes` to Rust
- Ported `hasTrailingDirectorySeparator` to Rust
- Ported `pathIsRelative` to Rust
- Ported `removeTrailingDirectorySeparator` to Rust
- Ported `ensureTrailingDirectorySeparator` to Rust
- Ported `hasExtension` to Rust
- Ported `getBaseFileName` to Rust
- Ported `fileExtensionIs` to Rust
- 13 Rust unit tests passing (4 string + 9 path)
- Commit: `979dde4c9`

Next Step: Continue Phase 1 (more utilities) or begin Phase 2 (Scanner)

==============================================================================
ESTIMATED TIMELINE (AGGRESSIVE)
==============================================================================

| Phase | Component           | Est. Effort | Risk    |
|-------|---------------------|-------------|---------|
| 0     | Infrastructure      | 1 week      | Low     | ✅ DONE
| 1     | Utilities           | 2 weeks     | Low     |
| 2     | Scanner             | 3 weeks     | Medium  |
| 3     | Parser              | 6 weeks     | Medium  |
| 4     | Binder              | 4 weeks     | Medium  |
| 5     | Type Checker        | 16 weeks    | High    |
| 6     | Emitter             | 6 weeks     | Medium  |
| 7     | Language Service    | 8 weeks     | Medium  |
| 8     | Full Rust Mode      | 4 weeks     | Low     |

Total: ~50 weeks (1 year) for complete migration

Note: Type checker (Phase 5) is the critical path. Consider parallel workstreams
for Scanner/Parser while spec'ing out checker architecture.

==============================================================================
ARCHITECTURAL DECISIONS
==============================================================================

1. **Memory Model**: Use **serialization** for Rust↔JS data transfer.
   During migration, we do NOT optimize for hybrid mode performance—the goal
   is to reach full Rust as quickly as possible. Serialization keeps the
   boundary clean and debuggable.

2. **Incremental Compilation**: **Required from day one**. The Rust checker
   must support:
   - Watch mode (`tsc --watch`)
   - Project references (`--build`)
   - `.tsbuildinfo` caching
   Design the Rust architecture with incrementality in mind—don't bolt it on later.

3. **Plugin API**: **Deferred**. No plugin support during migration.
   Transformers can be planned post-migration once the Rust AST is stable.
   Third-party plugins will need to wait for a new Rust-native API.

4. **wasm64**: **Monitor actively**. Large monorepos may hit wasm32's 4GB limit.
   Track the [Memory64 proposal](https://github.com/WebAssembly/memory64) and
   be ready to adopt when stable. Native binary mode (Phase 8) sidesteps this.

5. **Error Messages**: **Strict backward compatibility**. Error codes, messages,
   and spans must match TypeScript exactly. Tools like ESLint, editors, and CI
   pipelines depend on deterministic output. Add diff tests for diagnostics.
