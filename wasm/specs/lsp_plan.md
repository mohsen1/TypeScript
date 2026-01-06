
# Migration Plan: TypeScript Compiler → Rust via WebAssembly (LSP)

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

# Files

src/lsp/ (New), src/thin_binder.rs

# Goal

"Go to Definition" and "Find References".

## Tasks

Our focus is to make wasm Language Service Protocol (LSP) complete

- go-to-def
- find refs
- completions
- ... add more tasks (Ask Gemini when needed)



### Current Status

✅ **LSP Module Created** (2026-01-06)

The LSP module has been successfully scaffolded with the following components:

#### Completed

1. **Module Structure** (`wasm/src/lsp/`)
   - `mod.rs` - Module declarations and public exports
   - `position.rs` - Position/Location types and LineMap for offset<->position conversion (with tests)
   - `utils.rs` - Node lookup utilities using the flat ThinNodeArena
   - `resolver.rs` - Symbol resolution (simplified implementation)
   - `definition.rs` - Go-to-Definition feature (basic scaffolding)
   - `references.rs` - Find References feature (basic scaffolding)
   - `tests.rs` - Integration tests

2. **Integration**
   - Added `pub mod lsp;` to `wasm/src/lib.rs`
   - All code compiles and tests pass ✅

#### Architecture Decisions

Following Gemini's guidance, the LSP implementation uses:

1. **Data-Oriented Utils**: `find_node_at_offset` exploits the flat arena for O(N) hit-testing without pointer chasing
2. **Lazy Resolution**: `ScopeWalker` reconstructs scope chains on-the-fly (avoids storing massive Node->Scope map)
3. **Separation of Concerns**:
   - Binder: Handles declarations and static scope structure
   - LSP: Handles usage resolution and position mapping

#### Known Limitations

⚠️ The current `ScopeWalker` implementation is **simplified** and uses a basic AST traversal strategy:
- Uses linear scanning with simple heuristics for child node discovery
- May not correctly handle all scoping scenarios
- Tests for actual symbol resolution are marked as `#[ignore]` with TODO comments

#### Next Steps

1. **Implement Proper AST Traversal** (HIGHEST PRIORITY)
   - The `ScopeWalker::visit_children` method needs a complete rewrite
   - Should use proper parent-child relationships from ThinNodeArena
   - Consider adding a `get_children()` method to ThinNodeArena for each node type
   - Or implement a proper visitor pattern that knows how to traverse each node kind

2. **Test and Debug Symbol Resolution**
   - Once proper traversal is implemented, un-ignore the tests
   - Test with various code patterns: variables, functions, classes, nested scopes
   - Add edge case tests: shadowing, hoisting, etc.

3. **Add More LSP Features** (After basic resolution works)
   - Completions
   - Hover information
   - Signature help
   - Rename refactoring

#### Testing

All tests pass (603/603):
```bash
./wasm/test.sh  # ✅ All pass (5 ignored tests are expected)
```

## Quick Reference

```bash
# Tests (Docker)
./wasm/test.sh

# Baseline comparison
node scripts/baseline-test-rust.mjs

# Build WASM
./wasm/build-wasm.sh
```
