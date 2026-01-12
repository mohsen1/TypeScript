# Worker 1 Plan - Squad Anvil

## Mission
LSP Semantic Tokens Enhancement

Status: Active
Priority: P1 (High)

## Current Assignment
**Enhance LSP Semantic Tokens Implementation**

### Background
LSP semantic tokens provide IDEs with rich syntax highlighting. Current implementation needs enhancement for better TypeScript language support.

### Implementation Steps

1. [ ] Read current semantic tokens implementation in `src/lsp/semantic_tokens.rs`
2. [ ] Check which token types are missing or incorrect compared to VS Code's TypeScript server
3. [ ] Add support for missing token types:
   - Decorators
   - Type parameters
   - Namespace/module declarations
   - Modifier keywords (readonly, static, etc.)
4. [ ] Ensure proper token modifiers (readonly, static, async, etc.)
5. [ ] Test: Run LSP server and verify token output matches tsserver

### Key Code Locations
- `src/lsp/semantic_tokens.rs` - semantic token implementation
- `src/lsp/mod.rs` - LSP server

## Task Queue
- [ ] After LSP semantic tokens: help with source maps or diagnostic formatting

## Completed
- [x] Fixed catch clause variable emission (65b99a3305) - MERGED to squad/anvil
- [x] Fixed type assertion emission in ClassES5Emitter (65b99a3305) - MERGED to squad/anvil
- [x] Fixed try-throw parentheses (test_two_phase_emission_es5_class_try_throw_parenthesized) - MERGED

## Ready for Merge
Yes - Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Enhance semantic tokens with decorators and modifiers`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-1`
