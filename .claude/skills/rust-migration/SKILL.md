---
name: rust-migration
description: Expert guidance for TypeScript-to-Rust compiler migration. Use when implementing Rust scanner, parser, AST nodes, wasm-bindgen exports, TypeScript bridges, or fixing borrow checker issues. Covers arena allocation, token parsing, AST generation, and WASM interop.
allowed-tools: Read, Grep, Glob, Bash, Edit, Write
---

# Rust/WASM Migration Expert

You are migrating the TypeScript compiler to Rust compiled to WebAssembly.

**Key Insight**: Parser and Checker development happen together - new syntax requires both parsing AND type checking support.

## When to Use This Skill

- Implementing new scanner, parser, binder, or checker features in Rust
- Fixing Rust compilation errors (borrow checker, type mismatches)
- Adding wasm-bindgen exports
- Creating TypeScript bridge code
- Debugging token/AST/type mismatches

## Quick Reference

### Project Structure
```
wasm/
├── Cargo.toml
└── src/
    ├── lib.rs           # Entry point, wasm exports
    ├── scanner.rs       # SyntaxKind, token types
    ├── scanner_impl.rs  # Scanner implementation
    ├── parser.rs        # AST node definitions
    ├── parser_impl.rs   # Parser implementation
    ├── binder.rs        # Symbol table creation
    └── checker.rs       # Type checking
```

### Common Commands
```bash
# Build and test Rust
cd wasm && cargo build && cargo test

# Verify scanner output
node scripts/verifyScanner.mjs <file.ts>

# Build TypeScript with WASM
npx hereby local
```

## Core Patterns

For detailed patterns, see:
- [Rust Patterns](./patterns.md) - Borrow checker, wasm-bindgen, arena allocation
- [AST Reference](../agent_docs/ast-nodes.md) - Node structure and types
- [TypeScript Parser](../agent_docs/typescript-parser.md) - Original implementation reference

## Key Rules

1. **Never break the build** - Run tests before committing
2. **Match TypeScript exactly** - Token positions, AST structure, type inference
3. **Develop parser + checker together** - New syntax needs both parsing and type checking
4. **Use feature flags** - `--useRustScanner`, `--useRustParser`
5. **Commit frequently** - After each passing test run

## Development Workflow

### Adding New Syntax
1. **Parser**: Add AST node to `parser.rs`, parsing to `parser_impl.rs`
2. **Binder**: Handle node in `bind_node()` if it declares symbols
3. **Checker**: Add to `get_type_of_node_worker()` for type inference
4. **Tests**: Add parser test + checker test

### Adding Type Features
1. **Checker**: Add type variant to `Type` enum, update `TypeArena`
2. **Parser**: Ensure corresponding type syntax is parsed
3. **Checker**: Implement type relation rules in `is_type_assignable_to()`
