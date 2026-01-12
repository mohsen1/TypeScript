# Worker 1 Plan - Squad Anvil

## Mission
Source Map Implementation

Status: Active
Priority: P0 (Highest)

## Current Assignment
**Implement Source Map Generation**

### Background
Source maps allow debugging tools to map generated/transpiled code back to original source. Essential for debugging compiled TypeScript.

### Implementation Steps
1. [ ] Read current source map infrastructure in `src/source_map*.rs`
2. [ ] Check what's missing compared to standard source map v3 spec
3. [ ] Implement source map generation:
   - Track mappings from output positions to source positions
   - Generate VLQ-encoded mappings
   - Create .map file alongside .js output
4. [ ] Add source content to maps for better debugging
5. [ ] Test: Compile a file and verify .map file is generated correctly

### Key Code Locations
- `src/source_map*.rs` - source map implementation
- `src/thin_emitter.rs` - add mapping tracking during emission
- `src/cli/args.rs` --sourceMap flag already added

## Task Queue
- [ ] After source maps: help with more LSP features or additional CLI flags

## Completed
### Session 1: Emitter Edge Cases
- [x] Fixed catch clause variable emission (65b99a3305)
- [x] Added AS_EXPRESSION/TYPE_ASSERTION/SATISFIES_EXPRESSION handling (65b99a3305)

### Session 2: LSP Semantic Tokens Enhancement
- [x] Added semantic token support for decorators (adea84beac) - MERGED to squad/anvil
- [x] Added semantic token support for type parameters (adea84beac) - MERGED to squad/anvil
- [x] Added semantic token support for modifiers (adea84beac) - MERGED to squad/anvil

### Session 3: Source Map Implementation - VERIFICATION ✅
- [x] VLQ encoding fully implemented (vlq::encode, vlq::decode)
- [x] SourceMapGenerator with add_mapping, generate, generate_json, generate_inline
- [x] CLI --sourceMap flag implemented in args.rs
- [x] Driver wires up source map generation (enable_source_map, generate_source_map_json, sourceMappingURL)
- [x] All 905 source map tests pass
- Status: Implementation was already complete - verified and documented

## Ready for Merge
No new commits (implementation already existed)

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] source_maps: Implement VLQ mapping generation`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
