# Solver Fallback Analysis - Worker 7

## Overview
This document analyzes the current `solve_subtype` implementation in the TypeScript compiler to identify locations that return `Any` or `True` as fallback, and where error poisoning suppresses downstream errors.

**Key Finding**: The codebase uses TypeScript, not Rust. The solver/subtype logic is located in `src/compiler/checker.ts`.

## Core Subtype Checking Architecture

### Main Entry Points

1. **`isTypeSubtypeOf(source, target)`** - Line 21227
   - Returns `boolean`
   - Calls `isTypeRelatedTo(source, target, subtypeRelation)`

2. **`isTypeRelatedTo(source, target, relation)`** - Line 22222
   - Core type comparison function
   - Returns `boolean`
   - Delegates to `checkTypeRelatedTo` for complex types

3. **`checkTypeRelatedTo(...)`** - Line 22320
   - Main type checking implementation
   - Handles error reporting
   - Calls `isRelatedTo` for actual comparison

4. **`isRelatedTo(...)`** - Line 22699
   - Returns `Ternary` (True, Maybe, or False)
   - Normalizes types and performs comparison

## Fallback Behavior Analysis

### 1. Error Type is Actually `Any` Type (CRITICAL)

**Location**: `src/compiler/checker.ts:2075`
```typescript
var errorType = createIntrinsicType(TypeFlags.Any, "error");
```

**Impact**: When the compiler encounters an error (e.g., cannot resolve a symbol), it returns `errorType`, which is **literally an `Any` type**. This causes:
- Silent acceptance of invalid code
- Suppression of downstream type errors (error poisoning)
- False negatives in type checking

**Detection**: `isErrorType()` at line 11530
```typescript
function isErrorType(type: Type) {
    return type === errorType || !!(type.flags & TypeFlags.Any && type.aliasSymbol);
}
```

### 2. `Ternary.Unknown` Returns (Bailout)

#### Location 1: Variance Checking - Line 23579
```typescript
if (variances === emptyArray) {
    return Ternary.Unknown;
}
```
- **Context**: Alias variance checking when variance cannot be determined
- **Effect**: Treats the comparison as unknown, which may allow invalid types

#### Location 2: Variance Checking - Line 23990
```typescript
if (variances === emptyArray) {
    return Ternary.Unknown;
}
```
- **Context**: Recursive invocation of `getVariances` for generic type references
- **Comment**: "We return Ternary.Maybe for a recursive invocation of getVariances (signalled by emptyArray). This effectively means we measure variance only from type parameter occurrences that aren't nested in recursive instantiations of the generic type."
- **Effect**: Skips variance checking for recursive types, potentially allowing invalid assignments

### 3. `Ternary.Maybe` Returns (Assumptions)

#### Location 1: Circular Recursion Detection - Line 23304
```typescript
if (maybeKeysSet.has(id)) {
    return Ternary.Maybe;
}
```
- **Context**: `recursiveTypeRelatedTo` when source and target are already being compared
- **Effect**: Assumes types are related to break infinite recursion
- **Risk**: False positives when types are not actually related

#### Location 2: Deep Nesting Assumption - Line 23312
```typescript
if (broadestEquivalentId && maybeKeysSet.has(broadestEquivalentId)) {
    return Ternary.Maybe;
}
```
- **Context**: Constrained type parameter circular references
- **Effect**: Assumes types are related when circular constraints are detected

#### Location 3: Excessive Nesting - Line 23355
```typescript
if (expandingFlags === ExpandingFlags.Both) {
    result = Ternary.Maybe;
}
```
- **Context**: Both source and target are deeply nested type instantiations
- **Effect**: Assumes infinitely expanding types are equal
- **Comment**: "assume the types are equal and infinitely expanding"
- **Risk**: False positives for complex generic types

### 4. Structural Fallback for Unmeasurable Variance

**Location**: `relateVariances` - Line 24068
```typescript
if (some(variances, v => !!(v & VarianceFlags.AllowsStructuralFallback))) {
    // If some type parameter was `Unmeasurable` or `Unreliable`, and we couldn't pass by assuming it was identical, then we
    // have to allow a structural fallback check
    // We elide the variance-based error elaborations, since those might not be too helpful, since we'll potentially
    // be assuming identity of the type parameter.
    originalErrorInfo = undefined;
    resetErrorInfo(saveErrorInfo);
    return undefined;
}
```

- **Context**: When variance cannot be determined (unmeasurable or unreliable)
- **Effect**: Falls back to structural comparison
- **Risk**: May allow invalid type assignments for generic types

