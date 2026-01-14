# Worker-1 Task List

**Squad:** Binder (Critical Path)
**Branch:** `worker-1`
**EM:** EM-1
*Assigned: 2025-01-14*

---

## Priority Mission

Fix Global Scope and Lib Injection. **Target: Reduce TS2304 extra errors from 343 to <50.**

TS2304 ("Cannot find name") is the #1 source of "error poisoning." When the binder fails to resolve `console`, `Promise`, or `Array`, the solver defaults to `Any`, silencing all downstream errors.

---

## Assigned Tasks

### 1. Debug `console.log` Resolution Failure
**Priority:** P0 - Blocks most tests
**Files:** `src/lib_loader.rs`, `src/thin_binder.rs`

Investigation:
- Add logging to trace `console` symbol lookup
- Verify `lib.dom.d.ts` is loaded and parsed
- Check if DOM symbols are merged into root `SymbolTable`
- Confirm `console` is accessible from file scope

**Success Criteria:** `console.log()` resolves without TS2304

---

### 2. Fix `lib.d.ts` Symbol Merging
**Priority:** P0
**File:** `src/lib_loader.rs`

Current Issue: Library symbols may not be properly merged into the global scope.

Tasks:
- Verify `merge_lib_symbols()` is called after library parsing
- Ensure symbols from `lib.d.ts` and `lib.dom.d.ts` are in root table
- Check for namespace collisions or shadowing

**Success Criteria:** All `Promise`, `Array`, `Object` globals resolve

---

### 3. Fix Module Augmentation Resolution
**Priority:** P1
**File:** `src/thin_binder.rs`

TypeScript allows merging `interface Window` across files. We may not be handling this.

Tasks:
- Track augmentations across file boundaries
- Merge interface declarations with same name
- Ensure augmented symbols are visible in all files

**Success Criteria:** `interface Window { alert(): void }` in one file is accessible in another

---

### 4. Verify Basic Globals Resolution
**Priority:** P1
**Files:** All binder-related

Test that these globals always resolve:
- `console`
- `Array`
- `Object`
- `Promise`
- `Error`
- `Map`
- `Set`

**Success Criteria:** Zero TS2304 errors for built-in globals

---

## Validation

Run conformance tests after each fix:
```bash
npm run test:conformance
```

Check TS2304 counts:
```bash
grep "TS2304" conformance_test_output.txt | wc -l
```

**Target:** <50 extra TS2304 errors (down from 343)

---

## Notes

- Do NOT modify parser or solver code
- Focus ONLY on binding and symbol resolution
- Coordinate with Worker-2 (Binder Squad) to avoid conflicts
- Tag EM-1 when ready for merge
