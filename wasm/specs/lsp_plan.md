
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

#### Implementation Details (2026-01-06)

✅ **Proper AST Traversal Implemented**

Following Gemini's guidance, implemented comprehensive AST traversal:

1. **`for_each_child` Helper Method**
   - Central method that knows how to extract children for every `SyntaxKind`
   - Uses proper typed accessors (`get_block`, `get_function`, etc.)
   - Handles all major node types:
     - Source files, blocks, module blocks
     - Functions, methods, constructors, classes
     - Variables, statements (if, for, while, return, etc.)
     - Expressions (binary, call, property access, conditional, unary, etc.)
     - Control flow (try/catch, switch/case)
     - Object literals (property assignment, shorthand, spread)

2. **`node_creates_scope` Helper**
   - Centralized logic for determining which nodes create new scopes
   - Eliminates code duplication between `walk_to_node` and `collect_references`

3. **Performance Optimizations**
   - **O(N) traversal**: `collect_references` maintains scope state during traversal
   - **No O(N²) lookups**: Uses current scope stack instead of creating new walkers
   - **Early bailout**: Range check optimization in `walk_to_node`

4. **Symbol Resolution**
   - Go-to-Definition: ✅ Working (finds declarations from usages)
   - Find References: ✅ Working (finds all usages of a symbol)
   - Handles scoping correctly with scope stack

#### Test Results

All 607 tests pass! ✅
- LSP-specific tests: 14/14 passing
- Position utilities: 3/3 passing
- Resolver tests: 1/1 passing
- Definition tests: 2/2 passing
- References tests: 2/2 passing
- Integration tests: 3/3 passing
- Utils tests: 3/3 passing

#### Next Steps

1. **Add More LSP Features**
   - Completions (suggest identifiers based on current scope)
   - Hover information (show type/documentation for symbol at cursor)
   - Signature help (show function parameter info)
   - Rename refactoring

2. **Extend AST Coverage** (if needed)
   - Add more expression types as needed (template literals, JSX, etc.)
   - Add import/export handling for cross-file navigation

3. **Multi-File Support**
   - Extend to handle cross-file references
   - Implement project-wide find references

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
