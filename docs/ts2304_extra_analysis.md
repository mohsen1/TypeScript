# TS2304 Extra Errors Analysis

**Author:** Worker 7 (Binder Squad)
**Date:** 2025-01-14
**Task:** Investigate TS2304 extra errors - find patterns in false positives

---

## Executive Summary

Analysis of 3,000 conformance tests reveals **531 extra TS2304 errors** (false positives where WASM reports "Cannot find name" but TypeScript does not). These errors fall into distinct categories with clear root causes.

---

## Findings Summary

| Category | Count | Percentage | Root Cause |
|----------|-------|------------|------------|
| `local_reference` | 443 | 83.4% | Parser/binder issues |
| `type_parameter` | 30 | 5.6% | Type params not bound |
| `builtin_type` | 27 | 5.1% | Missing lib.d.ts types |
| `user_defined_type` | 21 | 4.0% | Incomplete AST |
| `global_object` | 4 | 0.8% | Missing `globalThis` |
| `global_constant` | 4 | 0.8% | Missing globals |
| `unknown` | 2 | 0.4% | Other issues |

---

## Top Symbols Not Found

| Symbol | Count | Analysis |
|--------|-------|----------|
| `target` | 56 | Decorator test directive confusion |
| `impl` | 38 | IIFE scoping issues |
| `number` | 37 | Primitive in extends clause |
| `x` | 29 | Constructor param not accessible (should be TS2301) |
| `IterableIterator` | 25 | Missing from lib.d.ts |
| `string` | 23 | Primitive in extends clause |
| `from` | 21 | Import statement keyword |
| `static` | 20 | Class member modifier |
| `type` | 19 | Type alias keyword |
| `any` | 16 | Primitive type keyword |
| `T`, `U` | 25 | Type parameters not in scope |

---

## Root Cause Analysis

### Category 1: Wrong Error Code (TS2304 vs TS2301)

**Example:** `initializerReferencingConstructorParameters.ts`
```typescript
class C {
    a = x;  // Our error: TS2304 "Cannot find name 'x'"
    constructor(x) { }  // TSC error: TS2301 "cannot reference 'x' declared in constructor"
}
```

**Problem:** The binder doesn't recognize that `x` exists as a constructor parameter. TypeScript correctly finds `x` but reports TS2301 (access restriction), not TS2304 (not found).

**Impact:** ~29 errors for `x` symbol

**Fix:** Binder should register constructor parameters in symbol table; checker should report appropriate access restriction errors.

---

### Category 2: Primitive Types in Extends Clause

**Example:** `classExtendingPrimitive.ts`
```typescript
class C extends number { }  // Our error: TS2304 "Cannot find name 'number'"
                           // TSC error: TS2693 "'number' only refers to a type"
```

**Problem:** When primitive keywords (`number`, `string`, `boolean`, `any`) appear in extends clauses, they should be recognized as type references that can't be used as values.

**Impact:** ~84 errors (37 + 23 + 12 + 16 = 88)

**Fix:** Parser/checker should recognize primitive type keywords in extends position and report TS2693 instead.

---

### Category 3: Type Parameters Not in Scope

**Example:** Generic functions
```typescript
function identity<T>(x: T): T {
    return x;  // Error: Cannot find name 'T'
}
```

**Problem:** Type parameters are not being registered in the symbol table when binding generic declarations.

**Impact:** ~30 errors (`T`: 12, `U`: 13, others: 5)

**Fix:** `bind_type_parameter_declaration` must add type parameters to current scope.

---

### Category 4: Missing Lib Types

**Symbol:** `IterableIterator` (25 occurrences)

**Problem:** Built-in iterator types from `lib.es2015.iterable.d.ts` are not loaded.

**Impact:** ~27 errors

**Fix:** Ensure comprehensive lib.d.ts loading includes:
- `IterableIterator`
- `AsyncIterableIterator`
- `DecoratorContext`

---

### Category 5: Keywords as Identifiers

**Symbols:** `from`, `static`, `type`, `import`, `get`, `set`, `as`, `await`

**Problem:** Parser treats certain contextual keywords as identifiers requiring resolution when they appear in specific syntactic positions.

| Keyword | Count | Context |
|---------|-------|---------|
| `from` | 21 | Import statements |
| `static` | 20 | Class members |
| `type` | 19 | Type alias declarations |
| `as` | 10 | Type assertions |
| `get`/`set` | 20 | Accessor declarations |
| `await` | 5 | Async contexts |
| `import` | 6 | Import expressions |

**Impact:** ~100+ errors

**Fix:** Parser needs better contextual keyword handling.

---

### Category 6: IIFE and Complex Scoping

**Example:** `fixSignatureCaching.ts`
```typescript
(function (define, undefined) {
    define(function () {
        var impl = {};  // Declared here
        impl.mobileDetectRules = {...};  // Error: Cannot find name 'impl'
    });
})(define, undefined);
```

**Problem:** Complex function scoping patterns (IIFEs, callbacks) may not correctly establish symbol scopes.

**Impact:** ~38 errors for `impl`

---

## Prioritized Fix Recommendations

### Priority 1: Constructor Parameter Scope (~29 errors)
- Register constructor parameters in binder
- Report TS2301 for illegal access instead of TS2304
- **Files:** `wasm/src/binder.rs`, `wasm/src/thin_binder.rs`

### Priority 2: Primitive Types in Extends (~88 errors)
- Recognize primitive keywords in extends position
- Report TS2693 instead of TS2304
- **Files:** Parser and checker

### Priority 3: Type Parameter Binding (~30 errors)
- Ensure `bind_type_parameter_declaration` adds to scope
- **Files:** `wasm/src/binder.rs`

### Priority 4: Contextual Keywords (~100 errors)
- Improve parser handling of `from`, `type`, `static`, etc.
- **Files:** `wasm/src/parser/`

### Priority 5: Lib Types (~27 errors)
- Add `IterableIterator`, `AsyncIterableIterator` to lib.d.ts
- **Files:** `tests/lib/lib.d.ts`

---

## Summary Table

| Fix | Est. Errors Fixed | Priority |
|-----|-------------------|----------|
| Constructor params | 29 | High |
| Primitive extends | 88 | High |
| Type parameters | 30 | High |
| Contextual keywords | 100+ | Medium |
| Lib types | 27 | Medium |
| IIFE scoping | 38 | Low |

**Total potential reduction:** ~300+ errors (56%+ of 531)

---

## Data Files

- **Analysis script:** `wasm/differential-test/analyze-extra-ts2304.mjs`
- **JSON report:** `wasm/differential-test/output/ts2304-extra-analysis.json`
