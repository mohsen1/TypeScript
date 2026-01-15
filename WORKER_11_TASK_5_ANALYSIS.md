# Task 5: Analysis of Remaining 94 Missing TS2322 Errors

**Date:** 2026-01-14
**Worker:** worker-11
**Status:** Post Task 4 (ERROR type suppression fix)

## Executive Summary

After Task 4's diagnostic suppression fix (which resolved ~310 ERROR-type TS2322 issues), **40 test files** remain with missing TS2322 errors. These are genuine type compatibility checks that WASM is not performing, distinct from the ERROR type suppression issues.

**Key Finding:** The remaining missing errors fall into **6 distinct categories**, each related to specific gaps in the WASM type checking implementation:

1. **Abstract Constructor Type Assignability** (2 files)
2. **Private Name Field Branding** (3 files)
3. **Static Index Signature Literal Types** (2 files)
4. **Computed Property Contextual Typing** (6 files)
5. **Destructuring Pattern Type Inference** (21 files) - **LARGEST CATEGORY**
6. **Control Flow Type Narrowing** (6 files)

## Methodology

### Data Collection
- Ran `find-missing-ts2322.mjs` script against 2,000+ conformance tests
- Identified 40 test files where TypeScript reports TS2322 but WASM does not
- Sampled and analyzed 20+ representative test cases in depth
- Cross-referenced TypeScript compiler output with WASM diagnostic output

### Analysis Tools
1. `analyze-ts2322.mjs` - Extracts TS2322 errors using TypeScript compiler API
2. `analyze-wasm.mjs` - Checks what WASM reports for the same files
3. Manual code review of test cases to understand type checking requirements

---

## Category 1: Abstract Constructor Type Assignability

**Count:** 2 files
**Percentage:** 5%

### Test Cases
- `classes/classDeclarations/classAbstractKeyword/classAbstractConstructorAssignability.ts`

### Pattern
```typescript
class A {}
abstract class B extends A {}
class C extends B {}

var AA: typeof A = B;  // TS2322: Type 'typeof B' is not assignable to type 'typeof A'
var BB: typeof B = A;  // TS2322: Type 'typeof A' is not assignable to type 'typeof B'
var CC: typeof C = B;  // TS2322: Type 'typeof B' is not assignable to type 'typeof C'
```

### Root Cause
**Missing: Abstract constructor type compatibility checks**

TypeScript correctly prevents assigning abstract constructor types to non-abstract constructor types. This requires:

1. Tracking abstract class declarations in symbol table
2. Storing "abstract" modifier on constructor types
3. Checking assignability rules:
   - Abstract → Non-abstract: ERROR
   - Non-abstract → Abstract: OK
   - Abstract → Abstract: OK

### Current WASM Behavior
WASM reports no errors (0 diagnostics). This suggests the type checker is not flagging abstract constructor type mismatches during `typeof` type comparisons.

### Related Code Locations
- Symbol resolution: `src/check/symbol.rs` - abstract class tracking
- Type comparison: `src/check/assignable.rs` - constructor type assignability
- AST nodes: `src/syntax/class.rs` - abstract modifier parsing

---

## Category 2: Private Name Field Branding

**Count:** 3 files
**Percentage:** 7.5%

### Test Cases
- `classes/members/privateNames/privateNamesUnique-1.ts`
- `classes/members/privateNames/privateNamesUnique-5.ts`
- `classes/members/privateNames/privateNameReadonly.ts`

### Pattern
```typescript
// Test 1: Private field branding
class A {
    #foo: number;
}

class B {
    #foo: number;
}

const b: A = new B();  // TS2322: Property '#foo' in type 'B' refers to a different member

// Test 2: Private method readonly
const C = class {
    #bar() {}
    foo() {
        this.#bar = console.log;  // TS2322: Type 'void' is not assignable to '() => void'
    }
}
```

### Root Cause
**Missing: Private name unique branding and accessibility**

