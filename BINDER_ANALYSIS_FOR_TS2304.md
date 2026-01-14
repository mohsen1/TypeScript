# Binder Analysis for TS2304 Error Reduction

**Date:** 2026-01-14
**Worker:** Worker 10 (EM-3: Semantics Squad)
**Task:** Reduce TS2304 "Cannot find name" errors from 343 to <50

---

## TS2304 Error Sources

TS2304 "Cannot find name 'X'" errors occur when the `resolveName` function in checker.ts fails to find a symbol. This can happen due to:

1. **Symbol not declared** - Symbol never added to symbol table by binder
2. **Wrong scope table** - Symbol declared in wrong table (locals vs members vs exports)
3. **Scope chain issue** - Symbol exists but not accessible from current location
4. **Declaration order** - Symbol used before it's declared (hoisting issues)
5. **Module/import issues** - Imported symbols not properly merged into scope
6. **Global symbol access** - Global symbols not visible in certain contexts

---

## Key Binder Functions

### `declareSymbolAndAddToSymbolTable` (line 2263)

Routes symbol declaration to the correct symbol table based on container type:
- **Modules/Source Files:** Use `exports` or `locals`
- **Classes:** Use `exports` (static) or `members` (instance)
- **Functions:** Use `locals`
- **Interfaces/Object literals:** Use `members` (not directly accessible)

**Critical insight:** If a symbol is added to the wrong table, it won't be found by `resolveName`.

### `declareSymbol` (line 748)

Creates or merges symbols in a symbol table:
- Handles symbol conflicts (e.g., var vs class with same name)
- Manages symbol flags and excludes
- Returns the symbol (or creates new one if needed)

**Potential issue:** If `name === undefined`, creates a symbol with `InternalSymbolName.Missing` which could cause downstream issues.

### Symbol Table Hierarchy

```
SourceFile
├── locals (file-level declarations)
└── symbol.exports (module exports)
    └── members (module member exports)

Class
├── locals (class-level locals)
├── members (instance members)
└── symbol.exports (static members)
```

---

## Common TS2304 Patterns

### Pattern 1: Module Import/Export Issues
```typescript
// moduleA.ts
export const foo = 1;

// moduleB.ts  
import { foo } from './moduleA';
console.log(foo); // TS2304 if import not properly bound
```

**Binder responsibility:** `bindImportClause` (line 3181) must register imported symbols properly.

### Pattern 2: Scope Visibility Issues
```typescript
function outer() {
    const x = 1;
    function inner() {
        console.log(x); // TS2304 if scope chain broken
    }
}
```

**Binder responsibility:** `bind` function must set up proper parent/child relationships.

### Pattern 3: Hoisting Issues
```typescript
const fn = function() {
    return helper(); // TS2304 if helper not hoisted
    function helper() { return 1; }
}
```

**Binder responsibility:** Functions must be bound before their usage in the same scope.

---

## Investigation Focus Areas

### 1. Import/Export Binding Priority: HIGH

**Files to check:**
- `bindImportClause` (line 3181)
- `bindExportDeclaration` (line 3164)
- `bindExportAssignment` (line 3125)

**Common issues:**
- Imported symbols not added to correct symbol table
- Export symbols not properly registered in `module.exports`
- Re-export not handling symbol forwarding correctly

### 2. Scope Chain Setup Priority: HIGH

**Files to check:**
- `bind` (main binding function)
- Container switching logic (line 466+)
- Parent pointer management

**Common issues:**
- Parent pointers not set correctly
- Container not properly restored after binding children
- Block scope vs function scope confusion

### 3. Symbol Table Merging Priority: MEDIUM

**Files to check:**
- `declareSymbol` (line 748)
- Symbol conflict resolution
- Module merging logic

**Common issues:**
- Merge conflicts not handled gracefully
- Symbol flags not properly set/checked
- Duplicate detection too strict/lenient

---

## Proposed Improvements

### Quick Wins (Low Risk)

1. **Add null checks in `declareSymbolAndAddToSymbolTable`**
   - Verify `container.locals` exists before using
   - Add defensive checks for undefined symbols

2. **Improve import binding error handling**
   - Better fallback when import resolution fails
   - Log when symbols are skipped

3. **Enhance scope debugging**
   - Add debug output for symbol table operations
   - Track symbol registration failures

### Medium-Term Improvements

1. **Refactor scope chain traversal**
   - Ensure consistent parent/child relationships
   - Verify container restoration

2. **Improve symbol table merging**
   - Better handling of module merging
   - Graceful degradation on conflicts

---

## Next Steps

1. Run conformance tests to capture actual TS2304 errors
2. Categorize errors by root cause
3. Implement targeted fixes for common patterns
4. Test and verify error reduction

**Goal:** Reduce from 343 to <50 TS2304 extra errors

---

## Notes

- This analysis is for TSC's TypeScript compiler (not Rust/WASM)
- The 343 "extra" TS2304 errors are from Rust/WASM compiler vs TSC comparison
- Improving TSC's binder helps establish the correct behavior
- Rust/WASM compiler can then match TSC's implementation

---

END OF ANALYSIS
