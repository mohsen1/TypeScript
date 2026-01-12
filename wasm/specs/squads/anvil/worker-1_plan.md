# Worker 1 Plan - Squad Anvil

## Mission
Additional CLI Features

Status: Active
Priority: P1 (High)

## Current Assignment
**Add More CLI Flags**

### Background
TypeScript compiler has many CLI flags. Add more commonly-used flags for better compatibility.

### Implementation Steps

1. [ ] Read current CLI implementation in `src/cli/args.rs`
2. [ ] Check which flags from `tsc --help` are still missing
3. [ ] Add support for important flags:
   - `--outFile`: Concatenate and emit output to single file
   - `--outDir`: Output directory (may already exist)
   - `--tsBuildInfoFile`: Specify .tsbuildinfo file
   - `--incremental`: Enable incremental compilation
4. [ ] Update argument parsing in clap configuration
5. [ ] Test: Run `./wasm/target/release/tsz --help` and verify flags are listed

### Key Code Locations
- `src/cli/args.rs` - CLI argument definitions
- `src/cli/driver.rs` - compilation driver

## Task Queue
- [ ] After CLI flags: help with LSP features or emitter work

## Completed
- [x] Fixed catch clause variable emission - MERGED to squad/anvil
- [x] Added AS_EXPRESSION/TYPE_ASSERTION/SATISFIES_EXPRESSION handling - MERGED to squad/anvil
- [x] LSP Semantic Tokens (decorators, type parameters, modifiers) - MERGED to squad/anvil
- [x] Source Map Implementation - Verified complete (905 tests passing)
- [x] Import Equals Emission Fix - Test passes (commit 8271a07cb1)

## Ready for Merge
Previous work merged to squad/anvil, latest ready for merge

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] cli: Add --outFile and --incremental flags`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