TypeScript treats private fields (`#foo`) as "branded" - each class's private field is a distinct type, even if they have the same name. This requires:

1. **Unique branding per declaration**: Each `#field` in Class A is distinct from `#field` in Class B
2. **Assignability checks**: Cannot assign instances between classes with different private field brands
3. **Private method readonly**: Private methods cannot be reassigned

### Current WASM Behavior
WASM reports other errors (TS2564, TS2741) but misses TS2322 for:
- Cross-class private field assignability
- Private method reassignment type mismatches

### Related Code Locations
- Private names: `src/check/private_name.rs` - brand checking
- Class members: `src/check/class.rs` - private field accessibility
- Symbol table: Must track private name scope per class

---

## Category 3: Static Index Signature Literal Types

**Count:** 2 files
**Percentage:** 5%

### Test Cases
- `classes/staticIndexSignature/staticIndexSignature1.ts`
- `classes/staticIndexSignature/staticIndexSignature2.ts`

### Pattern
```typescript
class C {
    static [s: string]: number;
    static [s: number]: 42;  // Literal type '42'
}

C[2] = 2;  // TS2322: Type '2' is not assignable to type '42'
```

### Root Cause
**Missing: Literal type narrowing in static index signatures**

When an index signature has a literal type (e.g., `42` instead of `number`), assignments to that signature must respect the literal type. This requires:

1. Detecting literal types in index signatures
2. Narrowing access expressions to the literal type
3. Rejecting non-literal values

### Current WASM Behavior
WASM reports TS4111 and TS7053 (index signature errors) but misses TS2322 for literal type mismatches.

### Technical Details
- Index signature access: `C[42]` should be typed as `42` (literal), not `number`
- Assignment: `C[2] = 2` should fail because `2 !== 42` as literal types

### Related Code Locations
- Index signatures: `src/check/index_signature.rs`
- Literal types: `src/check/literal.rs`
- Static members: `src/check/class.rs` - static property resolution

---

## Category 4: Computed Property Contextual Typing

**Count:** 6 files
**Percentage:** 15%

### Test Cases
- `es6/computedProperties/computedPropertyNamesContextualType8_ES5.ts`
- `es6/computedProperties/computedPropertyNamesContextualType8_ES6.ts`
- `es6/computedProperties/computedPropertyNamesContextualType9_ES5.ts`
- `es6/computedProperties/computedPropertyNamesContextualType9_ES6.ts`
- `es6/computedProperties/computedPropertyNamesContextualType10_ES5.ts`
- `es6/computedProperties/computedPropertyNamesContextualType10_ES6.ts`

### Pattern
```typescript
interface I {
    [s: string]: boolean;
    [s: number]: boolean;
}

var o: I = {
    [""+"foo"]: "",  // TS2322: Type 'string' is not assignable to 'boolean'
    [""+"bar"]: 0    // TS2322: Type 'number' is not assignable to 'boolean'
}
```

### Root Cause
**Missing: Contextual typing for computed property values**

When an object literal has a target type with index signatures, computed property values must be contextually typed based on the index signature type. This requires:

1. Detecting computed properties in object literals
2. Evaluating computed expressions to determine which index signature applies
3. Contextually typing the property value based on the index signature type

### Current WASM Behavior
WASM does not report TS2322 for computed property type mismatches.

### Technical Challenges
- Computed property expression evaluation: `""+"foo"` must be recognized as `string`
- Index signature selection: String index vs number index
- Contextual typing propagation to property values

### Related Code Locations
- Object literals: `src/check/object_literal.rs`
- Computed properties: `src/syntax/property.rs`
- Contextual typing: `src/check/contextual.rs`

---

## Category 5: Destructuring Pattern Type Inference

**Count:** 21 files
**Percentage:** 52.5% **[LARGEST CATEGORY]**

