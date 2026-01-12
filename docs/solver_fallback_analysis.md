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

---

## Part 3: Detailed Audit of subtype.rs (Worker 7 - 2026-01-12)

**File:** `wasm/src/solver/subtype.rs` (3489 lines)
**Auditor:** Worker 7 (Solver Squad)
**Task:** SOLV-1 - Audit current solve_subtype implementation

### Key Finding: No `solve_subtype` Function

The file `subtype.rs` does not contain a function named `solve_subtype`. Instead, it contains:

1. **`is_subtype_of(source, target)`** - Main entry point (line 224-226)
2. **`check_subtype(source, target)`** - Internal check with cycle detection (line 235-308)
3. **`check_subtype_inner(source, target)`** - Structural checks (line 311-834)

### Complete Fallback Behavior Catalog

#### 1. Any/Unknown/ERROR Type Fast Paths (Lines 245-273)

These are **intentional TypeScript semantics**:

| Line | Condition | Result | Purpose |
|------|-----------|--------|---------|
| 246-248 | `source == TypeId::ANY` | `True` | Any is assignable to anything |
| 251-253 | `target == TypeId::ANY` | `True` | Everything is assignable to any |
| 256-258 | `target == TypeId::UNKNOWN` | `True` | Everything is assignable to unknown |
| 261-263 | `source == TypeId::NEVER` | `True` | Never is assignable to everything |
| 266-268 | `target == TypeId::NEVER` | `False` | Nothing is assignable to never |
| 271-273 | `source/target == TypeId::ERROR` | `True` | Error recovery mode |

**Assessment:** These are correct TypeScript semantics, not problematic fallbacks.

#### 2. Depth Check Fallback (Lines 279-283)

```rust
if self.depth > 100 {
    return SubtypeResult::Provisional;
}
```

**Location:** `check_subtype` function (line 279-283)
**Fallback:** Returns `Provisional` when recursion depth exceeds 100
**Purpose:** Stack overflow prevention for deeply nested or recursive types
**Risk Level:** **HIGH** - This suppresses downstream errors

**Impact:**
- Silently accepts deeply nested types as compatible
- May hide real type incompatibilities in complex types
- Could mask errors in generated or recursive generic types

#### 3. Cycle Detection Fallback (Lines 289-294)

```rust
if self.in_progress.contains(&pair) {
    return SubtypeResult::Provisional;
}
```

**Location:** `check_subtype` function (line 289-294)
**Fallback:** Returns `Provisional` when cycle is detected
**Purpose:** Coinductive semantics for recursive types
**Risk Level:** **MEDIUM** - This is intentional but can mask errors

#### 4. Type Resolution Failures (Multiple Locations)

**Lines 657, 667, 677, 687:** "Can't expand - assume not a subtype"
**Lines 728, 738:** "Can't resolve - assume not a subtype"
**Line 783:** "Can't resolve target TypeQuery - not assignable"

**Pattern:** When type expansion or resolution fails, return `False`
**Assessment:** Conservative behavior - rejecting when uncertain. Not a problem for missing errors.

#### 5. unwrap_or with TypeId::ANY (Lines 1967-1968)

```rust
fn are_this_parameters_compatible(
    &mut self,
    source_type: Option<TypeId>,
    target_type: Option<TypeId>,
) -> bool {
    if source_type.is_none() && target_type.is_none() {
        return true;
    }
    let source_type = source_type.unwrap_or(TypeId::ANY);
    let target_type = target_type.unwrap_or(TypeId::ANY);
    self.are_parameters_compatible(source_type, target_type)
}
```

**Location:** `are_this_parameters_compatible` function (line 1959-1970)
**Fallback:** Missing `this` parameters default to `ANY`
**Risk Level:** **MEDIUM** - May hide `this` parameter mismatches

#### 6. get_array_element_type with ANY (Lines 2164-2165)

