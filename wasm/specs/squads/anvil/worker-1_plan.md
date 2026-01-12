# Worker 1 Plan - Squad Anvil

## Mission
Fix Remaining Emitter Issues

Status: Active
Priority: P0 (Highest)

## Current Assignment
**Fix Generic Type Parameter Constraint Checking** - IN PROGRESS (since session 6)

### Background
Type parameter constraint checking issue: when calling `deepFreeze<T extends object>(value as object)`, the type checker incorrectly reports "Argument of type 'T' is not assignable to parameter of type 'object'".

### Failing Test
- `compile_generic_utility_library_type_utilities` - Tests generic type utilities with constraints
- Error: TS2345 at line 793 (recursive call `deepFreeze(value as object)`)

### Investigation Status
1. [x] Identified issue in `src/solver/operations.rs` - generic function call resolution
2. [x] Added fallback to constraint when resolved type is TypeParameter
3. [x] Added special case for `object` to TypeParameter with `object` constraint
4. [ ] **ISSUE PERSISTS**: Type parameter 'T' not being resolved to 'object'
5. [ ] Need deeper investigation of type parameter resolution in recursive calls

### Commits
- `db2e4e6c7d` - Initial constraint checking fix attempt
- `f3b659d09d` - Added special case for object to constrained type parameter

### Key Code Locations
- `src/transforms/class_es5.rs` - ES5 transforms
- `src/thin_emitter.rs` - main emitter

## Task Queue
- [ ] After emitter fixes: help with LSP or CLI features

## Completed
- [x] Fixed catch clause variable emission - MERGED to squad/anvil
- [x] Added AS_EXPRESSION/TYPE_ASSERTION/SATISFIES_EXPRESSION handling - MERGED to squad/anvil
- [x] LSP Semantic Tokens - MERGED to squad/anvil
- [x] Source Map Implementation - Verified complete
- [x] Import Equals Emission Fix - Test passes
- [x] CLI Flags (--outFile, --tsBuildInfoFile, --incremental) - Complete (commit 9754ae94f1)

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
Previous work merged, latest ready for merge

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: Fix remaining ES5 emission issues`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