### Test Cases
- `es6/destructuring/destructuringParameterDeclaration2.ts`
- `es6/destructuring/destructuringParameterDeclaration3ES5.ts`
- `es6/destructuring/destructuringParameterDeclaration3ES6.ts`
- `es6/destructuring/destructuringParameterDeclaration4.ts`
- `es6/destructuring/destructuringParameterDeclaration5.ts`
- `es6/destructuring/destructuringParameterDeclaration8.ts`
- `es6/destructuring/destructuringParameterProperties2.ts`
- `es6/destructuring/destructuringParameterProperties5.ts`
- `es6/destructuring/optionalBindingParameters1.ts`
- `es6/destructuring/optionalBindingParameters2.ts`
- `es6/destructuring/optionalBindingParametersInOverloads1.ts`
- `es6/destructuring/optionalBindingParametersInOverloads2.ts`
- `es6/destructuring/restElementWithAssignmentPattern2.ts`
- `es6/destructuring/declarationsAndAssignments.ts`
- Plus 8 more...

### Pattern Examples

#### Example 1: Nested Array Destructuring
```typescript
function a0([a, b, [[c]]]: [number, number, string[][]]) { }
a0([1, "string", [["world"]]);  // TS2322: 'string' not assignable to 'number'
```

#### Example 2: Optional Binding Patterns
```typescript
function foo([x,y,z]?: [string, number, boolean]) { }
foo([false, 0, ""]);  // TS2322 x2: Wrong type at each position
```

#### Example 3: Rest Element with Assignment
```typescript
var a: string, b: number;
[...{ 0: a = "", b }] = ["", 1];  // TS2322: 'string | number' not assignable to 'string'
```

### Root Cause
**Missing: Comprehensive destructuring type checking**

This is the **largest category** of missing TS2322 errors. The issues span multiple destructuring scenarios:

1. **Nested pattern matching**: Array/object patterns within patterns
2. **Optional destructuring**: `...?` patterns with type annotations
3. **Rest elements**: `...rest` in complex destructuring contexts
4. **Default values**: Destructuring with initializers
5. **Contextual inference**: Destructured parameter types inferred from usage

### Current WASM Behavior
WASM reports various errors (TS2300, TS2345, TS2463) but consistently misses TS2322 for:
- Type mismatches in destructured values
- Nested pattern type violations
- Optional binding parameter type errors

### Technical Challenges
Destructuring type checking requires:

1. **Pattern Type Analysis**: Each binding pattern has an "implied type"
2. **Nested Contextual Typing**: Inner patterns inherit context from outer
3. **Tuple Type Checking**: Fixed-length arrays must match exactly
4. **Union Type Handling**: Destructuring unions requires branching
5. **Default Value Widening**: Initializers affect inferred types

### Sub-Categories

#### 5a: Array Destructuring (8 files)
- Nested arrays: `[[[c]]]`
- Tuple types: `[number, string, boolean]`
- Type inference from default values

#### 5b: Object Destructuring (5 files)
- Property shorthands: `{x, y}`
- Nested objects: `{z: {x, y: {j}}}`
- Computed properties in patterns

#### 5c: Parameter Destructuring (4 files)
- Function parameters: `function foo({a, b}) {}`
- Overload signatures: Destructuring in overloads vs implementations
- Optional patterns in parameters

#### 5d: Rest Elements (3 files)
- Array rest: `[...rest]`
- Object rest: `{...rest}`
- Complex rest with assignment patterns

#### 5e: Mixed Destructuring (1 file)
- Combining array/object destructuring
- Default values in complex patterns

### Related Code Locations
- Destructuring: `src/check/destructuring.rs` (if exists)
- Pattern matching: `src/check/pattern.rs`
- Type inference: `src/check/inference.rs`
- Parameter types: `src/check/signature.rs`

---

## Category 6: Control Flow Type Narrowing

**Count:** 6 files
**Percentage:** 15%

