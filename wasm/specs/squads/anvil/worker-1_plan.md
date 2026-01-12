# Worker 1 Plan - Squad Anvil

## Mission
Import Equals Emission Fix

Status: Active
Priority: P0 (Highest)

## Current Assignment
**Fix Import Equals Emission**

### Background
`import x = require('y')` is TypeScript-specific syntax for CommonJS imports. Current implementation has issues.

### Failing Test
`cli::driver_tests::invalidate_paths_with_dependents_symbols_handles_import_equals`

### Implementation Steps
1. [x] Read the failing test to understand expected behavior
2. [x] Find import equals handling in `src/thin_checker.rs`
3. [x] Fix type inference - was returning `string` instead of `any`
4. [x] Test: `invalidate_paths_with_dependents_symbols_handles_import_equals` passes

### Key Code Locations
- `src/thin_checker.rs:5184-5190` - Fixed to check for StringLiteral and return TypeId::ANY

### Fix Applied
In `src/thin_checker.rs`, added check for StringLiteral module_specifier:
- For `import x = require('y')`, module_specifier is a StringLiteral
- Previously: `get_type_of_node(StringLiteral)` returned `string` type
- Fixed: Return `TypeId::ANY` for StringLiteral module_specifiers

## Task Queue
- [ ] After import equals: help with other CLI/Driver issues

## Completed
- [x] Fixed catch clause variable emission - MERGED to squad/anvil
- [x] Added AS_EXPRESSION/TYPE_ASSERTION/SATISFIES_EXPRESSION handling - MERGED to squad/anvil
- [x] LSP Semantic Tokens (decorators, type parameters, modifiers) - MERGED to squad/anvil
- [x] Source Map Implementation - Verified complete (905 tests passing)

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
Yes (8271a07cb1)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: Fix import equals transformation`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
