# TypeScript to Rust/WASM Migration Project

## Project Overview

Incrementally migrating TypeScript compiler to Rust/WASM using "Strangler Fig" pattern.
Rust components progressively replace TypeScript while compiler stays functional.

**Key Insight**: Parser and Checker development happen together - new syntax requires both parsing AND type checking support.

## Current State

**Phase**: Integrated Parser + Checker Development
**Tests**: 125 Rust tests + 19 parser TS tests + 10 binder tests
**Architecture**: Scanner → Parser → Binder → Checker (all in Rust)

### Component Status

| Component | Status | Lines | Tests |
|-----------|--------|-------|-------|
| Scanner | Complete ✅ | ~800 | Token-verified |
| Parser | ~95% ✅ | ~4200 | 38 tests |
| Binder | Complete ✅ | ~1100 | 10 tests |
| Checker | In Progress 🔄 | ~1700 | 27 tests |

### What's Working
- **Scanner**: Token-for-token match with TS scanner
- **Parser**: Statements, expressions, declarations, types, JSX, decorators, arrow functions
- **Binder**: Symbol creation, scopes, declaration merging, flow analysis setup
- **Checker**: Type inference, assignability, function types, symbol resolution

## Key Files

| File | Purpose |
|------|---------|
| `wasm/src/scanner_impl.rs` | Token scanning |
| `wasm/src/parser.rs` | AST node definitions |
| `wasm/src/parser_impl.rs` | Parsing logic |
| `wasm/src/binder.rs` | Symbol binding |
| `wasm/src/checker.rs` | Type checking |
| `src/compiler/wasm.ts` | WASM bridge |
| `docs/TYPE_CHECKER_MINDMAP.md` | Type system architecture |

## Commands

```bash
# Run all Rust tests (do this frequently!)
source ~/.cargo/env && cd wasm && cargo test

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

## Current Tasks

### In Progress
- [ ] Generic types (type parameters, instantiation)
- [ ] Object type checking (property access, methods)

### Next Up
- [ ] Type narrowing (typeof, instanceof guards)
- [ ] Call expression type checking
- [ ] Class type checking

### Parser Gaps to Fill as Needed
- Assignment expressions (+=, etc.)
- Prefix/postfix operators (++, --)
- Spread in more contexts

## Reference

- `docs/TYPE_CHECKER_MINDMAP.md` - Visual type system architecture
- `~/code/typescript-go` - Microsoft's Go port for patterns
- TypeScript `src/compiler/checker.ts` - Original implementation
