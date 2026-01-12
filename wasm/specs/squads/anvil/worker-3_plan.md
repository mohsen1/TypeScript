# Worker 3 Plan - Squad Anvil

## Mission
LSP Code Actions

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement LSP Code Actions**

### Background
Code actions provide quick fixes and refactorings (e.g., "organize imports", "fix all", "extract to function"). Essential for IDE UX.

### Implementation Steps

1. [ ] Read current code actions in `src/lsp/code_actions.rs` or create if missing
2. [ ] Implement common code actions:
   - Organize imports (sort and deduplicate)
   - Remove unused imports
   - Fix all auto-fixable errors
   - Extract to function/refactor
3. [ ] Handle code action resolution
4. [ ] Test: Request code actions and verify suggestions

### Key Code Locations
- `src/lsp/code_actions.rs` - code actions implementation
- `src/lsp/mod.rs` - LSP server

## Task Queue
- [ ] After code actions: help with other LSP features or emitter work

## Completed
- [x] ES5 Private Accessor Emission (7 tests passing) - MERGED to squad/anvil
- [x] Parser error recovery: function keyword in class - MERGED to squad/anvil
- [x] LSP Go-To-Definition - All 25 tests passing - MERGED to squad/anvil
- [x] LSP Find All References - All 48 LSP tests passing - Complete

## Ready for Merge
Previous work merged, latest ready for merge

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Implement code actions`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
