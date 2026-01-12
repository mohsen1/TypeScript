# Worker 1 Plan - Squad Anvil

## Mission
Additional CLI Features

Status: COMPLETE ✅
Priority: P0 (Highest)

## Current Assignment
**Add More CLI Flags - COMPLETE**

All flags implemented and verified:
- --outFile <FILE> - Concatenate and emit output to single file
- --tsBuildInfoFile <FILE> - Specify .tsbuildinfo file path
- --incremental - Enable incremental compilation

### Background
TypeScript compiler has many CLI flags. Add more commonly-used flags for better compatibility.

### Implementation Steps
1. [x] Read current CLI implementation in `src/cli/args.rs`
2. [x] Add support for important flags:
   - `--outFile`: Concatenate and emit output to single file
   - `--tsBuildInfoFile`: Specify .tsbuildinfo file
   - `--incremental`: Enable incremental compilation
3. [x] Update argument parsing in clap configuration
4. [x] Wire up flags to config.rs and driver.rs
5. [x] Test: Run `./wasm/target/release/tsz --help` and verify flags are listed

### Key Code Locations
- `src/cli/args.rs:26-48` - Added CLI flag definitions
- `src/cli/config.rs:50-63` - Added CompilerOptions fields
- `src/cli/config.rs:91-97` - Added ResolvedCompilerOptions fields
- `src/cli/config.rs:181,186-187` - Added Default impl
- `src/cli/config.rs:295-327` - Added resolution logic
- `src/cli/config.rs:427,432-433` - Added merge logic
- `src/cli/driver.rs:3271-3279` - Added CLI override logic

## Task Queue
- [ ] After CLI flags: help with LSP features or emitter work

## Completed
- [x] Fixed catch clause variable emission - MERGED to squad/anvil
- [x] Added AS_EXPRESSION/TYPE_ASSERTION/SATISFIES_EXPRESSION handling - MERGED to squad/anvil
- [x] LSP Semantic Tokens (decorators, type parameters, modifiers) - MERGED to squad/anvil
- [x] Source Map Implementation - Verified complete (905 tests passing)
- [x] Import Equals Emission Fix - Test passes (commit 8271a07cb1)

### Session 5: Additional CLI Features - COMPLETE ✅
- [x] Added --outFile flag (concatenate output to single file)
- [x] Added --tsBuildInfoFile flag (specify .tsbuildinfo file path)
- [x] Added --incremental flag (enable incremental compilation)
- [x] Wired flags to CompilerOptions in config.rs
- [x] Added CLI override handling in driver.rs
- [x] All flags verified in --help output
- Commit: 9754ae94f1

### Session 4: Import Equals Emission Fix - COMPLETE ✅
- [x] Fixed import equals type to return `any` instead of `string`
- [x] For `import x = require('y')`, module_specifier is a StringLiteral
- [x] Added check in `src/thin_checker.rs:5184-5190` to return TypeId::ANY for StringLiteral module_specifiers
- [x] Test passes: `invalidate_paths_with_dependents_symbols_handles_import_equals`
- Commit: 8271a07cb1

### Session 3: Source Map Implementation - VERIFICATION COMPLETE ✅
- [x] VLQ encoding fully implemented (vlq::encode, vlq::decode)
- [x] SourceMapGenerator with add_mapping, generate, generate_json, generate_inline
- [x] CLI --sourceMap flag implemented in args.rs
- [x] Driver wires up source map generation (enable_source_map, generate_source_map_json, sourceMappingURL)
- [x] All 905 source map tests pass
- [x] **CLI Testing Verified**:
  - `--sourceMap` flag generates .js.map files alongside .js output
  - Generated .map file contains valid JSON with:
    - version: 3
    - file: output filename
    - sources: source file paths
    - sourcesContent: embedded source content
    - names: array of identifiers
    - mappings: VLQ-encoded mappings
  - Generated .js file includes `//# sourceMappingURL=` comment
  - VLQ encoding correctly encodes/decodes relative positions
- Status: Implementation complete and verified via CLI testing

## Ready for Merge
Yes (9754ae94f1)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] cli: Add --outFile and --incremental flags`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
