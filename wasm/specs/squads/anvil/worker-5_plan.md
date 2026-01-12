# Worker 5 Plan - Squad Anvil

## Mission
LSP Completion Enhancement

Status: Active
Priority: P1 (High)

## Current Assignment
**Enhance LSP Auto-Completion**

### Background
LSP completion (code suggestions) needs improvement for better TypeScript IDE support.

### Implementation Steps
1. [ ] Read current completion implementation in `src/lsp/completion.rs`
2. [ ] Check which completion types are missing:
   - Property completions on objects
   - Method completions
   - Import/export completions
   - Keyword completions
3. [ ] Enhance completion with:
   - Better context awareness
   - Symbol lookups from binder
   - Type information for completions
4. [ ] Add completion resolve for additional details
5. [ ] Test: Start LSP server and test completions in VS Code

### Key Code Locations
- `src/lsp/completion.rs` - completion implementation
- `src/lsp/mod.rs` - LSP server
- `src/binder/` - symbol resolution

## Task Queue
- [ ] After LSP completion: help with signature help or go-to-definition

## Completed
- [x] Parser error recovery: All 4 tests fixed - MERGED to squad/anvil
  - test_in_operator_private_identifier_narrows_required_property ✅
  - test_thin_parser_function_keyword_in_class_recovers ✅
  - test_thin_parser_jsx_like_syntax_in_ts_recovers ✅
  - test_thin_parser_type_assertion_in_new_expression_reports_ts1109 ✅

## Ready for Merge
Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Enhance code completion with symbol lookups`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-5`
