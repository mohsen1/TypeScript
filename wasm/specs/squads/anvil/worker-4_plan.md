# Worker 4 Plan - Squad Anvil

## Mission
Module System Emission Enhancement

Status: Active
Priority: P1 (High)

## Current Assignment
**Fix Module System Transformations**

### Background
Module transformations (ES modules ↔ CommonJS) need work. Import equals is tricky - assign to W5 instead.

### Implementation Steps

1. [ ] Read current module transforms in `src/transforms/`
2. [ ] Focus on ES module emission:
   - export statements
   - import statements
   - re-exports (export { x } from './y')
3. [ ] Fix CommonJS wrapper:
   - Module wrapper function
   - exports object handling
   - require() resolution
4. [ ] Test: Compile with different module settings and verify output

### Key Code Locations
- `src/transforms/` - module transformations
- `src/thin_emitter.rs` - emission
- `src/cli/args.rs` - module flag

## Task Queue
- [ ] After module transforms: help with decorator metadata or source maps

## Completed
- [x] Fixed shorthand methods binding (fc2b39a217) - MERGED to squad/anvil
- [x] Diagnostic formatting with snippets (97167d3e5e) - MERGED to squad/anvil
- [x] Parser recovery: JSX-like syntax and type assertion in new - MERGED to squad/anvil

## Ready for Merge
Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] transforms: Fix ES module emission`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
