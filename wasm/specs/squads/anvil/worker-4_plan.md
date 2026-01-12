# Worker 4 Plan - Squad Anvil

## Mission
Diagnostic Formatting Enhancement

Status: Active
Priority: P1 (High)

## Current Assignment
**Enhance Diagnostic Error Messages**

### Background
TypeScript's error messages are detailed and helpful. Our diagnostic messages should match tsc's format and quality.

### Implementation Steps

1. [ ] Read current diagnostic implementation in `src/checker/types/diagnostics/`
2. [ ] Check which error messages differ from tsc
3. [ ] Enhance error messages with:
   - Better context information
   - Suggestions for fixes
   - Related code locations
4. [ ] Format error messages to match tsc output:
   - File location (line:column)
   - Error code TS#####: Message
   - Underlined error span
5. [ ] Test: Compare output with `npx tsc --noEmit`

### Key Code Locations
- `src/checker/types/diagnostics/` - diagnostic messages and codes
- `src/checker/types/diagnostics/diagnostic_messages.rs` - message templates

## Task Queue
- [ ] After diagnostic formatting: help with source maps or CLI flags

## Completed
- [x] Fixed shorthand methods binding in object literals (fc2b39a217) - MERGED to squad/anvil
- [x] Fixed declare_symbol persistence for scope_chain updates (fc2b39a217)

## Ready for Merge
Yes - Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] diagnostics: Enhance error message formatting with context`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
