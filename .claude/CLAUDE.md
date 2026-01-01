# TypeScript to Rust/WASM Migration Project

## Project Overview

Incrementally migrating TypeScript compiler to Rust/WASM using "Strangler Fig" pattern.
Rust components progressively replace TypeScript while compiler stays functional.

## Current State

**Phase**: 3 - Parser (~98% complete)
**Tests**: 81 Rust tests + 19 TypeScript integration tests
**Features**: Full parser with --useRustParser flag working end-to-end

### What's Working
- Scanner: Complete, verified token-for-token match with TS scanner
- Parser: All major constructs (statements, expressions, declarations, types)
- Integration: parseWithRustParser() in parser.ts, 45+ node type conversions
- JSX & Decorators: Full support
- Type System: All advanced types (conditional, mapped, indexed access, infer, keyof, typeof)

## Key Files

| File | Purpose |
|------|---------|
| `MIGRATION_PLAN.md` | Detailed progress tracking |
| `wasm/src/lib.rs` | WASM entry points |
| `wasm/src/scanner_impl.rs` | Scanner (~800 lines) |
| `wasm/src/parser.rs` | AST nodes (~2200 lines, 130+ types) |
| `wasm/src/parser_impl.rs` | Parser (~3900 lines) |
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

- **Arena allocation**: `NodeArena` + `NodeIndex` for AST nodes
- **Serialization**: JSON via serde for Rust↔JS
- **Feature flags**: `--useRustScanner`, `--useRustParser`
- **Fallback**: Graceful fallback to TS parser on unsupported features

## Next Steps (Phase 4)

- [ ] Symbol table implementation
- [ ] Scope management
- [ ] Declaration merging
