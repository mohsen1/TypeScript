# TypeScript to Rust/WASM Migration Project

## Project Overview

Incrementally migrating TypeScript compiler to Rust/WASM using "Strangler Fig" pattern.
Rust components progressively replace TypeScript while compiler stays functional.

**Key Insight**: Parser and Checker development happen together - new syntax requires both parsing AND type checking support.

## Current State

**Phase**: Integrated Parser + Checker Development
**Tests**: 157 Rust tests + 19 parser TS tests + 10 binder tests
**Architecture**: Scanner → Parser → Binder → Checker (all in Rust)

### Component Status

| Component | Status | Lines | Tests |
|-----------|--------|-------|-------|
| Scanner | Complete ✅ | ~800 | Token-verified |
| Parser | ~98% ✅ | ~4700 | 45 tests |
| Binder | Complete ✅ | ~1400 | 13 tests |
| Checker | In Progress 🔄 | ~3800 | 35 tests |

### What's Working
- **Scanner**: Token-for-token match with TS scanner
- **Parser**: Statements, expressions, declarations, types, JSX, decorators, arrow functions, assignment expressions, unary operators
- **Binder**: Symbol creation, scopes, declaration merging, flow analysis (if/while)
- **Checker**: Type inference, assignability, function types, generics, object types, type narrowing

## Key Files

| File | Purpose |
|------|---------|
| `wasm/src/scanner_impl.rs` | Token scanning |
| `wasm/src/parser.rs` | AST node definitions |
| `wasm/src/parser_impl.rs` | Parsing logic |
| `wasm/src/binder.rs` | Symbol binding |
| `wasm/src/checker.rs` | Type checking |
| `src/compiler/wasm.ts` | WASM bridge |

## Documentation

| Document | Purpose |
|----------|---------|
| `docs/TYPE_CHECKER_MINDMAP.md` | Visual type system architecture and flow |
| `docs/TYPE_CHECKER_DESIGN.md` | High-level checker design decisions |
| `docs/TYPE_CHECKER_IMPLEMENTATION.md` | Implementation details and patterns |
| `docs/TYPESCRIPT_LANGUAGE_SPECIFICATION.md` | TypeScript language spec reference |
| `docs/TYPESCRIPT_ADVANCED_TYPES.md` | Advanced type system features |

## Commands

### ⚠️ CRITICAL: ALWAYS USE DOCKER FOR RUST TESTS/BENCHMARKS ⚠️

**NEVER run `cargo test` or `cargo bench` directly on the host machine!**
This WILL consume excessive RAM and crash the system.

**ALWAYS use Docker with memory limits:**

```bash
# Run all Rust tests (do this frequently!)
cd wasm && docker build -t rust-wasm-tests . && docker run --rm --memory="1g" --cpus="2.0" rust-wasm-tests

# Run specific test (MUST use Docker)
cd wasm && docker build -t rust-wasm-tests . && docker run --rm --memory="1g" --cpus="2.0" rust-wasm-tests cargo test test_name_here

# Run Rust benchmarks (MUST use Docker)
cd wasm && docker build -t rust-wasm-tests . && docker run --rm --memory="2g" --cpus="4.0" rust-wasm-tests cargo bench

# Build everything including WASM
source ~/.cargo/env && npx hereby local

# Test with Rust parser
node built/local/tsc.js file.ts --useRustParser --noEmit

# Verify components
node scripts/verifyScanner.mjs src/compiler/checker.ts
node scripts/verifyParser.mjs
node scripts/verifyBinder.mjs
```

## Development Workflow

### When Adding New Syntax
1. **Parser**: Add AST node to `parser.rs`, parsing to `parser_impl.rs`
2. **Binder**: Handle node in `bind_node()` if it declares symbols
3. **Checker**: Add to `get_type_of_node_worker()` for type inference
4. **Tests**: Add parser test + checker test

### When Adding Type Features
1. **Checker**: Add type variant to `Type` enum, update `TypeArena`
2. **Parser**: Ensure corresponding type syntax is parsed
3. **Checker**: Implement type relation rules in `is_type_assignable_to()`

## Architecture

```
Source Code
    ↓
┌─────────────────────┐
│  Scanner (Rust)     │  Tokenizes source text
└─────────────────────┘
    ↓ tokens
┌─────────────────────┐
│  Parser (Rust)      │  Builds AST with NodeArena
└─────────────────────┘
    ↓ AST
┌─────────────────────┐
│  Binder (Rust)      │  Creates symbols in SymbolArena
└─────────────────────┘
    ↓ symbols
┌─────────────────────┐
│  Checker (Rust)     │  Type inference with TypeArena
└─────────────────────┘
    ↓ types + diagnostics
```

### Arena Pattern
- `NodeArena` + `NodeIndex` - AST nodes
- `SymbolArena` + `SymbolId` - Symbols
- `TypeArena` + `TypeId` - Types
- `FlowNodeArena` + `FlowNodeId` - Control flow nodes

## Completed Features

### Parser
- [x] Assignment expressions (=, +=, -=, *=, etc.)
- [x] Unary expressions (typeof, void, delete, await, ++, --, !, ~)
- [x] Arrow functions with type parameters
- [x] Generic function/type declarations

### Checker
- [x] Generic types (type parameters, constraints, defaults)
- [x] Type instantiation (substituting type arguments)
- [x] Object type checking (property access, type literals)
- [x] Type narrowing (typeof guards, non-nullable)
- [x] Function types with type parameters

### Binder
- [x] Flow analysis for if/while statements
- [x] Flow node creation and antecedent tracking

## Current Tasks

### In Progress
- [ ] Control flow based type narrowing (use flow nodes in checker)
- [ ] Call expression type checking with generics

### Next Up
- [ ] instanceof type guards
- [ ] Class type checking
- [ ] Index signatures

## Reference

- `docs/TYPE_CHECKER_MINDMAP.md` - Visual type system architecture
- `docs/TYPESCRIPT_LANGUAGE_SPECIFICATION.md` - Language spec for behavior reference
- `docs/TYPESCRIPT_ADVANCED_TYPES.md` - Advanced type features guide
- `~/code/typescript-go` - Microsoft's Go port for patterns
- TypeScript `src/compiler/checker.ts` - Original implementation
