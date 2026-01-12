# Worker 2 Plan - Squad Anvil

## Mission
Declaration File Emission (.d.ts)

Status: Complete - Feature Already Implemented
Priority: P1 (High)

## Current Assignment
**[COMPLETE] Declaration File Emission (.d.ts)**

### Background
TypeScript declaration files (.d.ts) provide type information for JavaScript libraries. The compiler must generate these files from TypeScript source.

### Implementation Summary

The declaration file emission feature is **fully implemented** in `src/declaration_emitter.rs`:

1. **DeclarationEmitter class** - Generates .d.ts files by:
   - Stripping function bodies (keeping only signatures)
   - Preserving type annotations
   - Handling all declaration types (functions, classes, interfaces, enums, type aliases)
   - Supporting `declare` keyword for ambient contexts
   - Handling export/import statements

2. **CLI Integration** - `src/cli/driver.rs`:
   - Handles `--declaration` flag via `emit_declarations` option
   - Generates .d.ts files alongside .js output
   - Supports `--declarationDir` for custom output directory
   - Supports `--declarationMap` for source maps

3. **CLI Flags** (added in earlier work):
   - `--declaration` / `-d` - Generate .d.ts files
   - `--declarationMap` - Generate .d.ts.map files
   - `--sourceMap` - Generate .map files
   - `--rootDir` - Root directory
   - `--outDir` - Output directory

### Test Status
All 6 declaration tests passing:
- ✅ compile_declaration_true_emits_dts_files
- ✅ compile_declaration_false_no_dts_files
- ✅ compile_declaration_absent_no_dts_files
- ✅ compile_declaration_interface_and_type
- ✅ compile_declaration_class_with_methods
- ✅ compile_declaration_with_declaration_dir

### Key Code Locations
- `src/declaration_emitter.rs` - DeclarationEmitter implementation
- `src/cli/driver.rs` - CLI integration (lines 3000-3043)
- `src/cli/args.rs` - CLI argument definitions
- `src/cli/config.rs` - Compiler options

## Task Queue
- Awaiting next assignment

## Completed
- [x] Private accessor collection (c76225e474) - MERGED to squad/anvil
- [x] All 7 ES5 private accessor tests passing
- [x] CLI flags enhancement: --declaration, --declarationMap, --sourceMap, --rootDir (2f5834504e) - MERGED to squad/anvil
- [x] Declaration file emission (.d.ts) - Already fully implemented and tested

## Ready for Merge
No new commits - Declaration emission was already implemented

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] cli: Add --declaration and --sourceMap flags`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-2`
