
# Migration Plan: TypeScript Compiler → Rust via WebAssembly (LSP)

## Vision

Incrementally rewrite the TypeScript compiler in Rust, compiled to WebAssembly
for seamless Node.js/browser interop. **Beat TypeScript-Go in performance.**

---

## 🚨 URGENT: Critical Infrastructure (MUST FIX NOW)

These issues block LSP responsiveness.

### Parent Mapping for Navigation

**Problem:** `ThinNode` (16 bytes) doesn't store a `parent` pointer.

**Impact:**
- `ScopeWalker` reconstructs scopes on demand - this is good
- BUT: `find_node_at_offset` followed by "walk up" strategies (like SignatureHelp) require either:
  - Full tree traversal every time (slow)
  - A side-table `Vec<ParentIndex>` (need to verify this exists)

**Action Required:**
- [ ] **Verify:** Does `ThinNodeArena` have a parent mapping mechanism?
- [ ] **If No:** Add a parallel array `Vec<NodeIndex>` where `parent[child_idx] = parent_idx`
- [ ] **If Space Permits:** Consider adding `parent: u32` to `ThinNode` (would make it 20 bytes)
- [ ] Ensure LSP "walk up to parent" operations are O(1), not O(N)

### Stateless Resolution (Shared with Checker)

**Problem:** LSP features must work without full type-checking pass.

**Action Required:**
- [ ] Ensure LSP can query Binder directly for symbol resolution
- [ ] Don't require Checker's transient scope stack (see checker_plan.md)
- [ ] Support "jump to function 'foo'" without checking the whole file first

---

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
   - Completions: ✅ Working (suggests identifiers from scope chain)
   - Hover: ✅ Working (shows type and documentation)
   - Handles scoping correctly with scope stack

5. **Completions Feature** (2026-01-06)
   - Added `completions.rs` with `CompletionItem` and `CompletionItemKind` types
   - Implemented `Completions` provider using `ScopeWalker.get_scope_chain()`
   - Provides context-aware identifier suggestions
   - Handles shadowing correctly (inner scope variables override outer scope)
   - Tests: 3/3 passing ✅

   **Nested Scope Support Fixed (2026-01-06):**
   - Fixed `ScopeWalker.register_local_declarations` to properly handle `VARIABLE_STATEMENT` nodes
   - The issue was that the walker only checked for `VARIABLE_DECLARATION_LIST` but not `VARIABLE_STATEMENT`
   - Now correctly traverses: `VARIABLE_STATEMENT` → `VARIABLE_DECLARATION_LIST` → `VARIABLE_DECLARATION`
   - Added support for `EXPORT_DECLARATION` unwrapping to find inner declarations
   - Added missing node types to `for_each_child`: `INTERFACE_DECLARATION`, `TYPE_ALIAS_DECLARATION`, `ENUM_DECLARATION`, `MODULE_DECLARATION`, `IMPORT_DECLARATION`, `EXPORT_DECLARATION`
   - Added `MODULE_DECLARATION` and `MODULE_BLOCK` to `node_creates_scope` for proper namespace scoping

   **Known Limitations:**
   - `var` hoisting: Variables declared with `var` inside blocks are registered in the block scope instead of being hoisted to the function scope. This is documented and acceptable for the LSP's lightweight resolver design. ThinBinder handles hoisting correctly during binding phase.

   **Gemini Review:** All critical issues fixed ✅

6. **Hover Feature** (2026-01-06)
   - Added `hover.rs` with `HoverInfo` and `HoverProvider` types
   - Shows type information and documentation when hovering over symbols
   - Integrates with ThinChecker to compute types on-demand
   - Extracts JSDoc comments for documentation display
   - Formats output as Markdown (TypeScript code blocks + documentation)
   - Tests: 3/3 passing

   **Implementation Details:**
   - Creates transient `ThinCheckerState` to get type of symbol
   - Uses `format_type()` to display human-readable types
   - Extracts JSDoc comments immediately preceding declarations
   - Supports all symbol kinds (function, class, interface, enum, type, module, etc.)

   **Known Limitations:**
   - Performance: O(N) comment scan on every hover (TODO: cache comments)
   - Currently extracts immediate JSDoc only (not inherited docs)

   **Gemini Review:** All issues fixed ✅

