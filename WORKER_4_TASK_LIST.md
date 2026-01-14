# Worker 4 Task List

## Squad: Binder (CRITICAL)

## Current Task
- [x] Investigate lib.d.ts loading and symbol merging into root SymbolTable in wasm/src/binder

## Queue
- [ ] Trace how lib.d.ts symbols (Array, Promise, console) are supposed to be injected
- [ ] Fix symbol table merging to correctly expose global types
- [ ] Add tests verifying lib.d.ts globals are resolvable

## Completed
- [x] Investigate lib.d.ts loading and symbol merging into root SymbolTable

## Context
TS2304 (Cannot find name) is the #1 source of error poisoning. When the Binder fails to resolve Promise, the Solver defaults to Any, suppressing all downstream errors.

---

## Investigation Findings: lib.d.ts Loading and Symbol Merging

### Overview

The lib.d.ts loading mechanism is implemented but has a **critical gap**: `get_symbol()` vs `get_symbol_with_libs()` usage inconsistency in the checker.

### Architecture

**Files involved:**
- `wasm/src/lib_loader.rs` - Loads and parses lib.d.ts files
- `wasm/src/thin_binder.rs` - Manages symbol tables and scope resolution
- `wasm/src/lib.rs` - WASM interface, orchestrates lib loading
- `wasm/src/thin_checker.rs` - Type checker that uses symbols

### Loading Flow

1. **lib.rs:257-272** - `add_lib_file()`:
   - Parses lib.d.ts with `ThinParserState`
   - Binds with `ThinBinderState`
   - Creates `Arc<LibFile>` and stores in `lib_files`

2. **lib.rs:321-336** - During binding:
   - `bind_source_file_with_libs()` - binds source and merges lib symbols
   - `inject_lib_symbols()` - injects into `symbol_arenas` mapping

3. **thin_binder.rs:501-531** - `merge_lib_symbols()`:
   - Merges lib symbols into `file_locals` (global scope)
   - Merges into `current_scope` if at root level
   - Merges into root persistent scope (`scopes[0]`)
   - Stores arena mappings in `symbol_arenas` for cross-file lookup

4. **thin_binder.rs:244-279** - `resolve_identifier()`:
   - Walks scope chain looking for symbol
   - Falls back to `file_locals.get(name)` for globals

### Critical Gap Identified

**Problem:** After resolving a symbol name to SymbolId, the checker uses `get_symbol()` which only checks the local arena, NOT lib arenas.

**Root cause:** `thin_checker.rs` has **84+ usages** of `get_symbol()` that should use `get_symbol_with_libs()`.

**Example problematic pattern:**
```rust
// thin_checker.rs:906 - Fails for lib symbols!
let Some(symbol) = self.ctx.binder.get_symbol(sym_id) else {
    return true;
};

// SHOULD BE:
let Some(symbol) = self.ctx.binder.get_symbol_with_libs(sym_id, &lib_binders) else {
    return true;
};
```

**Functions with correct usage (resolve_name_from_scope):**
- Line 318: `get_symbol_with_libs(sym_id, &lib_binders)`
- Line 341: `get_symbol_with_libs(container_sym_id, &lib_binders)`
- Line 346: `get_symbol_with_libs(member_id, &lib_binders)`
- Line 385: `get_symbol_with_libs(sym_id, &lib_binders)`

**Functions with incorrect usage (84+ occurrences):**
- Lines 906, 920, 1018, 1049, 1094, 1233, 1333, 1368, 1438...
- All use `get_symbol(sym_id)` instead of `get_symbol_with_libs()`

### Why This Causes TS2304

1. User code references `Promise`
2. `resolve_identifier()` correctly finds SymbolId from `file_locals`
3. Later, `get_type_of_symbol()` calls `get_symbol(sym_id)`
4. `get_symbol()` only checks local arena, returns `None`
5. Checker falls back to `TypeId::ANY`, suppressing errors
6. Or reports TS2304 "Cannot find name 'Promise'"

### Solution Options

**Option A: Fix all 84+ call sites** (tedious, error-prone)
- Replace `get_symbol(sym_id)` with `get_symbol_with_libs(sym_id, &lib_binders)`
- Need to pass lib_binders through many function calls

**Option B: Make get_symbol() check lib arenas automatically**
- Modify `ThinBinderState::get_symbol()` to check `symbol_arenas` if not found locally
- Requires storing lib binders reference in ThinBinderState

**Option C: Hybrid - unified symbol arena**
- During `merge_lib_symbols()`, copy actual Symbol objects into local arena
- Pros: All lookups work automatically
- Cons: Memory duplication, potential sync issues

### Recommendations

1. **Short-term:** Option B - modify `get_symbol()` to check lib binders
2. **Medium-term:** Consider Option C for cleaner architecture
3. **Testing:** Add test that:
   - Loads lib.d.ts
   - Binds `const x: Promise<number> = Promise.resolve(1);`
   - Verifies no TS2304 error for Promise

### Files to Modify

1. `wasm/src/thin_binder.rs` - Store lib binders reference, update `get_symbol()`
2. `wasm/src/thin_checker.rs` - May need updates depending on approach
3. Add test in `wasm/src/thin_checker_tests.rs` or `wasm/src/lib_loader.rs`

### Test Verification

```rust
#[test]
fn test_lib_symbol_resolution_in_checker() {
    // Load lib.d.ts
    let lib_file = load_default_lib_dts().expect("lib.d.ts should load");

    // Parse user code
    let source = "const p: Promise<number> = Promise.resolve(1);";
    let mut parser = ThinParserState::new("test.ts".to_string(), source.to_string());
    let root = parser.parse_source_file();

    // Bind with lib
    let mut binder = ThinBinderState::new();
    binder.bind_source_file_with_libs(parser.get_arena(), root, &[lib_file]);

    // Check - should NOT produce TS2304 for Promise
    let mut checker = ThinChecker::new(...);
    checker.check_source_file(root);

    // Verify no "Cannot find name 'Promise'" errors
    assert!(!checker.diagnostics.iter().any(|d| d.code == 2304));
}
```
