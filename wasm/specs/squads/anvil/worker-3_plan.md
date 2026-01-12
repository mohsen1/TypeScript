# Worker 3 Plan - Squad Anvil

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
4. [ ] Handle edge cases:
   - Import equals with type annotations
   - Nested import equals
   - Import equals in different module systems
5. [ ] Test: `./wasm/test.sh invalidate_paths_with_dependents_symbols_handles_import_equals`

### Key Code Locations
- `src/thin_emitter.rs` - main emission
- `src/transforms/` - module transformations
- `src/cli/driver_tests.rs` - failing test

## Task Queue
- [ ] After import equals: help with generic utility library type inference or source maps

## Completed
- [x] ES5 Private Accessor Emission (7 tests passing) - MERGED to squad/anvil
- [x] Parser error recovery: test_thin_parser_function_keyword_in_class_recovers - MERGED to squad/anvil

## Ready for Merge
Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] emitter: Fix import equals transformation to CommonJS`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
