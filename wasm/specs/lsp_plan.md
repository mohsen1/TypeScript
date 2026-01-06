
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

#### Stateless Resolution (Shared with Checker)

**Problem:** LSP features must work without full type-checking pass.

**Action Required:**
- [ ] Ensure LSP can query Binder directly for symbol resolution
- [ ] Don't require Checker's transient scope stack (see checker_plan.md)
- [ ] Support "jump to function 'foo'" without checking the whole file first

1. **Continue Parser Fixes** (MEDIUM PRIORITY)
   - Fix remaining composite nodes: binary expressions, property access, function declarations
   - Fix delimited nodes: blocks, parenthesized expressions
   - These will further improve LSP accuracy but current fixes handle most critical cases

2. **Complete Code Actions Implementation**
   - Add more refactoring actions (extract function, inline variable, etc.)
   - Integrate with diagnostics for quick fixes

3. **Performance Optimizations**
   - Cache comment ranges in ThinParser (avoid O(N) scan on every hover)
   - Refactor ScannerState to use &str instead of cloning source

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
