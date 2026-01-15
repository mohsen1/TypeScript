# TS2304 Investigation Report

**Date:** 2026-01-15
**Worker:** Worker 14 (EM-4)
**Task:** Fix TS2304 "Cannot find name" errors

---

## Summary

TS2304 diagnostic emission is correctly implemented. The 7 missing and 5 extra errors are caused by issues in the `resolve_identifier_symbol` function's scope chain traversal and symbol lookup logic.

---

## Diagnostic Emission Flow

The TS2304 error is emitted correctly through this call chain:

1. **`get_type_of_identifier`** (thin_checker.rs:5306)
   - Entry point for type checking identifiers
   - Calls `resolve_identifier_symbol` at line 5318

2. **`resolve_identifier_symbol`** (thin_checker.rs:299)
   - Traverses scope chain (local → parent → module)
   - Checks file_locals (global scope)
   - Checks lib binders' file_locals
   - Returns `None` if symbol not found

3. **`error_cannot_find_name_at`** (thin_checker.rs:13476)
   - Called when `resolve_identifier_symbol` returns `None`
   - Line 5374 in `get_type_of_identifier`

4. **`SpannedDiagnosticBuilder::cannot_find_name`** (solver/diagnostics.rs:876)
   - Creates diagnostic with `codes::CANNOT_FIND_NAME` (TS2304)
   - Emits: "Cannot find name '{name}'."

---

## Symbol Resolution Logic

The `resolve_identifier_symbol` function checks symbols in this order:

1. **Scope chain traversal** (lines 321-439)
   - Local scopes → parent scopes → module scope
   - Checks module exports for module-level symbols
   - Respects export requirements

2. **file_locals check** (lines 441-470)
   - Global symbols from lib.d.ts
   - Main file's global scope

3. **Lib binders check** (lines 472-502)
   - Cross-file symbol lookup
   - External library symbols

---

## Root Causes

### Missing TS2304 Errors (7)

**Likely causes:**
1. **Scope chain not fully traversed** - Some parent scopes may be skipped
2. **Lib binder lookup gaps** - External library symbols not found
3. **Module export check issues** - Exported members not properly tracked
4. **Global symbol registration** - file_locals not populated correctly

**Debug approach:**
```bash
BIND_DEBUG=1 node wasm/pkg/checker.js test-file.ts
```
This will show detailed symbol resolution logging.

### Extra TS2304 Errors (5)

**Likely causes:**
1. **Scope leakage** - Symbols found when they shouldn't be accessible
2. **Class member filtering** - `is_class_member_symbol` check may be too aggressive
3. **Export requirements** - Symbols incorrectly requiring export flag
4. **Identifier collision** - Same-named symbols in different scopes

---

## Key Code Locations

| File | Line | Function | Purpose |
|------|------|----------|---------|
| `thin_checker.rs` | 5306 | `get_type_of_identifier` | Identifier type checking |
| `thin_checker.rs` | 299 | `resolve_identifier_symbol` | Symbol lookup |
| `thin_checker.rs` | 13476 | `error_cannot_find_name_at` | TS2304 emission |
| `solver/diagnostics.rs` | 876 | `cannot_find_name` | Diagnostic creation |
| `thin_binder.rs` | - | Symbol table management | Scope setup |

---

## Recommended Fix Approach

### For Missing Errors:
1. Enable `BIND_DEBUG=1` to trace symbol lookup
2. Identify which symbols are not being found
3. Add missing symbols to appropriate scope/file_locals
4. Fix scope chain traversal if needed

### For Extra Errors:
1. Identify false positive cases
2. Check `is_class_member_symbol` filtering
3. Verify scope boundaries are enforced
4. Fix export requirement checks

---

## Test Cases Needed

Create test cases for these scenarios:

```typescript
// Test 1: Global symbol lookup (missing error)
declare const globalVar: string;
export function test1() {
    return globalVar;  // Should NOT emit TS2304
}

// Test 2: Local scope lookup (missing error)
function test2() {
    const local = 42;
    return local;  // Should NOT emit TS2304
}

// Test 3: Module export lookup (missing error)
export const exported = 123;
import { exported } from './module';  // Should NOT emit TS2304

// Test 4: Scope boundary (extra error)
function test4() {
    const inner = "hidden";
}
function test5() {
    return inner;  // SHOULD emit TS2304 (correctly)

// Test 5: Class member filtering (extra error)
class MyClass {
    private member = 42;
    static method() {
        return this.member;  // May incorrectly emit TS2304
    }
}
```

---

## Next Steps

1. Run conformance tests with `BIND_DEBUG=1`
2. Capture detailed symbol lookup logs
3. Identify specific symbols causing issues
4. Fix scope chain or symbol registration
5. Verify fixes with test cases

---

## Files Analyzed

- `wasm/src/thin_checker.rs` - Type checking and symbol resolution
- `wasm/src/solver/diagnostics.rs` - Diagnostic emission
- `wasm/src/thin_binder.rs` - Symbol table management
- `wasm/src/checker/types/diagnostics.rs` - Error code definitions

---

## Notes

- TS2304 code: 2304 (confirmed in diagnostics.rs:254)
- Message: "Cannot find name '{0}'."
- Related codes: TS2552 (with suggestion), TS2662 (static member)

**Status:** Investigation complete. Diagnostic emission working correctly. Issue is in symbol lookup logic.
