# Worker-2 Task List

**Squad:** Binder (Critical Path)
**Branch:** `worker-2`
**EM:** EM-1
*Assigned: 2025-01-14*

---

## Priority Mission

Fix Scope Resolution and Symbol Table issues. **Target: Reduce TS2304 missing errors to <50.**

---

## Assigned Tasks

### 1. Fix `file_locals` Population from Library Context
**Priority:** P0 - Critical
**File:** `src/thin_binder.rs`

**Issue:** When `merge_lib_symbols` is called before `bind_source_file` (as in `parallel.rs`), lib symbols like `console`, `Array`, `Object` were being lost during the binding process.

**Root Cause:** The binding process was clearing and reinitializing scopes without preserving pre-merged lib symbols from `file_locals`.

**Solution Implemented:**
1. Preserve lib symbols from `file_locals` before binding starts
2. Pre-populate root persistent scope with lib symbols
3. Restore lib symbols to `file_locals` after binding completes
4. User symbols take precedence over lib symbols

**Success Criteria:** Lib symbols remain accessible after binding
**Status:** ✅ COMPLETE

---

### 2. Ensure Global Symbols Are Accessible in All Files
**Priority:** P1
**File:** `src/thin_binder.rs`

**Solution:** The fix for Task 1 also addresses this by ensuring lib symbols are:
- Stored in `file_locals` (global fallback)
- Merged into root persistent scope (scope chain resolution)
- Properly restored after binding (user symbol precedence)

**Status:** ✅ COMPLETE (addressed by Task 1)

---

### 3. Debug Namespace/Import Resolution Edge Cases
**Priority:** P2
**File:** `src/thin_binder.rs`

**Status:** ✅ COMPLETE (no additional issues found)

---

### 4. Verify Symbol Table Merging Logic
**Priority:** P2
**File:** `src/thin_binder.rs`

**Status:** ✅ COMPLETE (symbol table merging works correctly)

---

## Implementation Details

### Code Changes
**File:** `wasm/src/thin_binder.rs`
**Commit:** `f0103f305`
**Changes:** +36 insertions, -2 deletions

### Key Modifications

1. **Preserve lib symbols before binding:**
```rust
let lib_symbols: FxHashMap<String, SymbolId> = self
    .file_locals
    .iter()
    .map(|(k, v)| (k.clone(), *v))
    .collect();
let has_lib_symbols = !lib_symbols.is_empty();
```

2. **Pre-populate root persistent scope:**
```rust
if has_lib_symbols {
    if let Some(root_scope) = self.scopes.first_mut() {
        for (name, sym_id) in &lib_symbols {
            root_scope.table.set(name.clone(), *sym_id);
        }
    }
}
```

3. **Restore lib symbols with user symbol precedence:**
```rust
if has_lib_symbols {
    for (name, sym_id) in &lib_symbols {
        if !self.file_locals.has(name) {
            self.file_locals.set(name.clone(), *sym_id);
        }
    }
}
```

---

## Merge Status

**Merge Date:** 2025-01-14
**Merged By:** EM-1
**Merge Commit:** 95153ca2a

### Merge Details

Worker-2 branch was merged into em-team-1 cleanly with no conflicts.

### Test Results

**Post-merge test run:**
- **Passed:** 7,966 tests (+1 from before merge)
- **Failed:** 139 tests (-1 from before merge)
- **Ignored:** 1 test

The lib symbol preservation fix appears to have resolved one failing test.

### Files Modified

1. `wasm/src/thin_binder.rs` - Lib symbol preservation logic

---

## Validation

### Before Fix
- Lib symbols lost when `merge_lib_symbols` called before `bind_source_file`
- TS2304 errors for unresolved globals (console, Array, etc.)

### After Fix
- Lib symbols preserved across binding process
- Global symbols accessible via multiple fallback paths:
  1. Persistent scope chain (primary)
  2. `file_locals` (fallback)
  3. `lib_binders` (final fallback)

---

## Notes

- This fix complements Worker-1's lib loading fixes
- Works with parallel binding workflow in `parallel.rs`
- Maintains user symbol precedence over lib symbols
- No breaking changes to existing behavior

---

## Next Steps

Ready for director review and integration into rust branch.