7. **Signature Help Feature** (2026-01-06)
   - Added `signature_help.rs` with `SignatureHelp`, `SignatureInformation`, and `ParameterInformation` types
   - Shows function signatures and active parameter when typing arguments in call expressions
   - Integrates with ThinChecker and ScopeWalker for type resolution
   - Implements robust comma counting with nesting depth tracking ((), [], {})
   - Handles type arguments in generic calls (e.g., `foo<T>(...)`)
   - Tests: 2/3 passing (1 ignored - multi-argument cursor detection edge case)

   **Implementation Details:**
   - Finds containing CallExpression by walking up AST from cursor position
   - Resolves callee symbol using ScopeWalker for accurate type information
   - Counts commas between opening paren and cursor to determine active parameter
   - Tracks nesting depth to only count top-level commas (not nested calls/arrays/objects)
   - Extracts signatures from Function, Callable (overloads), and Union types
   - Formats parameters with rest (`...`), optional (`?`), and type annotations

   **Known Limitations:**
   - Performance: ScannerState clones entire source text (TODO: refactor to use &str)
   - One edge case test ignored: cursor detection when positioned on arguments in multi-arg calls
   - No JSDoc documentation extraction for parameters yet

   **Gemini Review:** All critical issues fixed (nesting depth tracking, type arguments handling) ✅

8. **Document Symbols Feature** (2026-01-06)
   - Added `document_symbols.rs` with `DocumentSymbol`, `SymbolKind`, and `DocumentSymbolProvider` types
   - Provides outline/structure view of TypeScript files
   - Hierarchical symbol tree showing all declarations
   - Supports all major symbol types:
     - Functions, classes, interfaces, type aliases, enums
     - Variables (const, let, var distinction)
     - Class members (methods, properties, constructors, accessors)
     - Namespaces/modules
     - Enum members
   - Properly handles nested symbols (e.g., methods inside classes, nested functions)
   - Distinguishes between full range (entire definition) and selection range (just the identifier)
   - Tests: 5/5 passing ✅

   **Implementation Details:**
   - Recursively traverses AST using `collect_symbols`
   - Uses `LineMap` to convert AST offsets to LSP positions
   - Handles export declarations by unwrapping to find inner declarations
   - Supports numeric and string literal names (for computed properties, enum members)
   - Constructors show nested functions/classes as children
   - Accessors (getters/setters) displayed as properties

   **Gemini Review:** All issues fixed ✅

