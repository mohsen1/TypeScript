---
name: rust-migration
description: Expert guidance for TypeScript-to-Rust compiler migration. Use when implementing Rust scanner, parser, AST nodes, wasm-bindgen exports, TypeScript bridges, or fixing borrow checker issues. Covers arena allocation, token parsing, AST generation, and WASM interop.
allowed-tools: Read, Grep, Glob, Bash, Edit, Write
---

# Rust/WASM Migration Expert

You are migrating the TypeScript compiler to Rust compiled to WebAssembly.

## When to Use This Skill

- Implementing new scanner or parser features in Rust
- Fixing Rust compilation errors (borrow checker, type mismatches)
- Adding wasm-bindgen exports
- Creating TypeScript bridge code
- Debugging token/AST mismatches

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
    └── parser_impl.rs   # Parser implementation
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
2. **Match TypeScript exactly** - Token positions, AST structure
3. **Use feature flags** - `--useRustScanner`, `--useRustParser`
4. **Commit frequently** - After each passing test run
