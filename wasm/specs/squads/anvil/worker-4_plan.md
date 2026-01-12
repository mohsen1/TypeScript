# Worker 4 Plan - Squad Anvil

## Mission
Generic Type Inference Enhancement

Status: Active
Priority: P1 (High)

## Current Assignment
**Fix Generic Utility Library Type Inference**

### Background
Complex generic patterns aren't inferring correctly. Need to enhance type inference for utility functions.

### Failing Test
`cli::driver_tests::compile_generic_utility_library_type_utilities`

### Error
```
TS2345: Argument of type 'T' is not assignable to parameter of type 'object'.
```

### Implementation Steps

1. [ ] Read the failing test to understand the generic patterns
2. [ ] Check type inference in `src/solver/infer.rs` or `src/solver/contextual.rs`
3. [ ] Investigate constraint handling for generic type parameters
4. [ ] Fix inference to properly handle:
   - Generic utility functions (map, filter, reduce)
   - Type parameter constraints
   - Conditional types
5. [ ] Test: `./wasm/test.sh compile_generic_utility_library_type_utilities`

### Key Code Locations
- `src/solver/infer.rs` - type inference
- `src/solver/contextual.rs` - contextual typing
- `src/solver/operations.rs` - type operations
- `src/solver/subtype.rs` - type compatibility

## Task Queue
- [ ] After generic inference: help with more CLI/Driver issues or LSP features

## Completed
- [x] Fixed shorthand methods binding - MERGED to squad/anvil
- [x] Diagnostic formatting with snippets - MERGED to squad/anvil
- [x] Parser recovery: JSX-like syntax and type assertion in new - MERGED to squad/anvil
- [x] Module System Emission review - Working correctly
- [x] Decorator Metadata Emission - All 171 tests passing, pushed to origin/worker/anvil-4

## Ready for Merge
Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: Fix generic type inference for utility functions`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
