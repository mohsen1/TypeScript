
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

#### Parent Mapping for Navigation

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
