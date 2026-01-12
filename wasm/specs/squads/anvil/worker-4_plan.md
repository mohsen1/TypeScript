# Worker 4 Plan - Squad Anvil

## Mission
LSP Document Symbols

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement LSP Document Symbols**

### Background
Document symbols provides a tree outline of symbols in a file (classes, functions, variables). Essential for code navigation.

### Implementation Steps

1. [ ] Read current document symbols in `src/lsp/symbols.rs` or create if missing
2. [ ] Implement symbol extraction:
   - Traverse AST to find all symbols
   - Return hierarchical tree structure
   - Include symbol kinds (class, function, variable, interface, etc.)
3. [ ] Handle different symbol types:
   - Namespaces and modules
   - Classes and interfaces
   - Functions and methods
   - Variables and parameters
4. [ ] Support symbol range and selection range
5. [ ] Test: Request document symbols and verify tree structure

### Key Code Locations
- `src/lsp/symbols.rs` - document symbols implementation
- `src/lsp/mod.rs` - LSP server
- `src/binder/` - symbol resolution

## Task Queue
- [ ] After document symbols: help with rename symbol or workspace symbols

## Completed
- [x] Fixed shorthand methods binding - MERGED to squad/anvil
- [x] Diagnostic formatting with snippets - MERGED to squad/anvil
- [x] Parser recovery: JSX-like syntax and type assertion in new - MERGED to squad/anvil
- [x] Module System Emission review - Working correctly
- [x] Decorator Metadata Emission - All 171 tests passing
- [x] Generic Type Inference Fix - Test passes (commit 3cde014e51)

## Ready for Merge
Previous work merged to squad/anvil, latest ready for merge

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Implement document symbols`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
