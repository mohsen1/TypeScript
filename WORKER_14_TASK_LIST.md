# WORKER-14 Task List

**EM:** EM-4
**Focus:** Symbol Resolution (Tier 3) - Fix TS2304 and TS2524 missing errors

## Mission

Fix symbol and module resolution - ensure "Cannot find name" (TS2304) and module member resolution (TS2524) errors are emitted when appropriate.

## Current Baseline

From conformance tests (200 files):
- **TS2304 Missing:** 7 occurrences - Symbol not found but should emit error
- **TS2304 Extra:** 5 occurrences - False positives on valid symbols
- **TS2524 Missing:** 12 occurrences - Module member resolution not checked

### What These Errors Mean

**TS2304:** "Cannot find name 'X'"

This error should be emitted when:
- A variable/function/type is used but not declared
- A symbol is not in scope
- Global symbols are not properly merged

**TS2524:** " 'X' is not a module and cannot be imported using 'import { ... } from'"

This error should be emitted when:
- Importing a member from a non-module file
- The file doesn't have appropriate exports

## Key Files

| File | Purpose |
|------|---------|
| `wasm/src/binder/` | Symbol table and scope management |
| `wasm/src/thin_binder.rs` | Main binder implementation |
| `wasm/src/thin_checker.rs` | Symbol usage checking |
| `wasm/src/parallel.rs` | Module handling |

## Steps

### 1. Verify Error Codes Exist

Check if TS2304 and TS2524 are defined:
```bash
grep -n "TS2304\|TS2524" wasm/src/checker/types/diagnostics.rs
```

### 2. Understand the Binder

The binder (`wasm/src/binder/`) builds the symbol table:
- Reads declarations
- Creates symbol entries
- Tracks scope chains

The checker then:
- Looks up symbols when used
- Emits TS2304 if not found
- Checks module imports for TS2524

### 3. Find Missing Test Cases

From the conformance output, identify specific test files:
- Look in `ambient/` and `Symbols/` categories
- Files with TS2304/TS2524 missing

### 4. Create Test Repros

```typescript
// /tmp/test2304.ts
// Should emit TS2304 for "undeclaredVar"
console.log(undeclaredVar);
```

```typescript
// /tmp/test2524.ts
// file1.ts - not a module
const x = 1;

// file2.ts - trying to import from non-module
import { x } from "./file1";  // Should emit TS2524
```

### 5. Fix Symbol Resolution

**For TS2304 Missing:**
1. Check binder is creating symbols for all declarations
2. Verify symbol lookup in checker handles all scope types
3. Check global symbol merging across files
4. Ensure undeclared symbols trigger the error

**For TS2524 Missing:**
1. Track which files are modules (have imports/exports)
2. When processing import, check if source is a module
3. Emit TS2524 if importing from non-module

**For TS2304 Extra (false positives):**
1. Verify global symbols (Promise, Array, etc.) are available
2. Check interface merging works across files
3. Ensure ambient declarations are processed

### 6. Validate

```bash
# Build
cd wasm && wasm-pack build --target web --out-dir pkg

# Test symbols/ambient categories
cd differential-test
bash run-conformance.sh --max=200 --workers=4 | grep -A10 "Symbols\|ambient"
```

## Success Criteria

- **TS2304 Missing:** Reduced from 7 to ≤2
- **TS2524 Missing:** Reduced from 12 to ≤4
- **TS2304 Extra:** Reduced from 5 to ≤2

## Resources

- Binder: `wasm/src/binder/mod.rs`, `wasm/src/thin_binder.rs`
- Symbol table: `wasm/src/binder/symbol.rs`
- Module handling: `wasm/src/parallel.rs`
- Checker symbol lookup: `wasm/src/thin_checker.rs` (search for `resolve_symbol`)

## Submit Your Work

1. Create branch: `git checkout -b worker-14-symbol-fixes`
2. Commit with clear messages
3. Run validation: `bash run-conformance.sh --max=500 --workers=8`
4. Notify EM-4 for review
