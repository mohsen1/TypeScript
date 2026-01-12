# Worker 4 Plan - Squad Anvil

## Mission
LSP Document Highlighting

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement LSP Document Highlighting**

### Background
Document highlighting provides symbol highlights (read/write occurrences) for better code navigation.

### Implementation Steps

1. [ ] Read current highlighting in `src/lsp/highlighting.rs` or create if missing
2. [ ] Implement document highlighting:
   - Find symbol at cursor position
   - Find all occurrences in document
   - Distinguish between read, write, and reference occurrences
3. [ ] Return list of document highlight ranges
4. [ ] Test: Request highlights and verify correct ranges

### Key Code Locations
- `src/lsp/highlighting.rs` - highlighting implementation
- `src/lsp/mod.rs` - LSP server
- `src/binder/` - symbol resolution

## Task Queue
- [ ] After highlighting: help with folding ranges or selection ranges

## Completed
- [x] Fixed shorthand methods binding - MERGED to squad/anvil
- [x] Diagnostic formatting with snippets - MERGED to squad/anvil
- [x] Parser recovery: JSX-like syntax and type assertion in new - MERGED to squad/anvil
- [x] Module System Emission review - Working correctly
- [x] Decorator Metadata Emission - All 171 tests passing
- [x] Generic Type Inference Fix - Test passes
- [x] LSP Document Symbols - All 5 tests PASS
- [x] LSP Rename Symbol - All tests passing
- [x] LSP Document Formatting - All tests PASS (commit 805a12080f)

## Ready for Merge
Previous work merged, latest ready for merge

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Implement document highlighting`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
