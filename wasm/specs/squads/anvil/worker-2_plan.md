# Worker 2 Plan - Squad Anvil

## Mission
LSP Signature Help Enhancement

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement LSP Signature Help**

### Background
Signature help shows parameter hints when calling functions. Essential for IDE integration.

### Implementation Steps

1. [ ] Read current signature help implementation in `src/lsp/signature_help.rs`
2. [ ] Check what's missing compared to VS Code's TypeScript server
3. [ ] Implement signature help with:
   - Function signature lookup from binder
   - Parameter information (names, types, optional flags)
   - Active parameter highlighting based on cursor position
   - Overload resolution for multiple signatures
4. [ ] Add support for:
   - Method calls
   - Constructor calls
   - Callable type invocations
5. [ ] Test: Start LSP server and verify signature help appears for function calls

### Key Code Locations
- `src/lsp/signature_help.rs` - signature help implementation
- `src/lsp/mod.rs` - LSP server
- `src/binder/` - symbol and signature resolution

## Task Queue
- [ ] After signature help: help with go-to-definition or document symbols

## Completed
- [x] Private accessor collection (c76225e474) - MERGED to squad/anvil
- [x] All 7 ES5 private accessor tests passing
- [x] CLI flags enhancement: --declaration, --declarationMap, --sourceMap, --rootDir (2f5834504e) - MERGED to squad/anvil

## Ready for Merge
Previous work merged to squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] lsp: Implement signature help with parameter hints`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-2`
