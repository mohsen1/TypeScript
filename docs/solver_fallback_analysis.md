# Solver Fallback Behavior Analysis

## Overview

This document analyzes fallback behaviors in the TypeScript type checking system that can suppress type errors. The codebase contains:

1. **TypeScript Compiler** (`src/compiler/checker.ts`) - Main implementation (3.1MB)
2. **Rust WASM Solver** (`wasm/src/solver/subtype.rs`) - WebAssembly component

Both implementations are analyzed below as they contribute to the overall type checking behavior.

---

## Part 1: TypeScript Compiler Analysis (`src/compiler/checker.ts`)

### Core Subtype Checking Architecture

#### Main Entry Points

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

### Fallback Behavior Analysis (TypeScript)

#### 1. Error Type is Actually `Any` Type (CRITICAL)

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

#### 2. `Ternary.Unknown` Returns (Bailout)

##### Location 1: Variance Checking - Line 23579
```typescript
if (variances === emptyArray) {
    return Ternary.Unknown;
}
```
- **Context**: Alias variance checking when variance cannot be determined
- **Effect**: Treats the comparison as unknown, which may allow invalid types

##### Location 2: Variance Checking - Line 23990
```typescript
if (variances === emptyArray) {
    return Ternary.Unknown;
}
```
- **Context**: Recursive invocation of `getVariances` for generic type references
- **Effect**: Skips variance checking for recursive types, potentially allowing invalid assignments

#### 3. `Ternary.Maybe` Returns (Assumptions)

##### Location 1: Circular Recursion Detection - Line 23304
```typescript
if (maybeKeysSet.has(id)) {
    return Ternary.Maybe;
}
```
- **Context**: `recursiveTypeRelatedTo` when source and target are already being compared
- **Effect**: Assumes types are related to break infinite recursion
- **Risk**: False positives when types are not actually related

##### Location 2: Deep Nesting Assumption - Line 23312
```typescript
if (broadestEquivalentId && maybeKeysSet.has(broadestEquivalentId)) {
    return Ternary.Maybe;
}
```
- **Context**: Constrained type parameter circular references
- **Effect**: Assumes types are related when circular constraints are detected

##### Location 3: Excessive Nesting - Line 23355
```typescript
if (expandingFlags === ExpandingFlags.Both) {
    result = Ternary.Maybe;
}
```
- **Context**: Both source and target are deeply nested type instantiations
- **Effect**: Assumes infinitely expanding types are equal
- **Risk**: False positives for complex generic types

#### 4. Structural Fallback for Unmeasurable Variance

**Location**: `relateVariances` - Line 24068
```typescript
if (some(variances, v => !!(v & VarianceFlags.AllowsStructuralFallback))) {
    originalErrorInfo = undefined;
    resetErrorInfo(saveErrorInfo);
    return undefined;
}
```

- **Context**: When variance cannot be determined (unmeasurable or unreliable)
- **Effect**: Falls back to structural comparison
- **Risk**: May allow invalid type assignments for generic types

### Error Poisoning Locations (TypeScript)

#### 1. Symbol Resolution Returns Error Type
**Location**: `getTypeOfSymbolAtLocation` - Line 1626
```typescript
return location ? getTypeOfSymbolAtLocation(symbol, location) : errorType;
```
- **Effect**: When location cannot be found, returns `errorType` (which is `Any`)
- **Downstream impact**: All further type checking is bypassed

#### 2. Type Resolution Returns Error Type
**Location**: `getTypeFromTypeNode` - Line 1661
```typescript
return node ? getTypeFromTypeNode(node) : errorType;
```
- **Effect**: When type node cannot be resolved, returns `errorType`
- **Downstream impact**: Invalid type annotations are silently accepted

#### 3. Global Type Resolution Returns Error Type
**Location**: `getGlobalOmitSymbol` check - Line 11588
```typescript
if (!omitTypeAlias) {
    return errorType;
}
```
- **Effect**: Missing global symbols result in error type
- **Downstream impact**: Invalid object rest/spread operations

---

## Part 2: Rust WASM Solver Analysis (`wasm/src/solver/subtype.rs`)

### Key Fallback Locations (Rust)

#### 1. Error Type Poisoning (subtype.rs:270-273)

**CRITICAL** - This is the primary source of suppressed errors.

```rust
// Error types are compatible with everything (for error recovery)
if source == TypeId::ERROR || target == TypeId::ERROR {
    return SubtypeResult::True;
}
```

**Impact**: When ANY type in a comparison is `ERROR`, the entire subtype check returns `True`, effectively silencing all downstream type errors.

**Problem**: This is overly broad error recovery. When an upstream error produces an `ERROR` type, subsequent type comparisons silently pass instead of propagating the error.

**Evidence from codebase**:
- `operations.rs:112, 117` - Returns `ERROR` on instantiation failures
- `operations.rs:114-115, 123-124` - Returns `ERROR` on argument mismatches
- `operations.rs:308, 317` - Returns `ERROR` in various failure paths

