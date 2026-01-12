# Worker 4 Plan - Squad Anvil

## Mission
LSP Rename Symbol

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement LSP Rename Symbol**

### Background
Rename symbol allows renaming identifiers across all references. Critical for refactoring.

### Implementation Steps

1. [ ] Read current rename implementation in `src/lsp/rename.rs` or create if missing
2. [ ] Implement rename functionality:
   - Find symbol at cursor position
   - Find all references to that symbol
   - Prepare workspace edit with all changes
   - Validate rename (check for conflicts)
3. [ ] Handle different symbol types:
   - Variables and functions
   - Class members
   - Parameters
4. [ ] Support rename preview
5. [ ] Test: Rename a symbol and verify all references updated

### Key Code Locations
- `src/lsp/rename.rs` - rename implementation
- `src/lsp/mod.rs` - LSP server
- `src/binder/` - symbol resolution

## Task Queue
- [ ] After rename: help with code actions or workspace symbols

## Completed
- [x] Fixed shorthand methods binding - MERGED to squad/anvil
- [x] Diagnostic formatting with snippets - MERGED to squad/anvil
- [x] Parser recovery: JSX-like syntax and type assertion in new - MERGED to squad/anvil
- [x] Module System Emission review - Working correctly
- [x] Decorator Metadata Emission - All 171 tests passing
- [x] Generic Type Inference Fix - Test passes
- [x] LSP Document Symbols - All 5 tests PASS (commit 77bf219aaf)

## Ready for Merge
Previous work merged, latest ready for merge

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Implement rename symbol`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
