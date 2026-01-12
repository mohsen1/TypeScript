# Worker 4 Plan - Squad Anvil

## Mission
Additional LSP Features

Status: Active
Priority: P1 (High)

## Current Assignment
**Enhance LSP Features**

### Background
Various LSP features need implementation for better IDE support.

### Implementation Steps

1. [ ] Read current LSP implementation in `src/lsp/`
2. [ ] Choose from available features:
   - Document formatting
   - Document highlighting
   - Folding ranges
   - Selection ranges
3. [ ] Implement chosen feature(s)
4. [ ] Test: Verify feature works in LSP client

### Key Code Locations
- `src/lsp/` - LSP implementations
- `src/lsp/mod.rs` - LSP server

## Task Queue
- [ ] After LSP features: help with emitter or CLI work

## Completed
- [x] Fixed shorthand methods binding - MERGED to squad/anvil
- [x] Diagnostic formatting with snippets - MERGED to squad/anvil
- [x] Parser recovery: JSX-like syntax and type assertion in new - MERGED to squad/anvil
- [x] Module System Emission review - Working correctly
- [x] Decorator Metadata Emission - All 171 tests passing
- [x] Generic Type Inference Fix - Test passes
- [x] LSP Document Symbols - All 5 tests PASS
- [x] LSP Rename Symbol - All tests passing, complete

## Ready for Merge
Previous work merged, latest ready for merge

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Implement document formatting`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
