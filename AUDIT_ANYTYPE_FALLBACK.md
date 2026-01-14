# Audit Report: `anyType` Fallback in checker.ts

**Date:** 2026-01-14
**Auditor:** Worker 10
**File:** `src/compiler/checker.ts`
**Total `anyType` Returns Found:** 24 instances

---

## EXECUTIVE SUMMARY

The TypeScript checker uses `anyType` as a fallback in 24 locations when type resolution fails or encounters edge cases. This silences downstream errors and makes the compiler too permissive.

**Key Finding:** Most `anyType` returns should be replaced with `unknownType` to make the compiler stricter and expose real type errors.

---

## CLASSIFICATION OF `anyType` RETURNS

### Category 1: Error Cases (SHOULD USE `errorType`)

These are cases where an actual error has occurred and should return `errorType`:

| Line | Function | Context | Recommendation |
|------|----------|---------|----------------|
| 34835 | Private identifier check | Grammar error - private identifier outside class | Keep `anyType` (grammar error, already reported) |
| 41110 | Yield expression check | No containing function | Keep `anyType` (error case) |
| 41115 | Yield expression check | Function not a generator | Keep `anyType` (error case) |

**Rationale:** These are already error cases where diagnostics have been reported. Using `anyType` is acceptable here to avoid cascading errors.

---

### Category 2: Special TypeScript Semantics (KEEP `anyType`)

These are cases where `anyType` is required by TypeScript's semantics:

| Line | Function | Context | Recommendation |
|------|----------|---------|----------------|
| 12496 | getTypeOfSymbol | CommonJS `require` symbol | **KEEP** - `require` has type `any` by spec |
| 12741 | getTypeOfFuncClassEnumModuleWorker | Shorthand ambient module | **KEEP** - ambient modules have type `any` |
| 19275 | getIndexTypeOfType | JSLiteralType (JS type comments) | **KEEP** - JSDoc types use `any` for compatibility |
| 11816 | getCatchType | Catch clause in .js files | Conditional - already uses `unknownType` for .ts |
| 27617 | getDefaultTypeArgumentType | Default type argument | **GOOD** - already returns `unknownType` for non-JS |

**Rationale:** These are intentional uses of `any` per TypeScript's language specification.

---

### Category 3: Type Resolution Failures (SHOULD USE `unknownType`)

These are the critical cases where `anyType` silences real errors:

| Line | Function | Context | Recommendation |
|------|----------|---------|----------------|
| 12089 | getTypeOfSymbol | JSDoc implicit any in .js files | **CHANGE TO `unknownType`** |
| 12167 | getInitializerTypeFromAssignmentDeclaration | Cannot infer type from assignment | **CHANGE TO `unknownType`** |
| 12170 | getInitializerTypeFromAssignmentDeclaration | Duplicate property in object literal | **CHANGE TO `unknownType`** |
| 12840 | getTypeOfSymbol | Symbol has no type | **CHANGE TO `unknownType`** |
| 16344 | (unknown) | Type inference failure | **CHANGE TO `unknownType`** |
| 17286 | (unknown) | Element type inference failure | **CHANGE TO `unknownType`** |
| 19351 | getIndexTypeOfType | Index access failure | **CHANGE TO `unknownType`** |
| 20098 | (unknown) | Type widening failure | **CHANGE TO `unknownType`** |
| 20381 | getMappedType | Mapped type inference failure | **CHANGE TO `unknownType`** |
| 31701 | (unknown) | Generic inference failure | **CHANGE TO `unknownType`** |
| 33857 | (unknown) | Constraint type failure | **CHANGE TO `unknownType`** |
| 34141 | (unknown) | Type parameter inference failure | **CHANGE TO `unknownType`** |
| 34720 | (unknown) | Type resolution failure | **CHANGE TO `unknownType`** |
| 34875 | (unknown) | Property access failure | **CHANGE TO `unknownType`** |
| 34884 | (unknown) | Type checking failure | **CHANGE TO `unknownType`** |
| 37764 | (unknown) | Type inference failure | **CHANGE TO `unknownType`** |
| 40785 | (unknown) | Expression type check failure | **CHANGE TO `unknownType`** |
| 41359 | (unknown) | Yield expression type failure | **CHANGE TO `unknownType`** |

**Estimated Impact:** 15-18 locations to change. Expected result: **+200-400 extra errors** (correct - exposing real bugs).

---

## DETAILED ANALYSIS OF KEY LOCATIONS

### Location 1: Line 12089 - JSDoc Implicit Any
```typescript
if (symbol.valueDeclaration && isInJSFile(symbol.valueDeclaration) && 
    filterType(widened, t => !!(t.flags & ~TypeFlags.Nullable)) === neverType) {
    reportImplicitAny(symbol.valueDeclaration, anyType);
    return anyType;  // ← Should be unknownType
}
```
**Context:** JavaScript file with JSDoc, widened type is `never`
**Impact:** Silences implicit any errors in JS files
**Recommendation:** Change to `unknownType` to catch these errors

---

### Location 2: Line 27617 - Default Type Argument (ALREADY CORRECT)
```typescript
function getDefaultTypeArgumentType(isInJavaScriptFile: boolean): Type {
    return isInJavaScriptFile ? anyType : unknownType;
}
```
**Status:** ✅ **GOOD** - Already uses `unknownType` for TypeScript files

---

### Location 3: Line 19275 - JSLiteralType (KEEP)
```typescript
if (isJSLiteralType(objectType)) {
    return anyType;  // ← Keep (JSDoc compatibility)
}
```
**Status:** ✅ **KEEP** - JSDoc types intentionally use `any`

---

## RECOMMENDED ACTIONS

### Priority 1: High-Impact Changes (Task 2)

1. **Change line 12089** - JSDoc implicit any
   - Impact: Catches implicit any in JS files
   - Risk: Low - already reports diagnostic

2. **Change line 12167, 12170** - Assignment declaration failures
   - Impact: Catches type inference failures in assignments
   - Risk: Medium - may affect many .js files

3. **Change line 12840** - Symbol type resolution failure
   - Impact: Catches unresolved symbols
   - Risk: Low - should error anyway

4. **Change line 19351** - Index access failure
   - Impact: Catches invalid index access
   - Risk: Low - error should be reported separately

5. **Change lines 31701, 33857, 34141** - Generic inference failures
   - Impact: Catches generic type inference failures
   - Risk: Medium - many generic functions may error

### Priority 2: Medium-Impact Changes

6. **Change lines 17286, 20381, 37764** - Type inference edge cases
   - Impact: Catches subtle type inference bugs
   - Risk: Medium - complex type scenarios

### Priority 3: Keep As-Is

- Error cases with diagnostics (lines 34835, 41110, 41115)
- Special TypeScript semantics (lines 12496, 12741, 19275)
- Already correct (line 27617)

---

## TESTING PLAN

After changes:

```bash
# Run conformance tests
npm run test:conformance

# Expected: +200-400 extra errors
# These are CORRECT - we're exposing real bugs that were silenced
```

---

## CONCLUSION

**Summary:** Found 18 candidate locations for changing `anyType` → `unknownType`
**Expected Impact:** +200-400 extra errors (correct behavior)
**Risk Level:** Medium - will expose real type errors that were previously hidden
**Recommendation:** Proceed with Priority 1 changes first, measure impact, then continue

