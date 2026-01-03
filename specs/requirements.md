# TypeScript to Rust Migration Requirements

## Vision

Incrementally rewrite the entire TypeScript compiler and type checker in Rust,
compiled to WebAssembly for seamless Node.js/browser interop. The migration
follows the "Strangler Fig" pattern: Rust components progressively replace
TypeScript modules while the compiler remains fully functional at every step.

## Guiding Principles

1. **Never Break the Build** – Every commit must pass `hereby runtests-parallel`.
2. **Iterate in Small Slices** – One function, one module at a time.
3. **Test Before & After** – Existing test suite is the source of truth; add Rust unit tests.
4. **Performance Parity First** – Match TS speed before optimizing.
5. **Feature Flags** – New Rust paths can be toggled off if regressions appear.
6. **Document Everything** – Each migrated module gets a section in the migration plan.
7. **Use Docker** – All builds and tests should run in Docker for reproducibility.

## Architecture

- **Language:** Rust (compiled to WASM via `wasm-pack`)
- **WASM Crate:** `wasm/` directory at repo root
- **Integration:** `src/compiler/wasm.ts` acts as the bridge between TS and WASM
- **Build System:** `hereby` tasks for building WASM and running tests
- **Container:** `Dockerfile` for reproducible builds

## Current Status

| Phase | Component        | Status  | Notes |
|-------|------------------|---------|-------|
| 0     | Infrastructure   | ✅ DONE | Toolchain, crate, build integration |
| 1     | Utilities        | ✅ DONE | String comparison, path utils, char classification |
| 2     | Scanner          | ✅ 95%  | Core complete, JSX/JSDoc deferred |
| 3     | Parser           | ✅ 98%  | Full parsing, JSX, decorators |
| 4     | Binder           | ✅ DONE | Symbols, scopes, flow analysis setup |
| 5     | Type Checker     | 🟡 60%  | Core inference done, unions blocked |
| 6     | Emitter          | ⬜ 0%   | Not started |
| 7     | Language Service | ⬜ 0%   | Not started |
| 8     | Full Rust Mode   | ⬜ 0%   | Not started |

## Feature Flags

| Flag | Description |
|------|-------------|
| `--useRustScanner` | Use Rust scanner instead of TypeScript |
| `--useRustParser` | Use Rust parser instead of TypeScript (implies scanner) |
| `--useRustChecker` | Use Rust type checker (future, implies parser) |

### Running Tests with Rust Flags

```bash
# Run all tests with Rust scanner
npx hereby runtests-parallel -- --useRustScanner

# Run all tests with Rust parser
npx hereby runtests-parallel -- --useRustParser

# Run all tests with Rust checker (when ready)
npx hereby runtests-parallel -- --useRustChecker

# Run specific test suites
npx hereby runtests --runner=fourslash -- --useRustScanner
npx hereby runtests --runner=compiler -- --useRustParser

# Run specific test file
npx hereby runtests --tests=tests/cases/compiler/someTest.ts -- --useRustChecker
```

### Implementing a New Feature Flag

1. **CommandLineOptionDeclarations** (`src/compiler/commandLineParser.ts`)
2. **CompilerOptions interface** (`src/compiler/types.ts`)
3. **Diagnostic message** (`src/compiler/diagnosticMessages.json`)
4. **WASM bridge** (`src/compiler/wasm.ts`)
5. **Integration point** in the relevant compiler phase

## Architectural Decisions

1. **Memory Model**: Use serialization for Rust↔JS data transfer during migration.
2. **Incremental Compilation**: Required from day one (watch mode, project references).
3. **Plugin API**: Deferred until post-migration.
4. **wasm64**: Monitor actively for large monorepos (4GB limit).
5. **Error Messages**: Strict backward compatibility with exact error codes.

## Reference

- **Full Migration Plan:** `specs/migration_plan.md`
- **Task List:** `@fix_plan.md`
- **Build Instructions:** `@AGENT.md`
