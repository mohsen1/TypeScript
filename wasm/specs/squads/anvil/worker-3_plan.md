# Worker 3 Plan - Squad Anvil

## Mission
LSP Find References

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement LSP Find All References**

### Background
Find all references shows where a symbol is used throughout the codebase. Essential for refactoring.

### Implementation Steps

1. [ ] Read current find references in `src/lsp/references.rs` or create if missing
2. [ ] Implement reference finding:
   - Find symbol at cursor position
   - Search all files in project for symbol usage
   - Return list of locations (file, line, column)
3. [ ] Handle different symbol types:
   - Variables and functions
   - Class members
   - Parameters
   - Type aliases
4. [ ] Distinguish between definition, read, and write references
5. [ ] Test: Find references and verify all usages are found

### Key Code Locations
- `src/lsp/references.rs` - find references implementation
- `src/lsp/mod.rs` - LSP server
- `src/binder/` - symbol resolution

## Task Queue
- [ ] After find references: help with rename symbol or document symbols

## Completed
- [x] ES5 Private Accessor Emission (7 tests passing) - MERGED to squad/anvil
- [x] Parser error recovery: function keyword in class - MERGED to squad/anvil
- [x] LSP Go-To-Definition - All 25 tests passing - MERGED to squad/anvil

## Ready for Merge
Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Implement find all references`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
