# Worker 3 Plan - Squad Anvil

## Mission
LSP Go-To-Definition Implementation

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement LSP Go-To-Definition**

### Background
Go-to-definition allows users to navigate to where symbols are defined. Critical IDE feature.

### Implementation Steps

1. [ ] Read current go-to-definition in `src/lsp/definition.rs`
2. [ ] Implement definition lookup:
   - Find symbol at cursor position
   - Look up symbol declaration in binder
   - Return location (file, line, column)
3. [ ] Handle different symbol types:
   - Variables and functions
   - Class members
   - Import/export declarations
   - Type aliases
4. [ ] Test: Start LSP server and verify F12/ Cmd+Click navigates to definitions

### Key Code Locations
- `src/lsp/definition.rs` - go-to-definition implementation
- `src/lsp/mod.rs` - LSP server
- `src/binder/` - symbol resolution

## Task Queue
- [ ] After go-to-definition: help with find-references or document symbols

## Completed
- [x] ES5 Private Accessor Emission (7 tests passing) - MERGED to squad/anvil
- [x] Parser error recovery: function keyword in class - MERGED to squad/anvil

## Ready for Merge
Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Implement go-to-definition navigation`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
