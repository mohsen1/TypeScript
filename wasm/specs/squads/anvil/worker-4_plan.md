# Worker 4 Plan - Squad Anvil

## Mission
Decorators Metadata Emission

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement Decorator Metadata Emission**

### Background
Decorators need metadata for runtime reflection. TypeScript emits `__decorate` helper calls.

### Implementation Steps

1. [ ] Read current decorator handling in `src/transforms/` and `src/thin_emitter.rs`
2. [ ] Implement decorator metadata:
   - Generate `__decorate` helper function if not exists
   - Emit decorator applications at runtime
   - Handle class decorators, method decorators, parameter decorators
3. [ ] Order decorator execution correctly (bottom-up for parameters, top-down for classes)
4. [ ] Test: Compile class with decorators and verify `__decorate` calls

### Key Code Locations
- `src/transforms/` - decorator transformations
- `src/thin_emitter.rs` - emission
- `src/thin_parser.rs` - decorator parsing

## Task Queue
- [ ] After decorator metadata: help with source maps or CLI features

## Completed
- [x] Fixed shorthand methods binding - MERGED to squad/anvil
- [x] Diagnostic formatting with snippets - MERGED to squad/anvil
- [x] Parser recovery: JSX-like syntax and type assertion in new - MERGED to squad/anvil
- [x] Module System Emission review - Working correctly, no fixes needed

## Ready for Merge
Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] transforms: Implement decorator metadata emission`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
