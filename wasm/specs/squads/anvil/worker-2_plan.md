# Worker 2 Plan - Squad Anvil

## Mission
Declaration File Emission (.d.ts)

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement Declaration File Generation**

### Background
TypeScript can generate .d.ts declaration files for libraries. Essential for publishing TypeScript packages.

### Implementation Steps

1. [ ] Read current declaration emission in `src/thin_emitter.rs` or `src/transforms/`
2. [ ] Implement .d.ts file generation:
   - Strip function bodies
   - Keep type annotations
   - Export declarations
   - Generate for all .ts files in compilation
3. [ ] Handle `--declaration` CLI flag (already added by W2)
4. [ ] Handle `--declarationMap` for source maps of .d.ts files
5. [ ] Test: Compile with `--declaration` and verify .d.ts files are created

### Key Code Locations
- `src/thin_emitter.rs` - main emitter
- `src/cli/args.rs` - --declaration flag exists
- `src/cli/driver.rs` - compilation driver

## Task Queue
- [ ] After declaration files: help with source maps or CLI features

## Completed
- [x] Private accessor collection - MERGED to squad/anvil
- [x] ES5 private accessor tests passing (with W3)
- [x] CLI flags: --declaration, --declarationMap, --sourceMap, --rootDir - MERGED to squad/anvil
- [x] LSP Signature Help - All 21 tests passing - MERGED to squad/anvil

## Ready for Merge
Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: Implement .d.ts declaration file generation`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-2`