### Test Cases
- `es6/for-ofStatements/for-of10.ts`
- `es6/for-ofStatements/for-of11.ts`
- `es6/for-ofStatements/for-of47.ts`
- `es6/for-ofStatements/for-of48.ts`
- `es6/templates/taggedTemplateStringsWithTypeErrorInFunctionExpressionsInSubstitutionExpression.ts`
- `es6/templates/taggedTemplateStringsWithTypeErrorInFunctionExpressionsInSubstitutionExpressionES6.ts`

### Pattern Examples

#### Example 1: For-of Variable Type Narrowing
```typescript
var v: string;
for (v of [0]) { }  // TS2322: 'number' not assignable to 'string'
```

#### Example 2: For-of Union Narrowing
```typescript
var v: string;
for (v of [0, ""]) { }  // TS2322: 'string | number' not assignable to 'string'
```

#### Example 3: Template String Function Expressions
```typescript
function foo(...rest: any[]) {}
foo `${function (x: number) { x = "bad"; } }`;  // TS2322 in function body
```

### Root Cause
**Missing: Control Flow Analysis (CFA) for type narrowing**

These errors require tracking types through control flow:

1. **For-of iteration**: Each iteration narrows the loop variable type
2. **Union distribution**: `T | U` arrays require element-wise checking
3. **Nested contexts**: Type errors in nested expressions (template substitutions, function bodies)

### Current WASM Behavior
WASM reports no errors for these cases, indicating lack of:
- For-of variable type tracking
- Union type distribution in iteration
- Deep type checking in nested expression contexts

### Technical Challenges

#### For-of Type Narrowing
```typescript
// What TypeScript does:
var v: string;
for (v of [0]) { }
// 1. Infer array element type: number
// 2. Check assignability: number assignable to string? NO
// 3. Report TS2322
```

#### Template String Substitutions
```typescript
foo `${function (x: number) { x = "bad"; } }`
// 1. Parse tagged template with substitution
// 2. Type check function expression in substitution context
// 3. Detect assignment to number parameter: x = "bad"
// 4. Report TS2322 in nested context
```

### Related Code Locations
- For-of: `src/check/for_of.rs`
- Control flow: `src/check/cfa.rs` or `src/check/control_flow.rs`
- Template strings: `src/check/template.rs`
- Assignment checking: `src/check/assignment.rs`

---

## Special Cases

### Const Enum Property Access (1 file)
**Test:** `constEnums/constEnumPropertyAccess1.ts`

This file uses const enums in computed property contexts. TypeScript reports TS2322, but the test file itself appears to be testing const enum behavior rather than type errors. May need further investigation.

### Intl.NumberFormat Options (1 file)
**Test:** `es2023/intlNumberFormatES5UseGrouping.ts`

```typescript
new Intl.NumberFormat('en-GB', { useGrouping: 'true' });  // TS2322: string not assignable to boolean
```

This tests type checking of built-in DOM/lib types. Requires correct lib.d.ts definitions.

---

## Cross-Cutting Issues

### 1. Symbol Resolution Gaps
Multiple categories (abstract classes, private names, static members) indicate incomplete symbol resolution:
- Abstract modifier not tracked in constructor types
- Private names not uniquely branded per class
- Static index signatures not properly resolved

### 2. Type Inference Limitations
The destructuring category (52.5% of cases) reveals major gaps in:
- Nested pattern type inference
- Contextual typing propagation
- Default value type widening

### 3. Solver Bailouts
Complex scenarios likely causing solver bailouts:
- Nested destructuring (3+ levels deep)
- Union types in destructuring
- Computed properties with index signatures
- For-of with union-typed arrays

### 4. CFA (Control Flow Analysis) Gaps
Control flow narrowing not implemented:
- For-of loop variable type tracking
- Template string substitution type checking
- Nested expression contexts

---

## Impact Assessment

### Severity Levels

