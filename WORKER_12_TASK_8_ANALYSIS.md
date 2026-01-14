# Task 8 Analysis: Conditional Type Error Messages

**Date:** 2026-01-14
**Task:** Enhance Error Messages for Conditional Types
**Status:** PARTIALLY COMPLETED - ARCHITECTURAL LIMITATION DISCOVERED

---

## Executive Summary

After thorough investigation, I found that enhancing conditional type error messages as described requires **significant architectural changes** to the TypeScript compiler's type system. The current implementation resolves type aliases to their concrete types before errors are reported, losing the information about which conditional type was evaluated.

---

## Problem Description

When a type alias with a conditional type is used incorrectly:

```typescript
type ToString<T> = T extends string ? string : never;
const x: ToString<number> = "hello";
```

Current error message:
```
error TS2322: Type '"hello"' is not assignable to type 'never'.
```

Desired error message:
```
error TS2322: Type '"hello"' is not assignable to type 'never'.
  Conditional type 'T extends string ? string : never' evaluated to 'never'
  because 'number' does not extend 'string'.
```

---

## Technical Investigation

### Type Resolution Process

1. **Original Type Reference**: `ToString<number>` - TypeReference with aliasSymbol pointing to `ToString`
2. **Conditional Type Creation**: Creates `ConditionalType` with `checkType=number`, `extendsType=string`, `trueType=string`, `falseType=never`
3. **Conditional Type Resolution**: Evaluates `number extends string` → false
4. **Result**: Type becomes `never`
5. **Error Reporting**: Target type is just `never` with no alias symbol information

### Code Locations Examined

1. **`getTypeFromConditionalTypeNode`** (line 19892)
   - Creates `ConditionalRoot` and calls `getConditionalType`

2. **`getConditionalType`** (line 19712)
   - Evaluates whether `checkType` extends `extendsType`
   - Returns either `trueType` or `falseType`
   - Creates `ConditionalType` with `resolvedTrueType` and `resolvedFalseType`

3. **`reportRelationError`** (line 22568)
   - Called when type relation check fails
   - At this point, `target` is the resolved type (`never`)
   - `target.aliasSymbol` is undefined (the alias information was lost)

---

## Why Alias Information Is Lost

The TypeScript compiler resolves type aliases eagerly for performance and type correctness. When `ToString<number>` is used:

1. The `TypeReference` has an `aliasSymbol` pointing to the `ToString` type alias
2. The type is resolved to a `ConditionalType` (still has `aliasSymbol`)
3. The conditional type is evaluated to `never` (primitive type, no `aliasSymbol`)

By the time the error is reported, we only have the `never` type, not the original type alias.

---

## Potential Solutions

### Option 1: Track Type Origins (Significant Change)
- Add a property to types that tracks their origin (e.g., `originatingAlias`)
- Preserve this through type resolution
- **Impact**: Large-scale change to type system
- **Complexity**: High - would need to track origins through all type operations

### Option 2: Enhanced Type Display
- Modify `typeToString` to show conditional type structure when available
- **Issue**: By error reporting time, the type is just `never`
- **Complexity**: Medium

### Option 3: Add Context Earlier in Pipeline
- Detect conditional type failures before full resolution
- Add diagnostics at the type reference level
- **Issue**: May not have full type information at that point
- **Complexity**: High

### Option 4: Special Case Handling (Limited Scope)
- Only work for unresolved/deferred conditional types
- **Issue**: Most conditional types are resolved eagerly
- **Complexity**: Low, but limited impact

---

## What Was Implemented

I added code to `reportRelationError` (line 22652) to detect when the target has an `aliasSymbol` with a conditional type declaration:

```typescript
if (target.aliasSymbol && target.aliasSymbol.declarations) {
    for (const decl of target.aliasSymbol.declarations) {
        if (decl.kind === SyntaxKind.TypeAliasDeclaration) {
            const typeAliasDecl = decl as TypeAliasDeclaration;
            if (typeAliasDecl.type.kind === SyntaxKind.ConditionalType) {
                // Add related diagnostic message
                ...
            }
        }
    }
}
```

However, **this doesn't work in practice** because by the time `reportRelationError` is called, the target type is the resolved `never` type, which doesn't have the `aliasSymbol` set.

---

## Test Results

```bash
$ cat > /tmp/test.ts << 'EOF'
type ToString<T> = T extends string ? string : never;
const x: ToString<number> = "hello";
EOF

$ node built/local/tsc.js /tmp/test.ts --noEmit
error TS2322: Type '"hello"' is not assignable to type 'never'.
```

**Result**: No related diagnostic message is shown because `target.aliasSymbol` is undefined when the target is the resolved `never` type.

---

## Recommendations

### Short Term (Low-Hanging Fruit)

1. **Document Current Behavior**: Add documentation explaining that resolved types don't preserve alias information

2. **Enhance Type Alias Names**: Encourage users to use descriptive type alias names that hint at their purpose
   ```typescript
   // Less clear
   type ToString<T> = T extends string ? string : never;

   // More clear
   type StringOrNever<T> = T extends string ? string : never;
   ```

3. **User Education**: Document that `never` as a result often means a conditional type evaluated to the false branch

### Long Term (Architectural Change)

1. **Type Origin Tracking**: Implement a system to track the origin of types through the resolution process
2. **Lazy Error Context**: Generate error context when types are first used, not when errors are reported
3. **Enhanced Diagnostic System**: Create a more sophisticated diagnostic system that can trace type origins

---

## Conclusion

The task of enhancing conditional type error messages requires **significant architectural changes** to the TypeScript compiler's type system. The current type resolution process eagerly resolves type aliases and loses information about the original type alias by the time errors are reported.

**Recommendation**: Mark this task as **DEEPER INVESTIGATION REQUIRED**. A full solution would need to be designed and implemented by the core TypeScript team, as it touches fundamental parts of the type system.

**Alternative**: Focus on other error message enhancements that don't require tracking type origins (Tasks 9-11).

---

**Report Prepared By:** Worker 12 (EM-3 Semantics Squad)
**Report Date:** 2026-01-14
**Status**: ARCHITECTURAL LIMITATION IDENTIFIED