### 5. Early `Ternary.True` Returns

#### Location 1: Type Identity - Line 22700
```typescript
if (originalSource === originalTarget) return Ternary.True;
```
- **Correct behavior**: Same type reference

#### Location 2: Singleton Types - Line 22728
```typescript
if (source.flags & TypeFlags.Singleton) return Ternary.True;
```
- **Context**: Identity relation check
- **Correct behavior**: Singleton types (undefined, null, etc.)

#### Location 3: Type Parameter Fast Path - Line 22738
```typescript
if (source.flags & TypeFlags.TypeParameter && getConstraintOfType(source) === target) {
    return Ternary.True;
}
```
- **Context**: Type parameter with constraint exactly matching target
- **Correct behavior**: Optimized common case

## Error Poisoning Locations

### 1. Symbol Resolution Returns Error Type

**Location**: `getTypeOfSymbolAtLocation` - Line 1626
```typescript
return location ? getTypeOfSymbolAtLocation(symbol, location) : errorType;
```
- **Effect**: When location cannot be found, returns `errorType` (which is `Any`)
- **Downstream impact**: All further type checking is bypassed

### 2. Type Resolution Returns Error Type

**Location**: `getTypeFromTypeNode` - Line 1661
```typescript
return node ? getTypeFromTypeNode(node) : errorType;
```
- **Effect**: When type node cannot be resolved, returns `errorType`
- **Downstream impact**: Invalid type annotations are silently accepted

### 3. Global Type Resolution Returns Error Type

**Location**: `getGlobalOmitSymbol` check - Line 11588
```typescript
if (!omitTypeAlias) {
    return errorType;
}
```
- **Effect**: Missing global symbols result in error type
- **Downstream impact**: Invalid object rest/spread operations

### 4. Marker Type Creation Returns Error Type

**Location**: `createMarkerType` - Line 25019
```typescript
if (isErrorType(type)) {
    return type;
}
```
- **Context**: Creating marker types for variance checking
- **Effect**: When declared type is an error type, returns it directly
- **Downstream impact**: Variance checking is bypassed

## Key Data Structures

### Relation Comparison Results
- `RelationComparisonResult.Succeeded` - Types are related
- `RelationComparisonResult.Failed` - Types are not related
- `RelationComparisonResult.ComplexityOverflow` - Too complex to check
- `RelationComparisonResult.StackDepthOverflow` - Recursion too deep
- `RelationComparisonResult.ReportsUnmeasurable` - Variance cannot be measured
- `RelationComparisonResult.ReportsUnreliable` - Variance result is unreliable

### Ternary Type
- `Ternary.True` - Definitely related
- `Ternary.False` - Definitely not related
- `Ternary.Maybe` - Related with assumptions (circular/recursive)
- `Ternary.Unknown` - Cannot determine

## Recommendations

### High Priority
1. **Change `errorType` from `Any` to `Unknown`** (line 2075)
   - This will cause errors to be reported instead of silently accepting invalid code
   - Aligns with PROJECT_DIRECTION.md directive: "Change the default fallback from `Any` to `Unknown`"

2. **Audit all `Ternary.Unknown` returns**
   - Replace with explicit error handling or stricter defaults
   - Document why each case cannot determine the relationship

3. **Reduce `Ternary.Maybe` assumptions**
   - Currently, `Maybe` is treated as success in many contexts
   - Consider making `Maybe` propagate as an error unless explicitly allowed

### Medium Priority
4. **Track error propagation path**
   - Add diagnostic context when `errorType` is returned
   - Help identify root cause of type resolution failures

5. **Improve variance checking fallback**
   - Currently falls back to structural comparison too eagerly
   - Consider stricter defaults for generic types

### Low Priority
6. **Add telemetry for fallback cases**
   - Track how often each fallback is triggered
   - Use data to prioritize fixes

## Test Cases to Verify

After making changes, verify these scenarios produce errors:

```typescript
// Should error: string is not assignable to number
let x: number = "string";

// Should error: wrong generic type
let y: Array<string> = [1, 2, 3];

// Should error: unresolved symbol
let z: SomeNonExistentType = 123;

// Should error: generic with wrong type argument
function foo<T extends string>(x: T): void {}
foo(123); // Should error
```

## Files Referenced

- `src/compiler/checker.ts` - Main type checker implementation
- `PROJECT_DIRECTION.md` - Project goals and directives
- `WORKER_7_TASK_LIST.md` - Current task list for Worker 7