#### 2. Depth Check Fallback (subtype.rs:279-283)

Returns `Provisional` (treated as `True`) when recursion exceeds depth limit.

```rust
if self.depth > 100 {
    return SubtypeResult::Provisional;
}
```

**Impact**: Deeply nested types may incorrectly pass subtype checks.
**Severity**: Low - only affects pathological cases.

#### 3. Cycle Detection (subtype.rs:290-294)

Returns `Provisional` (treated as `True`) for recursive type cycles.

```rust
if self.in_progress.contains(&pair) {
    return SubtypeResult::Provisional;
}
```

**Impact**: This is correct coinductive semantics for recursive types, not a bug.

#### 4. Application Expansion Failures (subtype.rs:642-650, 652-660)

Returns `False` when generic applications cannot be expanded.

```rust
(TypeKey::Application(app_id), _) => {
    if let Some(expanded) = self.try_expand_application(*app_id) {
        self.check_subtype(expanded, target)
    } else {
        SubtypeResult::False
    }
}
```

**Impact**: Returns `False` (fails the check), which is correct behavior. Not a suppression issue.

#### 5. Type Resolution Failures (subtype.rs:682-731)

Returns `False` when symbol references cannot be resolved.

```rust
(TypeKey::Ref(s_sym), TypeKey::Ref(t_sym)) => {
    // ...
    (None, None) => {
        SubtypeResult::False
    }
}
```

**Impact**: Returns `False`, which is conservative and correct. Not a suppression issue.

### Other `TypeId::ANY` Usage (Rust)

Multiple locations in `operations.rs` use `TypeId::ANY` as a fallback:

1. **Line 1242**: `substitution.insert(info.name, TypeId::ANY)` - When type parameter cannot be resolved
2. **Line 1709-1712**: Object access on `ANY` returns `ANY`
3. **Lines 2126, 2221, 2253, 2274, 2285, 2341, 2361, 2380, 2394**: Array methods using `ANY` for rest parameters

---

## Key Data Structures

### Relation Comparison Results (TypeScript)
- `RelationComparisonResult.Succeeded` - Types are related
- `RelationComparisonResult.Failed` - Types are not related
- `RelationComparisonResult.ComplexityOverflow` - Too complex to check
- `RelationComparisonResult.StackDepthOverflow` - Recursion too deep
- `RelationComparisonResult.ReportsUnmeasurable` - Variance cannot be measured

### Ternary Type (TypeScript)
- `Ternary.True` - Definitely related
- `Ternary.False` - Definitely not related
- `Ternary.Maybe` - Related with assumptions (circular/recursive)
- `Ternary.Unknown` - Cannot determine

### SubtypeResult (Rust)
- `SubtypeResult::True` - The relationship is definitely true
- `SubtypeResult::False` - The relationship is definitely false
- `SubtypeResult::Provisional` - In a cycle and assuming true (coinductive)

---

## Recommendations

### High Priority (SOLV-4)

1. **Change `errorType` from `Any` to `Unknown`** (TypeScript line 2075)
   - This will cause errors to be reported instead of silently accepting invalid code
   - Aligns with PROJECT_DIRECTION.md directive: "Change the default fallback from `Any` to `Unknown`"

2. **Change ERROR fallback from `True` to `Unknown`** (Rust subtype.rs:270-273)
   - Modify the error poisoning behavior to be more restrictive

3. **Audit all `Ternary.Unknown` returns** (TypeScript)
   - Replace with explicit error handling or stricter defaults

4. **Reduce `Ternary.Maybe` assumptions** (TypeScript)
   - Consider making `Maybe` propagate as an error unless explicitly allowed

### Medium Priority

5. **Track error propagation path**
   - Add diagnostic context when `errorType` is returned
   - Help identify root cause of type resolution failures

6. **Propagate errors instead of silencing** (Rust)
   - When an `ERROR` type is encountered, emit a diagnostic indicating the comparison was poisoned

### Low Priority

7. **Add telemetry for fallback cases**
   - Track how often each fallback is triggered
   - Use data to prioritize fixes

---

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

// Should error: assignment to interface with wrong type
interface A { prop: number; }
let w: A = { prop: "string" };
```

---

## Files Referenced

### TypeScript Compiler
- `src/compiler/checker.ts` - Main type checker implementation (3.1MB)
- `src/compiler/types.ts` - Type system definitions
- `src/compiler/core.ts` - Core utilities

### Rust WASM Solver
- `wasm/src/solver/subtype.rs` - Subtype checking implementation (3476 lines)
- `wasm/src/solver/operations.rs` - Type operations
- `wasm/src/solver/instantiate.rs` - Type instantiation
- `wasm/src/solver/types.rs` - Type definitions

### Documentation
- `PROJECT_DIRECTION.md` - Project goals and directives
- `WORKER_7_TASK_LIST.md` - Current task list for Worker 7
