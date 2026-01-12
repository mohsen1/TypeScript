# Worker 1 Plan - Squad Anvil

## Mission
Type Solver Constraint Enhancement

Status: Active
Priority: P1 (High)

## Current Assignment
**Fix Generic Type Parameter Constraint Checking**

### Background
The test `compile_generic_utility_library_type_utilities` fails with TS2345 when T extends object. Type parameter constraints aren't being properly handled.

### Failing Test
`cli::driver_tests::compile_generic_utility_library_type_utilities`

### Error
```
TS2345: Argument of type 'T' is not assignable to parameter of type 'object'.
```

### Implementation Steps

1. [ ] Read the failing test to understand the generic pattern
2. [ ] Check constraint handling in `src/solver/operations.rs`
3. [ ] Fix type parameter constraint checking:
   - When T extends object, T should be assignable to object
   - Check constraint resolution in subtype checks
4. [ ] Test: `./wasm/test.sh compile_generic_utility_library_type_utilities`

### Key Code Locations
- `src/solver/operations.rs` - constraint checking
- `src/solver/infer.rs` - type inference
- `src/solver/subtype.rs` - type compatibility

## Task Queue
- [ ] After constraint fix: help with other solver or emitter issues

## Completed
- [x] Fixed catch clause variable emission - MERGED to squad/anvil
- [x] Added AS_EXPRESSION/TYPE_ASSERTION/SATISFIES_EXPRESSION handling - MERGED to squad/anvil
- [x] LSP Semantic Tokens - MERGED to squad/anvil
- [x] Source Map Implementation - Verified complete
- [x] Import Equals Emission Fix - Test passes
- [x] CLI Flags (--outFile, --tsBuildInfoFile, --incremental) - Complete
- [x] Emitter Fixes - All 756/756 emitter tests pass (commit 0e3b0b935f)

## Ready for Merge
Previous work merged, latest ready for merge

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: Fix generic type parameter constraint checking`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
