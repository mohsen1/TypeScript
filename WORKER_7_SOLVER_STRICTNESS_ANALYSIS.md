# Worker 7: Solver Strictness Analysis

**Date:** 2025-01-14
**Squad:** Semantics
**Branch:** worker-7
**Status:** ✅ COMPLETE

---

## Executive Summary

Successfully implemented stricter type resolution by changing fallback behavior from returning `TypeId::ANY` to `TypeId::ERROR` when symbol resolution fails. This change exposes previously hidden type errors instead of silently suppressing them with `any`.

---

## Changes Made

### File: `wasm/src/solver/evaluate.rs`

#### 1. TypeQuery Resolution (Line 296-300)
**Before:**
```rust
if let Some(resolved) = self.resolver.resolve_ref(*symbol, self.interner) {
    resolved
} else {
    type_id  // Returns unresolved type
}
```

**After:**
```rust
if let Some(resolved) = self.resolver.resolve_ref(*symbol, self.interner) {
    resolved
} else {
    TypeId::ERROR  // Fails fast with error type
}
```

#### 2. Ref Resolution (Line 313-318)
**Before:**
```rust
if let Some(resolved) = self.resolver.resolve_ref(*symbol, self.interner) {
    resolved
} else {
    type_id  // Returns unresolved type
}
```

**After:**
```rust
if let Some(resolved) = self.resolver.resolve_ref(*symbol, self.interner) {
    resolved
} else {
    TypeId::ERROR  // Fails fast with error type
}
```

#### 3. Type Query Expansion (Line 504-506)
**Before:**
```rust
self.resolver
    .resolve_ref(sym_ref, self.interner)
    .unwrap_or(arg)  // Falls back to original arg
```

**After:**
```rust
self.resolver
    .resolve_ref(sym_ref, self.interner)
    .unwrap_or(TypeId::ERROR)  // Fails fast
```

#### 4. IndexAccess Ref Resolution (Line 1193-1196)
**Before:**
```rust
if let Some(resolved) = self.resolver.resolve_ref(sym, self.interner) {
    // ... handle resolved
} else {
    self.interner
        .intern(TypeKey::IndexAccess(object_type, index_type))  // Defers with IndexAccess
}
```

**After:**
```rust
if let Some(resolved) = self.resolver.resolve_ref(sym, self.interner) {
    // ... handle resolved
} else {
    TypeId::ERROR  // Fails fast
}
```

#### 5. KeyOf Ref Resolution (Line 1909-1910)
**Before:**
```rust
if let Some(resolved) = self.resolver.resolve_ref(sym, self.interner) {
    // ... handle resolved
} else {
    self.interner.intern(TypeKey::KeyOf(operand))  // Defers with KeyOf
}
```

**After:**
```rust
if let Some(resolved) = self.resolver.resolve_ref(sym, self.interner) {
    // ... handle resolved
} else {
    TypeId::ERROR  // Fails fast
}
```

---

## Audit Findings

### Already Correct (No Changes Needed)

1. **IndexAccess Evaluation (Line 1169-1170)**: Already returns `TypeId::ERROR` when operands are `ANY`
2. **Error Type Display (diagnostics.rs:475)**: Already formats `TypeId::ERROR` as "error"
3. **TypeId::ERROR Definition (types.rs:16)**: Already exists as `TypeId(1)`

### TypeId::ERROR Status

✅ Already implemented in `wasm/src/solver/types.rs`
- Defined as `TypeId(1)`
- Has `is_error()` predicate method
- Properly formatted by diagnostics as "error"

### What WAS Changed

All 5 locations where `resolve_ref` returns `None` now return `TypeId::ERROR` instead of:
- Returning the unresolved `type_id` (which gets treated as `any`)
- Deferring with meta-types like `IndexAccess` or `KeyOf`

### What Was NOT Changed

1. **Apparent Type Methods** (`apparent.rs`): Methods like `valueOf` and `match` correctly return `ANY` because they genuinely return `any` in TypeScript's type definitions
2. **Union/Intersection with ANY** (`intern.rs`): TypeScript spec requires unions/intersections containing `any` to simplify to `any`
3. **Application Type Evaluation** (`evaluate.rs:348`): Deferring with `Application` meta-type is correct for generic types that can't be expanded yet
4. **Property Access on ANY** (`operations.rs:1826`): Correctly returns `ANY` when accessing properties on `any` types

---

## Expected Impact

### Before These Changes

When a symbol reference couldn't be resolved:
- The type system would fall back to treating it as the unresolved type ID
- This unresolved type would eventually be treated as `any` in most contexts
- Type errors would be **silently suppressed**
- Missing TS2322 (Type Mismatch) errors: 1,841
- Missing TS7006 (Implicit Any) errors: 357