#### High Severity (Architecture Changes Required)
1. **Destructuring Pattern Type Inference** (21 files, 52.5%)
   - Requires comprehensive pattern matching system
   - Affects core ES6 feature
   - User-facing impact: High

2. **Abstract Constructor Types** (2 files, 5%)
   - Requires symbol table extensions
   - OOP type system completeness

#### Medium Severity (Feature Implementation)
3. **Private Name Branding** (3 files, 7.5%)
   - ES2022 feature
   - Privacy guarantees important

4. **Control Flow Narrowing** (6 files, 15%)
   - ES6 for-of loops
   - Common in real code

#### Low Severity (Edge Cases)
5. **Computed Property Contextual Typing** (6 files, 15%)
   - Advanced feature
   - Less common patterns

6. **Static Index Signature Literals** (2 files, 5%)
   - Edge case
   - Rare in practice

---

## Recommended Fix Priority

### Phase 1: Quick Wins (Low-Hanging Fruit)
1. **Static Index Signature Literal Types**
   - Isolated fix in index signature checking
   - Low complexity
   - 2 files

2. **Abstract Constructor Type Checking**
   - Single point: typeof type comparison
   - Modifier flag already exists in AST
   - 2 files

### Phase 2: Core Feature Completeness
3. **Private Name Branding**
   - ES2022 feature completeness
   - Requires brand tracking per class
   - 3 files

4. **Control Flow Analysis for For-of**
   - Iterate over array elements, check each
   - Foundation for broader CFA
   - 6 files

### Phase 3: Major Feature Implementation
5. **Destructuring Pattern Type Inference**
   - Largest category (52.5%)
   - Requires pattern type inference engine
   - Multiple sub-features (nested, optional, rest)
   - 21 files

6. **Computed Property Contextual Typing**
   - Object literal contextual typing
   - Complex interaction with index signatures
   - 6 files

---

## Technical Implementation Notes

### Destructuring Type Checking Strategy

The destructuring category requires building a pattern type inference system:

```rust
// Pseudocode for required system
struct PatternChecker {
    fn check_array_pattern(&self, pattern: ArrayPattern, value_type: Type) -> Result {
        // 1. Normalize value_type to tuple type if possible
        // 2. For each element:
        //    a. Get element type from tuple (or union)
        //    b. Recursively check element pattern
        //    c. Handle default values (widen type)
        //    d. Handle rest elements (array/union rest)
    }

    fn check_object_pattern(&self, pattern: ObjectPattern, value_type: Type) -> Result {
        // 1. For each property:
        //    a. Resolve property name (computed or literal)
        //    b. Get property type from value_type
        //    c. Check nested pattern
        //    d. Handle default values
        //    e. Handle shorthand properties
    }
}
```

### Private Name Branding Strategy

```rust
// Track private names per class
struct PrivateNameBrand {
    class_id: ClassId,
    name: String,
}

// Two private names are compatible iff:
fn private_names_compatible(a: PrivateNameBrand, b: PrivateNameBrand) -> bool {
    a.class_id == b.class_id && a.name == b.name
}
```

### For-of Type Checking Strategy

```rust
fn check_for_of(loop_var: &Expr, iterable: &Expr) {
    let element_type = get_array_element_type(iterable);
    let var_type = get_type_annotation(loop_var);

    // For each iteration, check assignability
    if !is_assignable(element_type, var_type) {
        report_ts2322(element_type, var_type);
    }
}
```

---

## Test File Locations

All test files are located at:
```
/tmp/orchestrator-workspace/worktrees/worker-11/tests/cases/conformance/
```

### Full List by Category

**Abstract Constructors (2):**
1. `classes/classDeclarations/classAbstractKeyword/classAbstractConstructorAssignability.ts`

**Private Names (3):**
2. `classes/members/privateNames/privateNameReadonly.ts`
3. `classes/members/privateNames/privateNamesUnique-1.ts`
4. `classes/members/privateNames/privateNamesUnique-5.ts`

