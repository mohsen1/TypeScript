
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

### 🚨 URGENT: Critical Infrastructure (MUST FIX NOW)

These issues block LSP responsiveness.

#### Parent Mapping for Navigation ✅ COMPLETED (2026-01-06)

**Problem:** `ThinNode` (16 bytes) doesn't store a `parent` pointer.

**Impact:**
- `ScopeWalker` reconstructs scopes on demand - this is good
- BUT: `find_node_at_offset` followed by "walk up" strategies (like SignatureHelp) require either:
  - Full tree traversal every time (slow)
  - A side-table `Vec<ParentIndex>` (need to verify this exists)

**Solution Implemented:**
- [x] **Verified:** `ThinNodeArena` has `extended_info: Vec<ExtendedNodeInfo>` with parent field
- [x] **Implemented:** Parent pointers are now set during node creation in all `add_*` methods
- [x] **Helper methods added:** `set_parent`, `set_parent_list`, `set_parent_opt_list` for O(1) parent linking
- [x] **Fixed Default:** `ExtendedNodeInfo::default()` now sets `parent = NodeIndex::NONE`
- [x] **Tests added:** 3 comprehensive tests verify parent mapping works correctly
- [x] **All tests pass:** 691/691 tests passing

**Implementation Details:**
- Parent pointers set incrementally during AST construction (O(N) total, O(1) per edge)
- Updated methods: `add_binary_expr`, `add_call_expr`, `add_function`, `add_block`, `add_source_file`, `add_if_statement`, `add_variable`, `add_return`, `add_unary_expr`, `add_access_expr`, `add_variable_declaration`
- LSP "walk up to parent" operations are now O(1) via `arena.get_extended(node).parent`

#### Stateless Resolution (Shared with Checker) ✅ COMPLETED (Already Working)

**Problem:** LSP features must work without full type-checking pass.

**Solution Verified:**
- [x] **LSP queries Binder directly:** ScopeWalker resolves identifiers using only Binder data
- [x] **No full type-check required:** LSP features work with just Parser + Binder output
- [x] **On-demand type computation:** ThinCheckerState created only when type info needed (hover)
- [x] **All LSP tests pass:** 41/41 tests passing without full file type-checking

**Implementation Details:**
- ScopeWalker reconstructs scope chains on demand by walking the AST
- Resolves identifiers to symbols using Binder's node_symbols and file_locals
- Checker only invoked for specific symbols when type information requested
- Supports Go-to-Definition, Find References, Hover, Completions, Signature Help

1. **Continue Parser Fixes** (MEDIUM PRIORITY)
   - Fix remaining composite nodes: binary expressions, property access, function declarations
   - Fix delimited nodes: blocks, parenthesized expressions
   - These will further improve LSP accuracy but current fixes handle most critical cases

2. **Complete Code Actions Implementation**
   - Add more refactoring actions (extract function, inline variable, etc.)
   - Integrate with diagnostics for quick fixes

### ✅ Performance Optimizations

#### Comment Range Caching ✅ COMPLETED (2026-01-06)

**Problem:** Every hover request was doing an O(N) scan of the entire source file to find comments.

**Solution Implemented:**
- [x] **Added comments field to SourceFileData:** Stores Vec<CommentRange> computed once during parsing
- [x] **Cached during parse_source_file:** Single O(N) scan when file is parsed
- [x] **Added get_leading_comments_from_cache:** O(log N) binary search to find relevant comments
- [x] **Updated hover.rs:** Uses cached comments instead of rescanning
- [x] **All tests pass:** 691/691 tests passing

**Performance Impact:**
- **Before:** O(N) scan on every hover (N = file size in characters)
- **After:** O(log C) binary search (C = number of comments, typically << N)
- **Example:** 5MB file with 1000 comments: ~5M operations → ~10 operations per hover

**Implementation Details:**
- Comments cached in SourceFileData.comments during parsing
- Binary search (partition_point) finds insertion point in O(log C)
- Only iterates backwards through adjacent comments (typically 1-3)
- Hover response time improved dramatically on large files

#### Remaining Optimizations
   - Refactor ScannerState to use &str instead of cloning source (reduce memory 2x)
   - Consider caching type information for repeated queries

3. **Add More LSP Features**
   - Code actions (complete implementation)

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