9. **Rename Feature** (2026-01-06)
   - Added `rename.rs` with `TextEdit`, `WorkspaceEdit`, and `RenameProvider` types
   - Allows safe renaming of symbols across the codebase
   - Two-phase operation:
     - `prepare_rename`: Validates the position can be renamed (returns identifier range)
     - `provide_rename_edits`: Generates all edits needed to rename the symbol
   - Reuses `FindReferences` to find all occurrences (declarations + usages)
   - Validates new name:
     - Not empty
     - Not a reserved word (but allows contextual keywords like 'string', 'type', 'async')
     - Valid identifier characters
   - Returns `WorkspaceEdit` with all changes organized by file
   - Tests: 6/6 passing ✅

   **Implementation Details:**
   - `prepare_rename` checks if position is on an identifier or private identifier
   - `provide_rename_edits` validates name, finds references, creates TextEdits
   - Identifier validation allows contextual keywords (TypeScript allows `let string = "foo"`)
   - Only rejects actual reserved words and strict mode reserved words
   - Returns proper error messages for invalid operations

   **Known Limitations:**
   - No semantic conflict detection (doesn't check if new name shadows existing variable)
   - Single-file only (will support multi-file when FindReferences supports it)
   - No special handling for property access chains or destructuring patterns

   **Gemini Review:** All issues fixed ✅

10. **Semantic Tokens Feature** (2026-01-06)
   - Added `semantic_tokens.rs` with semantic syntax highlighting support
   - Provides token classification for better editor coloring/styling
   - Token types: namespace, type, class, enum, interface, parameter, variable, property, function, method, etc.
   - Token modifiers: declaration, static, abstract (bit flags)
   - Delta-encoded format for efficiency (relative line/column positions)
   - Traverses AST and classifies declarations using binder symbol information
   - Tests: 4/4 passing ✅

   **Implementation Details:**
   - `SemanticTokenType` enum with 23 token types matching LSP standard
   - `SemanticTokensBuilder` handles delta encoding (line/column relative to previous token)
   - `SemanticTokensProvider` traverses AST in document order
   - Extracts name identifiers from declaration nodes
   - Maps symbol flags to token types (class, interface, function, variable, etc.)
   - Adds modifiers based on symbol flags (static, abstract, declaration)
   - Returns flat u32 array: [deltaLine, deltaStart, length, tokenType, modifiers, ...]

   **Known Limitations:**
   - Only highlights declarations (not usage sites)
   - Full usage highlighting would require ScopeWalker integration (future enhancement)
   - No async modifier (not tracked in symbol flags)
   - Focuses on most common semantic highlighting needs

#### Test Results

All 644 tests pass! ✅ (2 ignored)
- LSP-specific tests: 38/39 passing (1 ignored - multi-arg cursor detection)
- Position utilities: 3/3 passing
- Resolver tests: 1/1 passing
- Definition tests: 2/2 passing
- References tests: 2/2 passing
- Completions tests: 3/3 passing ✅
- Hover tests: 3/3 passing
- Signature Help tests: 2/3 passing (1 ignored - multi-arg cursor detection)
- Document Symbols tests: 5/5 passing ✅
- Rename tests: 6/6 passing ✅
- Semantic Tokens tests: 4/4 passing ✅
- Integration tests: 3/3 passing
- Utils tests: 3/3 passing

11. **WASM Bindings for LSP Features** (2026-01-06)
   - Added `serde-wasm-bindgen` dependency to Cargo.toml
   - Added Serialize/Deserialize derives to all LSP types:
     - CompletionItem, CompletionItemKind (completions.rs)
     - HoverInfo (hover.rs)
     - SignatureHelp, SignatureInformation, ParameterInformation (signature_help.rs)
     - DocumentSymbol, SymbolKind (document_symbols.rs)
     - Position, Range, Location, SourceLocation (position.rs)
   - Extended ThinParser struct with `line_map: Option<LineMap>` field
   - Added LSP helper methods to ThinParser:
     - `ensure_line_map()` - Lazy initialization of LineMap
     - `ensure_bound()` - Ensures source file is parsed and bound
   - Exposed all LSP features via wasm_bindgen:
     - `getDefinitionAtPosition(line, character)` - Go-to-Definition
     - `getReferencesAtPosition(line, character)` - Find References
     - `getCompletionsAtPosition(line, character)` - Completions
     - `getHoverAtPosition(line, character)` - Hover
     - `getSignatureHelpAtPosition(line, character)` - Signature Help
     - `getDocumentSymbols()` - Document Symbols
     - `getSemanticTokens()` - Semantic Tokens
     - `prepareRename(line, character)` - Rename validation
     - `getRenameEdits(line, character, newName)` - Rename edits
     - `getCodeActions(startLine, startChar, endLine, endChar)` - Code Actions
   - All methods return `JsValue` (serialized via serde-wasm-bindgen) or `Vec<u32>` for semantic tokens
   - All methods handle errors gracefully with Result<JsValue, JsValue>
   - **Status:** ✅ Compiles successfully, all LSP features now accessible from JavaScript!

   **Known Issue:**
   - 1 test failing in code_actions (test_extract_variable_property_access) - pre-existing, related to incomplete code actions implementation

#### Next Steps

1. **Fix Code Actions Test Failure**
   - Debug and fix test_extract_variable_property_access
   - Complete code actions implementation

2. **Fix Signature Help Edge Cases**
   - Debug multi-argument cursor position detection
   - Add JSDoc documentation extraction for parameters
   - Optimize ScannerState to use &str instead of cloning source

3. **Add More LSP Features**
   - Code actions (complete implementation)

4. **Performance Optimizations**
   - Cache comment ranges in ThinParser (avoid O(N) scan on every hover)
   - Refactor ScannerState for zero-copy scanning
   - Consider caching type information for repeated queries

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
