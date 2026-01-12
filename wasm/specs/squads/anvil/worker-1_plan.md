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
1. [ ] Read the failing test to understand expected behavior
2. [ ] Find import equals handling in `src/thin_emitter.rs` or `src/transforms/`
3. [ ] Ensure proper transformation: `import x = require('y')` → `var x = require('y')`
4. [ ] Fix type inference - should be `any` type, not `string`
5. [ ] Test: `./wasm/test.sh invalidate_paths_with_dependents_symbols_handles_import_equals`

### Key Code Locations
- `src/thin_emitter.rs` - main emission
- `src/thin_checker.rs` - type checking (fix to return any for import equals)
- `src/binder.rs` - symbol binding (declare_in_persistent_scope)

## Task Queue
- [ ] After import equals: help with other CLI/Driver issues

## Completed
- [x] Fixed catch clause variable emission - MERGED to squad/anvil
- [x] Added AS_EXPRESSION/TYPE_ASSERTION/SATISFIES_EXPRESSION handling - MERGED to squad/anvil
- [x] LSP Semantic Tokens (decorators, type parameters, modifiers) - MERGED to squad/anvil
- [x] Source Map Implementation - Verified complete (905 tests passing)

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
No commits (task in progress)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: Fix import equals transformation`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
