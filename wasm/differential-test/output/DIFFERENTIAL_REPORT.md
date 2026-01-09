# Differential Testing Report: TypeScript vs Rust/WASM Compiler

**Generated:** 2026-01-09
**Test Suite:** Unsoundness Catalog + Phase 1 Critical Tests

## Critical Finding: Interface Member Parsing Bug

During differential testing, a critical bug was discovered in interface type lowering. When an interface has **two or more** property signatures, the type system misinterprets the second property.

### Reproduction

```typescript
// Works correctly (single property):
interface Foo { x: number; }
const p1: Foo = { x: 1 };  // OK

// FAILS (two properties):
interface Bar { x: number; y: number; }
const p2: Bar = { x: 1, y: 2 };
// Error: Property 'number' is missing in type '{ x: number; y: number }'
// but required in type '{ number: any; x: number }'.
```

The error message reveals the bug: the interface `{ x: number; y: number }` is being interpreted as `{ number: any; x: number }`. The parser treats the `y: number;` line's type keyword `number` as a property name instead of a type annotation.

### Investigation Findings

**Parsing is CORRECT:**
```
Interface 'Point' found at node 9
  members list: [NodeIndex(4), NodeIndex(8)]
  Member 0 (idx 4): kind=172
    name_idx: NodeIndex(1)
    type_annotation_idx: NodeIndex(3)
    name_text: 'x'
    type_node kind: 184
  Member 1 (idx 8): kind=172
    name_idx: NodeIndex(5)
    type_annotation_idx: NodeIndex(7)
    name_text: 'y'
    type_node kind: 184
```

Both properties are parsed correctly with the right names and type annotations.

**Bug Location:**
The bug is in TYPE LOWERING, not parsing. When `collect_interface_members` processes the second property signature, it appears to be retrieving the wrong SignatureData from the `signatures` vector.

Specifically:
- Each ThinNode has a `data_index` field that indexes into type-specific data vectors
- For PROPERTY_SIGNATURE nodes, `data_index` indexes into `arena.signatures`
- The `get_signature(node)` method uses `signatures.get(node.data_index as usize)`

**Suspected Root Cause:**
When lowering the second property, `sig.name` returns NodeIndex(3) instead of NodeIndex(5), which is the TYPE_REFERENCE node for the first property's type annotation instead of the identifier node for `y`.

This could be caused by:
1. Wrong `data_index` stored in the second property's ThinNode
2. Data corruption in the `signatures` vector
3. Off-by-one error in how signatures are indexed

### Impact
This bug affects ALL interfaces with more than one property, which is the vast majority of real-world TypeScript code.

---

## Executive Summary

| Metric | Value |
|--------|-------|
| Total Tests | 14 |
| Passed | 5 (35.7%) |
| Failed | 9 (64.3%) |
| High Severity (Missing Errors) | 2 |
| Medium Severity (Extra Errors) | 7 |

## Prioritized Bug List

### High Priority: Missing Errors (Potential Unsoundness)

These are cases where TSC reports an error but the WASM compiler does not, indicating potential unsoundness in the Rust implementation.

#### 1. TS2559: Weak Type Detection

**Catalog Item:** #13 - Weak Type Detection
**Phase:** 4 (Feature Barrier)

```typescript
interface Weak { a?: number }
const x = { b: "unrelated" };
const y: Weak = x; // TSC Error: Type '{ b: string; }' has no properties in common with type 'Weak'.
```

**TSC Behavior:** Reports error TS2559 - weak types (all optional properties) require at least one overlapping property
**WASM Behavior:** No error - treats it as valid structural subtyping (incorrect)

**Fix Required:** Implement weak type detection in the solver. When the target type has *only* optional properties, verify that the source type shares at least one property key with the target.

---

#### 2. TS2322: Generic Constraint Assignability

**Catalog Item:** #31 - Base Constraint Assignability
**Phase:** 4 (Feature Barrier)

```typescript
function f<T extends string, U>(x: T, y: U) {
    let s: string = x; // OK: T extends string
    let t: T = "hello"; // TSC Error: Type 'string' is not assignable to type 'T'.
}
```

