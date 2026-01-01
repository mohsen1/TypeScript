# TypeScript to Rust/WASM Migration Project

## Project Overview

Incrementally migrating TypeScript compiler to Rust/WASM using "Strangler Fig" pattern.
Rust components progressively replace TypeScript while compiler stays functional.

## Current State

**Phase**: 5 - Type Checker (in progress)
**Tests**: 125 Rust tests + 19 parser TS tests + 10 binder tests
**Features**: Full parser + binder + type infrastructure + function type inference

### What's Working
- Scanner: Complete, verified token-for-token match with TS scanner
- Parser: All major constructs (statements, expressions, declarations, types, arrow functions)
- Binder: Symbol creation, scope management, declaration merging
- Checker: Type infrastructure, intrinsic types, type assignability, function types
- Integration: parseWithRustParser() + bindSourceFile() working end-to-end
- JSX & Decorators: Full support
- Type System: All advanced types (conditional, mapped, indexed access, infer, keyof, typeof)
- Function Types: FunctionType struct, parameter inference, type aliases, arrow functions

## Key Files

| File | Purpose |
|------|---------|
| `MIGRATION_PLAN.md` | Detailed progress tracking |
| `wasm/src/lib.rs` | WASM entry points |
| `wasm/src/scanner_impl.rs` | Scanner (~800 lines) |
| `wasm/src/parser.rs` | AST nodes (~2200 lines, 130+ types) |
| `wasm/src/parser_impl.rs` | Parser (~4000 lines) |
| `wasm/src/binder.rs` | Binder (~1100 lines) |
| `wasm/src/checker.rs` | Type Checker (~1700 lines, 24 tests) |
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

# Verify binder
node scripts/verifyBinder.mjs
```

## Architecture

- **Arena allocation**: `NodeArena` + `NodeIndex` for AST nodes, `SymbolArena` for symbols, `TypeArena` for types
- **Serialization**: JSON via serde for Rust↔JS
- **Feature flags**: `--useRustScanner`, `--useRustParser`
- **Fallback**: Graceful fallback to TS parser on unsupported features

## Phase 5 Progress

- [x] Type struct and TypeFlags
- [x] TypeArena with singleton intrinsic types
- [x] CheckerState with type caching
- [x] Type assignability (is_type_assignable_to)
- [x] Symbol type resolution via binder
- [x] Function type inference (FunctionType, parameters, return types)
- [x] TypeReference resolution for keyword types
- [x] Arrow function expression parsing
- [ ] Generic types and instantiation
- [ ] Object type checking
