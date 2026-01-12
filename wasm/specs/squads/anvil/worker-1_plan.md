# Worker 1 Plan - Squad Anvil

## Mission
Import Equals Emission Fix

Status: Active
Priority: P1 (High)

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

## Ready for Merge
Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: Fix import equals transformation`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
