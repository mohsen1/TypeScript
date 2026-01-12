# Worker 2 Plan - Squad Anvil

## Mission
CLI Flags Enhancement

Status: Complete
Priority: P1 (High)

## Current Assignment
**[COMPLETE] Implement Missing CLI Flags**

### Background
The TypeScript compiler has many CLI flags. Our WASM compiler needs to support common flags for compatibility.

### Implementation Steps

1. [x] Read current CLI implementation in `src/cli/driver.rs`
2. [x] Check which flags from `tsc --help` are missing
3. [x] Add support for missing important flags:
   - `--declaration` or `-d`: Generate .d.ts files
   - `--declarationMap`: Generate .d.ts.map files
   - `--sourceMap` or `-sourcemaps`: Generate .map files
   - `--outDir`: Output directory (already existed)
   - `--rootDir`: Root directory
4. [x] Update argument parsing in clap configuration
5. [x] Test: Run `./wasm/target/release/tsz --help` and verify flags are listed

### Key Code Locations
- `src/cli/driver.rs` - CLI driver and argument parsing
- `src/cli/args.rs` - CLI argument definitions
- `src/thin_emitter.rs` - source map emission

## Task Queue
- Awaiting next assignment

## Completed
- [x] Private accessor collection (c76225e474) - MERGED to squad/anvil (with Worker 3 emission)
- [x] All 7 ES5 private accessor tests now passing (Worker 3 completed emission)
- [x] CLI flags enhancement: --declaration, --declarationMap, --sourceMap, --rootDir (2f5834504e)

## Ready for Merge
Yes

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] cli: Add --declaration and --sourceMap flags`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-2`