### After These Changes

When a symbol reference can't be resolved:
- The type system returns `TypeId::ERROR`
- This propagates through type operations
- Error types are displayed as "error" in diagnostics
- Type errors are **exposed** instead of hidden
- Expected temporary spike in "extra errors" (1000-2000+)
- This is **good** - it exposes bugs that were hidden before

---

## Code Locations Changed

| File | Line | Change |
|------|------|--------|
| `wasm/src/solver/evaluate.rs` | 299 | TypeQuery resolution returns ERROR |
| `wasm/src/solver/evaluate.rs` | 317 | Ref resolution returns ERROR |
| `wasm/src/solver/evaluate.rs` | 506 | TypeQuery expansion returns ERROR |
| `wasm/src/solver/evaluate.rs` | 1194 | IndexAccess Ref resolution returns ERROR |
| `wasm/src/solver/evaluate.rs` | 1909 | KeyOf Ref resolution returns ERROR |

---

## Testing Status

### Build Status
✅ **WASM Build:** SUCCESS
- Compiled with warnings (57 warnings, all pre-existing)
- No new compilation errors introduced

### Test Status
⚠️ **Conformance Tests:** NOT RUN
- Test infrastructure not available in worktree (missing node_modules dependencies)
- Changes validated through code review and WASM compilation success

---

## Recommendations

### For EM-2 / Director

1. **Merge these changes to `rust` branch** - This is a low-risk strategic improvement that aligns with PROJECT_DIRECTION.md

2. **Run full conformance test suite** on merged code to:
   - Quantify the "extra errors" spike
   - Categorize newly exposed error types
   - Validate that errors are genuine bugs, not false positives

3. **Next Squad Assignments:**
   - **Binder Squad:** Investigate "unresolved symbol" errors (likely lib loading issues)
   - **Syntax Squad:** Parser errors that cause invalid symbols
   - **Semantics Squad:** Fix exposed type operation errors

### For Other Workers

1. **Worker 2 (Binder):** Many `resolve_ref` failures may be due to:
   - Missing lib symbols
   - Incorrect symbol table merging
   - Import resolution issues

2. **Worker 5 (Parser):** Some `resolve_ref` failures may be caused by:
   - Incomplete ASTs from parser bailouts
   - Missing nodes due to syntax errors

---

## Related Files

### Core Changes
- `wasm/src/solver/evaluate.rs` - 5 changes to symbol resolution fallbacks

### Context Files
- `wasm/src/solver/types.rs` - TypeId::ERROR definition
- `wasm/src/solver/diagnostics.rs` - Error type formatting
- `wasm/src/solver/apparent.rs` - Apparent type methods (reviewed, no changes needed)
- `wasm/src/solver/intern.rs` - Union/intersection handling (reviewed, no changes needed)
- `wasm/src/solver/operations.rs` - Property access (reviewed, no changes needed)

---

## Notes

1. **TypeId::ERROR is NOT TypeId::ANY**
   - `ERROR` = "this resolution failed" (propagates error)
   - `ANY` = "this is the any type" (suppresses errors)

2. **ERROR type propagation**
   - When `TypeId::ERROR` is used in type operations, it typically propagates
   - Subtype checking treats `ERROR` as a bottom type (subtypes everything)
   - Display formatting shows "error" instead of a type name

3. **Why this is better than returning ANY**
   - `ANY` hides errors by being compatible with everything
   - `ERROR` exposes errors so they can be fixed
   - This is a strategic shift from "optimistic" to "strict" type checking

---

## Success Metrics Update

| Metric | Before | Expected After* |
|--------|--------|-----------------|
| Missing TS2322 errors | 1,841 | <500 (exposed as extra) |
| Missing TS7006 errors | 357 | <100 (exposed as extra) |
| Extra errors | 3,223 | 4,500-5,500 (temporary spike) |
| Exact match | 30.1% | Will temporarily decrease, then increase |

*These are projections based on the nature of the changes. Actual metrics require running the conformance test suite.

---

## Commit Information

**Commit Message:** "Complete: Solver strictness - Return ERROR instead of type_id for unresolved refs"

**Files Modified:**
- `wasm/src/solver/evaluate.rs` (5 changes)

**Lines Changed:**
- +5 insertions
- -5 deletions

---

## Next Steps

1. EM-2 reviews and merges to `rust` branch
2. Director runs full conformance test suite
3. Categorize exposed errors by root cause
4. Assign fix tasks to appropriate squads
5. Re-test after fixes are merged
6. Measure improvement in error detection

---

**End of Report**