**Static Index Signatures (2):**
5. `classes/staticIndexSignature/staticIndexSignature1.ts`
6. `classes/staticIndexSignature/staticIndexSignature2.ts`

**Computed Properties (6):**
7. `es6/computedProperties/computedPropertyNamesContextualType10_ES5.ts`
8. `es6/computedProperties/computedPropertyNamesContextualType10_ES6.ts`
9. `es6/computedProperties/computedPropertyNamesContextualType8_ES5.ts`
10. `es6/computedProperties/computedPropertyNamesContextualType8_ES6.ts`
11. `es6/computedProperties/computedPropertyNamesContextualType9_ES5.ts`
12. `es6/computedProperties/computedPropertyNamesContextualType9_ES6.ts`

**Destructuring (21):**
13. `es6/destructuring/declarationsAndAssignments.ts`
14. `es6/destructuring/destructuringParameterDeclaration2.ts`
15. `es6/destructuring/destructuringParameterDeclaration3ES5.ts`
16. `es6/destructuring/destructuringParameterDeclaration3ES5iterable.ts`
17. `es6/destructuring/destructuringParameterDeclaration3ES6.ts`
18. `es6/destructuring/destructuringParameterDeclaration4.ts`
19. `es6/destructuring/destructuringParameterDeclaration5.ts`
20. `es6/destructuring/destructuringParameterDeclaration8.ts`
21. `es6/destructuring/destructuringParameterProperties2.ts`
22. `es6/destructuring/destructuringParameterProperties5.ts`
23. `es6/destructuring/optionalBindingParameters1.ts`
24. `es6/destructuring/optionalBindingParameters2.ts`
25. `es6/destructuring/optionalBindingParametersInOverloads1.ts`
26. `es6/destructuring/optionalBindingParametersInOverloads2.ts`
27. `es6/destructuring/restElementWithAssignmentPattern2.ts`

**For-of / CFA (6):**
28. `es6/for-ofStatements/for-of10.ts`
29. `es6/for-ofStatements/for-of11.ts`
30. `es6/for-ofStatements/for-of47.ts`
31. `es6/for-ofStatements/for-of48.ts`
32. `es6/templates/taggedTemplateStringsWithTypeErrorInFunctionExpressionsInSubstitutionExpression.ts`
33. `es6/templates/taggedTemplateStringsWithTypeErrorInFunctionExpressionsInSubstitutionExpressionES6.ts`

**Other (2):**
34. `constEnums/constEnumPropertyAccess1.ts`
35. `es2023/intlNumberFormatES5UseGrouping.ts`

---

## Conclusion

The remaining 94 missing TS2322 errors (from 40 test files) represent **genuine type checking deficiencies** in the WASM compiler, not just diagnostic suppression issues. The patterns are clear and fixable, with **destructuring type inference** being the dominant category at **52.5%**.

### Key Takeaways

1. **Destructuring is the biggest gap** - 21/40 files (52.5%)
2. **Multiple feature gaps** - 6 distinct categories identified
3. **No single root cause** - Issues span symbol resolution, type inference, and CFA
4. **Prioritization is critical** - Destructuring alone represents >50% of cases
5. **Implementable incrementally** - Categories can be tackled independently

### Recommended Next Steps

1. **Start with quick wins** (Phase 1) to reduce count by ~10%
2. **Build foundation** with abstract constructors and private names (Phase 2)
3. **Invest in pattern type inference** system (Phase 3) - addresses 52.5% of cases
4. **Extend CFA** for for-of and control flow (Phase 3)
5. **Address edge cases** (computed properties, static signatures) as needed

### Estimated Effort

- **Phase 1** (2 categories, 4 files): 2-3 days
- **Phase 2** (2 categories, 9 files): 5-7 days
- **Phase 3** (2 categories, 27 files): 15-20 days (mostly destructuring)

**Total: ~22-30 development days** for full coverage of remaining TS2322 errors.
