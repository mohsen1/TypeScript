# TypeScript to Rust/WASM Migration Project

## Project Overview

Incrementally migrating TypeScript compiler to Rust/WASM using "Strangler Fig" pattern.
Rust components progressively replace TypeScript while compiler stays functional.

## Current State

**Phase**: 4 - Binder (in progress)
**Tests**: 94 Rust tests + 19 TypeScript integration tests
**Features**: Full parser + basic binder with symbol tables

### What's Working
- Scanner: Complete, verified token-for-token match with TS scanner
- Parser: All major constructs (statements, expressions, declarations, types)
- Integration: parseWithRustParser() in parser.ts, 45+ node type conversions
- JSX & Decorators: Full support
- Type System: All advanced types (conditional, mapped, indexed access, infer, keyof, typeof)
- Binder: Symbol creation, scope management, declaration merging

## Key Files

| File | Purpose |
|------|---------|
| `MIGRATION_PLAN.md` | Detailed progress tracking |
| `wasm/src/lib.rs` | WASM entry points |
| `wasm/src/scanner_impl.rs` | Scanner (~800 lines) |
| `wasm/src/parser.rs` | AST nodes (~2200 lines, 130+ types) |
| `wasm/src/parser_impl.rs` | Parser (~4000 lines) |
| `wasm/src/binder.rs` | Binder (~1000 lines) |
| `src/compiler/parser.ts` | TS integration (parseWithRustParser, convertNode) |
| `src/compiler/wasm.ts` | WASM bridge |

## Commands

```bash
# Run Rust tests
source ~/.cargo/env && cd wasm && cargo test

# Build everything
source ~/.cargo/env && npx hereby local

# Test with Rust parser
node built/local/tsc.js file.ts --useRustParser --noEmit

# Verify scanner
node scripts/verifyScanner.mjs src/compiler/checker.ts

# Verify parser
node scripts/verifyParser.mjs
```

## Architecture

- **Arena allocation**: `NodeArena` + `NodeIndex` for AST nodes, `SymbolArena` for symbols
- **Serialization**: JSON via serde for Rust↔JS
- **Feature flags**: `--useRustScanner`, `--useRustParser`
- **Fallback**: Graceful fallback to TS parser on unsupported features

## Next Steps

- [ ] Flow analysis setup (control flow graph, narrowing)
- [ ] TypeScript integration for binder
- [ ] Type checker foundation
