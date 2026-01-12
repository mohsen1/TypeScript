# Worker 1 Plan - Squad Anvil

## Mission
Source Map Implementation

Status: COMPLETE ✅
Priority: P0 (Highest)

## Current Assignment
**Source Map Implementation - COMPLETE**

All components implemented and verified:
- VLQ encoding (905 tests pass)
- SourceMapGenerator API
- CLI --sourceMap flag
- .map file generation with valid JSON
- sourceMappingURL comments in .js output

## Task Queue
- [ ] Next task TBD (awaiting assignment)

## Completed
### Session 1: Emitter Edge Cases
- [x] Fixed catch clause variable emission (65b99a3305)
- [x] Added AS_EXPRESSION/TYPE_ASSERTION/SATISFIES_EXPRESSION handling (65b99a3305)

### Session 2: LSP Semantic Tokens Enhancement
- [x] Added semantic token support for decorators (adea84beac) - MERGED to squad/anvil
- [x] Added semantic token support for type parameters (adea84beac) - MERGED to squad/anvil
- [x] Added semantic token support for modifiers (adea84beac) - MERGED to squad/anvil

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
No new commits (implementation already existed)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] source_maps: Implement VLQ mapping generation`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
