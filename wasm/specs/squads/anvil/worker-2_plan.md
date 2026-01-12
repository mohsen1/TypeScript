# Worker 2 Plan - Squad Anvil

## Mission
Declaration File Emission (.d.ts)

Status: Complete - Feature Already Implemented
Priority: P1 (High)

## Current Assignment
**[COMPLETE] Declaration File Emission (.d.ts)**

### Background
TypeScript can generate .d.ts declaration files for libraries. Essential for publishing TypeScript packages.

### Implementation Summary

The declaration file emission feature is **fully implemented** in `src/declaration_emitter.rs`:

**DeclarationEmitter class** generates .d.ts files by:
- Stripping function bodies (adding `declare` keyword)
- Preserving all type annotations
- Handling all declaration types:
  - Function declarations
  - Class declarations (with members)
  - Interface declarations
  - Type aliases
  - Enum declarations
  - Variable declarations
  - Export/import statements

**CLI Integration** (`src/cli/driver.rs:3000-3043`):
- Handles `--declaration` flag via `emit_declarations` option
- Generates .d.ts files for all .ts files in compilation
- Supports `--declarationDir` for custom output directory
- Supports `--declarationMap` for source maps

### Implementation Details

The `DeclarationEmitter` processes each source file statement:
1. Checks if statement should be in declaration file (exported declarations)
2. Strips function bodies, keeps signatures
3. Preserves type annotations
4. Outputs `declare` keyword for appropriate declarations

### Test Status
All 6 declaration tests passing:
- ✅ compile_declaration_true_emits_dts_files
- ✅ compile_declaration_false_no_dts_files
- ✅ compile_declaration_absent_no_dts_files
- ✅ compile_declaration_interface_and_type
- ✅ compile_declaration_class_with_methods
- ✅ compile_declaration_with_declaration_dir

### Key Code Locations
- `src/declaration_emitter.rs` - DeclarationEmitter implementation (strips bodies, keeps types)
- `src/cli/driver.rs` - CLI integration (lines 3000-3043)
- `src/cli/args.rs` - CLI argument definitions (--declaration, --declarationMap)
- `src/cli/config.rs` - Compiler options (emit_declarations)

## Task Queue
- Awaiting next assignment

## Completed
- [x] Private accessor collection - MERGED to squad/anvil
- [x] ES5 private accessor tests passing (with W3)
- [x] CLI flags: --declaration, --declarationMap, --sourceMap, --rootDir - MERGED to squad/anvil
- [x] LSP Signature Help - All 21 tests passing - MERGED to squad/anvil
- [x] Declaration file emission (.d.ts) - Fully implemented and tested

## Ready for Merge
Yes - No new commits needed, feature was already implemented

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: Implement .d.ts declaration file generation`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-2`