**TSC Behavior:** Reports TS2322 - a string literal cannot be assigned to generic type `T` (even though `T extends string`)
**WASM Behavior:** Reports TS2304 "Cannot find name 'T'" - suggests generic type parameter resolution is not working

**Fix Required:**
1. Ensure type parameters are properly scoped within generic function bodies
2. Implement the rule: `Constraint(T) <: U` does NOT imply `Literal <: T`

---

### Medium Priority: Extra Errors (False Positives)

These are cases where the WASM compiler reports errors that TSC does not report. While less severe than missing errors, they will cause valid code to be rejected.

#### Common Pattern: TS2304 "Cannot find name" for Type Parameters

Multiple tests show the WASM compiler failing to resolve type parameters:

| Test Case | Extra Errors |
|-----------|--------------|
| AnyTypeTests | 7 × TS2304 |
| ObjectTrifectaTests | 8 × TS2304, TS2322, TS2353 |
| VoidReturnTests | 4 × TS2304 |
| CovariantArrayTests | TS2322, 3 × TS2304 |
| NullUndefinedTests | 3 × TS2304 |
| OptionalityTests | 4 × TS2304 |
| ExcessPropertyTests | 4 × TS2304 |

**Root Cause Analysis:**
The high number of TS2304 errors suggests that type annotations (especially interface names like `Animal`, `Dog`, `Point`, `Config`) are not being properly resolved from their declarations to their uses.

**Potential Issues:**
1. Binder is not properly linking type declarations to their symbols
2. Type annotation resolution in the checker is not searching the correct scopes
3. Export/namespace handling may be incorrect

---

## Passing Tests

The following tests show correct parity between TSC and WASM:

| Test | TSC Errors | WASM Errors | Status |
|------|------------|-------------|--------|
| Rest Parameter Bivariance | 0 | 0 | ✓ |
| Covariant `this` Types | 0 | 0 | ✓ |
| Error Poisoning | 1 | 1 | ✓ |
| Function Bivariance | 1 | 1 | ✓ |
| Literal Widening | 1 | 1 | ✓ |

---

## Recommendations

### Immediate Actions (This Sprint)

1. **Fix Type Parameter Resolution**
   - Investigate why type parameters like `T`, `U` are not found in generic function bodies
   - Check the binder's handling of type parameter declarations
   - Ensure type parameters are added to the correct scope

2. **Fix Interface/Type Alias Resolution**
   - Debug why declared interfaces (`Animal`, `Dog`, `Point`) produce "Cannot find name" errors
   - Check scope chain traversal in the checker

### Near-Term Actions (Next Sprint)

3. **Implement Weak Type Detection (TS2559)**
   - Add logic to solver/compat.rs to detect "weak" types (only optional properties)
   - Enforce overlapping property requirement

4. **Implement Generic Constraint Assignability Rules**
   - Ensure `T = literal` is rejected when `T extends BaseType`
   - Add proper variance handling for generic type parameters

### Long-Term Actions

5. **Expand Differential Test Suite**
   - Add tests from `tests/cases/compiler/` directory
   - Automate running against TypeScript's full conformance suite
   - Set up CI to catch regressions

---

## Test Infrastructure

The differential test harness is located at:
```
wasm/differential-test/
├── runner.js          # Main test runner
├── package.json       # Dependencies
├── test-cases/        # Custom test files
│   └── phase1-critical.ts
└── output/
    ├── report.json    # Machine-readable results
    └── DIFFERENTIAL_REPORT.md
```

### Running Tests

```bash
cd wasm/differential-test
npm install
npm test                    # Run all tests
npm run test:catalog        # Run only Phase 1 tests
npm run test:verbose        # Show detailed output
```

---

## Appendix: Error Code Reference

| Code | Description |
|------|-------------|
| TS2304 | Cannot find name 'X' |
| TS2322 | Type 'X' is not assignable to type 'Y' |
| TS2353 | Object literal may only specify known properties |
| TS2559 | Type has no properties in common with type (weak type check) |
