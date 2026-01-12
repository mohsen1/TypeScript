# Worker 1 Plan - Squad Anvil

## Mission
Additional Emitter Features

Status: Active
Priority: P0 (Highest)

## Current Assignment
**Fix Remaining Emitter Issues**

### Background
ES5 class transforms and emitter edge cases need attention.

### Implementation Steps
1. [ ] Check for failing emitter tests in `src/transforms/*_tests.rs`
2. [ ] Review readonly modifier emission
3. [ ] Fix any remaining try/catch/finally emission issues
4. [ ] Test: `./wasm/test.sh --test emitter`

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

## Ready for Merge
Previous work merged, latest ready for merge

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: Fix remaining ES5 emission issues`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