```rust
fn get_array_element_type(&self, type_id: TypeId) -> TypeId {
    if type_id == TypeId::ANY {
        return TypeId::ANY;
    }
    // ... rest of function
}
```

**Location:** `get_array_element_type` function (line 2162-2172)
**Fallback:** `any[]` returns `any` as element type
**Assessment:** Correct TypeScript behavior

### Error Poisoning Locations (Detailed)

| Line | Context | Behavior |
|------|---------|----------|
| 271-273 | Main check_subtype | ERROR in source or target → True |
| 2654-2656 | explain_failure | ERROR in source or target → None (no explanation) |
| 1058 | is_object_keyword_type | ERROR is object keyword type |

### "Can't expand/resolve" Failures (Conservative)

#### Application Type Expansion (Lines 653-670)

```rust
// Source is Application, target is structural - try to expand and compare
(TypeKey::Application(app_id), _) => {
    if let Some(expanded) = self.try_expand_application(*app_id) {
        self.check_subtype(expanded, target)
    } else {
        // Can't expand - assume not a subtype
        SubtypeResult::False
    }
}
```

**Risk Level:** LOW - Conservative (rejects when uncertain)

### Special Rest Parameter Bivariance (Multiple Locations)

**Lines:** 2085, 2147, 2164-2165, 2281, 2339, 2374, 2432, 2467, 2525, 3194, 3271

```rust
let rest_is_top = self.allow_bivariant_rest
    && matches!(rest_elem_type, Some(TypeId::ANY | TypeId::UNKNOWN));
```

When `rest_is_top` is true, parameter checking is skipped and returns `True`.

**Assessment:** This is TypeScript's actual behavior for rest parameters with `any` or `unknown` types.

### Summary of Potential Issues

#### High Priority

1. **Depth Check Provisional Return (Line 279-283)**
   - Returns `Provisional` (treated as `True`) at depth 100
   - May hide real type errors in deeply nested types
   - No error reported to user

#### Medium Priority

2. **unwrap_or(TypeId::ANY) for this parameters (Lines 1967-1968)**
   - Missing `this` parameters default to `ANY`
   - Could miss `this` context validation errors

3. **Error Poisoning (Lines 271-273, 2654-2656)**
   - Suppresses all errors when ERROR type is present
   - May hide related type errors

#### Low Priority

4. **Conservative "Can't expand" failures**
   - These return `False` (reject) when uncertain
   - May cause false positives but not false negatives
   - User gets an error, so not silently hiding bugs

### Test Cases for Validation

#### Test 1: Depth Limit
```typescript
// Should this error? Currently returns Provisional → True
type Deep100<T> = { x: Deep99<T> };
type Deep99<T> = { x: Deep98<T> };
// ... 98 more levels
let x: Deep100<number> = "string";
```

#### Test 2: this parameter default
```typescript
// Should this error?
function fn(this: void) { }
const obj = { method: fn };
obj.method(); // this is obj, not void
```

#### Test 3: Error poisoning cascade
```typescript
// When first error occurs, are related errors reported?
type Bad = NonExistingType;
let x: Bad = 123; // Error 1
let y: Bad = "string"; // Should this also be Error 2?
```

### Function Call Tree

```
is_subtype_of (public entry)
  └─> check_subtype (cycle detection)
      ├─> [Fast paths] Any/Unknown/Never/Error (lines 245-273)
      ├─> [Depth check] Provisional if depth > 100 (lines 279-283)
      ├─> [Cycle detection] Provisional if cycle (lines 289-294)
      └─> check_subtype_inner (structural checks)
          ├─> evaluate_type
          ├─> check_intrinsic_subtype
          ├─> check_literal_to_intrinsic
          ├─> check_tuple_subtype
          ├─> check_function_subtype
          │   └─> are_this_parameters_compatible (unwrap_or ANY)
          ├─> check_callable_subtype
          ├─> check_object_subtype
          ├─> check_object_with_index_subtype
          └─> try_expand_application / try_expand_mapped
              └─> [Can't expand] → False
```
