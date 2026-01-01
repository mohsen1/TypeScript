# TypeScript to Rust/WASM Migration Project

## Project Overview
We are incrementally migrating the TypeScript compiler to Rust, compiled to WebAssembly.
This follows the "Strangler Fig" pattern - Rust components progressively replace TypeScript
modules while the compiler remains fully functional at every step.

## Critical Rules

### Never Break the Build
- Every commit must pass `npx hereby runtests-parallel`
- Run `npx hereby local` before any commit to ensure build passes
- Use feature flags (`--useRustScanner`, `--useRustParser`) for new Rust code paths

### Git Workflow
- Commit frequently after each passing test run
- Never commit broken code
- One logical change per commit
- Always run tests before committing

### Rust/WASM Development
- All Rust code lives in `wasm/` directory
- Use `cargo build` and `cargo test` from `wasm/` directory
- WASM output goes to `built/local/wasm/`
- TypeScript bridge is in `src/compiler/wasm.ts`

## Key Files

| File | Purpose |
|------|---------|
| `MIGRATION_PLAN.md` | Master plan with phases, progress log, and verification gates |
| `wasm/src/lib.rs` | Main Rust entry point, wasm-bindgen exports |
| `wasm/src/scanner.rs` | Token types, SyntaxKind enum |
| `wasm/src/scanner_impl.rs` | Scanner implementation |
| `wasm/src/parser.rs` | AST node definitions (~120 types) |
| `wasm/src/parser_impl.rs` | Parser implementation |
| `src/compiler/wasm.ts` | TypeScript bridge to WASM |
| `src/compiler/scanner.ts` | Contains `createRustScanner()` adapter |

## Current Phase: Phase 3 - Parser

### Completed
- Phase 0: Infrastructure (wasm crate, build integration)
- Phase 1: Utilities (string comparison, path utils, char classification)
- Phase 2: Scanner (full token scanning, rescan methods, template literals)
- Phase 3.1: AST Node Definitions (~120 node types)
- Phase 3.2: Parser Core (statements, expressions, classes, imports/exports, types)

### In Progress
- AST-to-TypeScript-AST conversion
- `--useRustParser` CLI flag integration

## Architecture Decisions

1. **Arena Allocation**: AST nodes use `NodeArena` with `NodeIndex` references
2. **Flag Constants**: Use `pub mod` with constants (not enums) for wasm-bindgen compatibility
3. **Serialization Boundary**: Use JSON serialization for Rust↔JS data transfer during migration
4. **Scanner Integration**: Parser uses `ScannerState` from `scanner_impl.rs`

## Testing Commands

```bash
# Build and test Rust
cd wasm && cargo build && cargo test

# Build TypeScript
npx hereby local

# Run TypeScript tests
npx hereby runtests-parallel

# Test with Rust scanner
node built/local/tsc.js --useRustScanner /path/to/file.ts --noEmit

# Verify scanner token-by-token
node scripts/verifyScanner.mjs src/compiler/checker.ts
```
