# TypeScript Type Checker: A Type-Theoretic Analysis

## Overview

The TypeScript type checker (`checker.ts`) is ~45,000 lines of code implementing a sophisticated
type system that combines:
- **Structural subtyping** (not nominal like Java/C#)
- **Bidirectional type inference** (flow-sensitive)
- **Gradual typing** (via `any` and `unknown`)
- **Higher-kinded polymorphism** (generics, mapped types, conditional types)

This document provides a type-theoretic foundation for understanding and implementing the checker.

---

## Part 1: Type Theory Foundations

### 1.1 The Type Universe

TypeScript's type universe can be categorized as:

```
Types (τ)
├── Primitive Types
│   ├── string, number, boolean, bigint, symbol
│   ├── null, undefined, void
│   ├── never (⊥ - bottom type, uninhabited)
│   └── unknown (⊤ - top type, all values)
│
├── Literal Types (singleton types)
│   ├── String literals: "hello", "world"
│   ├── Number literals: 42, 3.14
│   ├── Boolean literals: true, false
│   └── Template literal types: `hello ${string}`
│
├── Compound Types
│   ├── Union (τ₁ | τ₂) - sum type, either/or
│   ├── Intersection (τ₁ & τ₂) - product in disguise, both
│   └── Tuple ([τ₁, τ₂, ...τₙ]) - ordered product
│
├── Object Types
│   ├── Interface/Type literal: { x: τ₁, y: τ₂ }
│   ├── Array: τ[] or Array<τ>
│   ├── Function: (x: τ₁) => τ₂
│   └── Class (nominal-ish via private fields)
│
├── Parametric Types (∀)
│   ├── Generic types: T, Array<T>
│   ├── Mapped types: { [K in keyof T]: ... }
│   └── Conditional types: T extends U ? X : Y
│
└── Type Operators
    ├── keyof T - index type query
    ├── T[K] - indexed access
    ├── typeof x - type query
    └── infer R - type inference in conditionals
```

### 1.2 Subtyping Relation (≤)

TypeScript uses **structural subtyping**: `A ≤ B` if A's structure is compatible with B's.

**Subtyping Rules:**

```
─────────────── (Reflexivity)
    τ ≤ τ

  τ₁ ≤ τ₂    τ₂ ≤ τ₃
─────────────────────── (Transitivity)
       τ₁ ≤ τ₃

─────────────── (Top)
   τ ≤ unknown

─────────────── (Bottom)
  never ≤ τ

     τ ≤ τ₁         τ ≤ τ₂
─────────────── ─────────────── (Union)
  τ ≤ τ₁ | τ₂    τ ≤ τ₁ | τ₂

  τ ≤ τ₁    τ ≤ τ₂
─────────────────── (Intersection Introduction)
   τ ≤ τ₁ & τ₂

  τ₁ ≤ τ    τ₂ ≤ τ
─────────────────── (Intersection Elimination)
   τ₁ & τ₂ ≤ τ
```

**Object Subtyping (Width & Depth):**

```
For object types { p₁: τ₁, ..., pₙ: τₙ }:

Width subtyping: More properties = subtype
  { x: number, y: string } ≤ { x: number }

Depth subtyping: Covariant property types
  { x: "hello" } ≤ { x: string }
```

### 1.3 Variance

Variance describes how subtyping of compound types relates to subtyping of their components:

```
Covariance (+):     A ≤ B  ⟹  F<A> ≤ F<B>
Contravariance (-): A ≤ B  ⟹  F<B> ≤ F<A>
Invariance (0):     A ≤ B  ⟹  no relationship
Bivariance (±):     A ≤ B  ⟹  F<A> ≤ F<B> AND F<B> ≤ F<A>
```

**TypeScript Variance Rules:**

| Position | Variance | Example |
|----------|----------|---------|
| Return type | Covariant | `() => Dog` ≤ `() => Animal` |
| Parameter type | Contravariant* | `(a: Animal) => void` ≤ `(d: Dog) => void` |
| Property (read) | Covariant | `{ x: Dog }` ≤ `{ x: Animal }` |
| Property (write) | Contravariant | Writing requires invariance |
| Mutable property | Invariant | Actually bivariant for compatibility |
| Array element | Covariant** | `Dog[]` ≤ `Animal[]` (unsound!) |

*TypeScript uses bivariance for function parameters in some modes for practical reasons.
**This is technically unsound but practical.

### 1.4 Type Inference (Hindley-Milner Extended)

TypeScript uses **bidirectional type checking** with **local type inference**:

```
Γ ⊢ e ⇒ τ    (Inference/Synthesis mode - compute type from expression)
Γ ⊢ e ⇐ τ    (Checking mode - verify expression has expected type)
```

**Key inference rules:**

```
─────────────────────── (Var)
Γ, x: τ ⊢ x ⇒ τ

Γ ⊢ e₁ ⇒ (τ₁) → τ₂    Γ ⊢ e₂ ⇐ τ₁
───────────────────────────────────── (App)
         Γ ⊢ e₁(e₂) ⇒ τ₂

    Γ, x: τ₁ ⊢ e ⇒ τ₂
─────────────────────────── (Abs-Infer)
Γ ⊢ (x: τ₁) => e ⇒ τ₁ → τ₂

Γ ⊢ e ⇒ τ'    τ' ≤ τ
─────────────────────── (Sub)
     Γ ⊢ e ⇐ τ
```

**Contextual Typing:**
```typescript
// Type flows from context to lambda
const f: (x: number) => number = x => x + 1;
//                                 ↑ x inferred as number from context
```

---

## Part 2: TypeScript Checker Architecture

### 2.1 Core Data Structures

```
┌─────────────────────────────────────────────────────────────┐
│                        Type                                  │
├─────────────────────────────────────────────────────────────┤
│ flags: TypeFlags        // What kind of type                │
│ id: TypeId              // Unique identifier                │
│ symbol?: Symbol         // Associated declaration           │
│ aliasSymbol?: Symbol    // If this is a type alias          │
│ aliasTypeArguments?: Type[]                                 │
└─────────────────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────────────────┐
│                  Specialized Types                           │
├─────────────────────────────────────────────────────────────┤
│ ObjectType              │ UnionType        │ IntersectionType│
│ ├─ properties: Symbol[] │ ├─ types: Type[] │ ├─ types: Type[]│
│ ├─ callSignatures       │ └─ origin        │ └─ resolved     │
│ ├─ constructSignatures  │                  │                 │
│ └─ indexInfos           │                  │                 │
├─────────────────────────┼──────────────────┼─────────────────┤
│ TypeParameter           │ ConditionalType  │ MappedType      │
│ ├─ constraint           │ ├─ checkType     │ ├─ typeParameter│
│ ├─ default              │ ├─ extendsType   │ ├─ constraintType│
│ └─ target               │ ├─ trueType      │ ├─ templateType │
│                         │ └─ falseType     │ └─ modifiers    │
└─────────────────────────┴──────────────────┴─────────────────┘
```

### 2.2 Type Checking Pipeline

```
Source Code
     │
     ▼
┌─────────────┐
│   Parser    │ → AST (syntax tree)
└─────────────┘
     │
     ▼
┌─────────────┐
│   Binder    │ → Symbols (name resolution) ← WE ARE HERE
└─────────────┘
     │
     ▼
┌─────────────┐
│  Checker    │ → Types + Diagnostics
└─────────────┘
     │
     ├─► getTypeOfSymbol(symbol) → Type
     ├─► checkExpression(node) → Type
     ├─► isTypeAssignableTo(source, target) → boolean
     └─► getSignaturesOfType(type, kind) → Signature[]
```

### 2.3 Key Checker Functions (Mental Model)

```typescript
// The checker maintains a type cache
interface TypeChecker {
    // Entry points
    getTypeOfSymbol(symbol: Symbol): Type;
    getTypeAtLocation(node: Node): Type;

    // Expression checking (bidirectional)
    checkExpression(node: Expression, contextualType?: Type): Type;
    checkExpressionCached(node: Expression): Type;

    // Type relationships
    isTypeAssignableTo(source: Type, target: Type): boolean;
    isTypeSubtypeOf(source: Type, target: Type): boolean;
    isTypeIdenticalTo(source: Type, target: Type): boolean;

    // Type resolution
    getApparentType(type: Type): Type;  // Follow type aliases, get members
    getBaseTypes(type: InterfaceType): Type[];
    getPropertiesOfType(type: Type): Symbol[];

    // Inference
    inferTypeArguments(signature: Signature, args: Expression[]): Type[];
    getContextualType(node: Expression): Type | undefined;

    // Narrowing (flow analysis)
    getFlowTypeOfReference(reference: Node, type: Type, flow: FlowNode): Type;
}
```

---

## Part 3: Core Algorithms

### 3.1 Type Relationship Checking

The heart of the checker is `isTypeRelatedTo`:

```
isTypeRelatedTo(source, target, relation)
├── Quick checks (identity, any, never, unknown)
├── Union source: ALL members must relate to target
├── Union target: source must relate to SOME member
├── Intersection source: SOME member must relate to target
├── Intersection target: source must relate to ALL members
├── Object types: structural comparison
│   ├── Check properties (with variance)
│   ├── Check call signatures
│   ├── Check construct signatures
│   └── Check index signatures
├── Generic instantiation: instantiate and recurse
└── Conditional types: special distribution rules
```

**Relation Types:**
- `Assignable`: Can assign source to target? (most common)
- `Subtype`: Is source a proper subtype?
- `Identity`: Are types structurally identical?
- `Comparable`: Can compare with == or ===?

### 3.2 Type Instantiation

Generics require instantiation - replacing type parameters with arguments:

```
instantiateType(type, mapper)
├── If type is TypeParameter, return mapper(type)
├── If type is ObjectType with type parameters:
│   └── Create new type with instantiated members
├── If type is UnionType:
│   └── Union of instantiated member types
├── If type is ConditionalType:
│   └── May need to distribute over unions
└── Cache results to avoid infinite recursion
```

**Example:**
```typescript
type Box<T> = { value: T };
// instantiateType(Box<T>, { T → number }) = { value: number }
```

### 3.3 Type Inference

For generic function calls:

```typescript
function identity<T>(x: T): T { return x; }
identity(42);  // Infer T = number
```

**Inference Algorithm:**
```
inferTypeArguments(signature, args, contextualType?)
│
├── Create inference context with type parameters
│
├── For each (parameter, argument) pair:
│   └── inferTypes(parameterType, argumentType, context)
│       ├── If parameterType is TypeParameter:
│       │   └── Add argumentType as inference candidate
│       ├── If parameterType is ObjectType:
│       │   └── Recurse on matching properties
│       ├── If parameterType is UnionType:
│       │   └── Try each constituent
│       └── Handle covariant/contravariant positions differently
│
├── For return type with contextual type:
│   └── inferTypes(returnType, contextualType, context, contravariant)
│
└── getInferredTypes(context)
    └── For each type parameter, pick best candidate
        ├── Union of covariant candidates
        └── Intersection of contravariant candidates
```

### 3.4 Control Flow Analysis (Narrowing)

TypeScript narrows types based on control flow:

```typescript
function foo(x: string | number) {
    if (typeof x === "string") {
        // x is narrowed to string here
        x.toUpperCase();
    } else {
        // x is narrowed to number here
        x.toFixed();
    }
}
```

**Narrowing Algorithm:**
```
getFlowTypeOfReference(reference, declaredType, flowNode)
│
├── Walk backwards through flow graph
│
├── At FlowCondition nodes:
│   ├── typeof x === "string" → narrow to string
│   ├── x instanceof C → narrow to C
│   ├── x === null → narrow to null
│   ├── "prop" in x → narrow to types with prop
│   └── x (truthiness) → exclude null/undefined
│
├── At FlowAssignment nodes:
│   └── Use assigned type
│
├── At FlowBranchLabel (merging branches):
│   └── Union of types from all antecedents
│
└── Cache results keyed by (reference, flowNode)
```

---

## Part 4: Advanced Type Features

### 4.1 Conditional Types

```typescript
type IsString<T> = T extends string ? true : false;
```

**Evaluation Rules:**
```
T extends U ? X : Y

1. If T is union: distribute
   (A | B) extends U ? X : Y  →  (A extends U ? X : Y) | (B extends U ? X : Y)

2. If T is type parameter: defer (return conditional type)

3. If T ≤ U: return X

4. If T and U are disjoint: return Y

5. Otherwise: return union X | Y or defer
```

### 4.2 Mapped Types

```typescript
type Readonly<T> = { readonly [K in keyof T]: T[K] };
```

**Resolution:**
```
{ [K in keyof T]: F<K, T[K]> }

1. Get keys of T (keyof T)
2. For each key K:
   a. Get property type T[K]
   b. Apply template F<K, T[K]>
3. Construct new object type with transformed properties
```

### 4.3 Template Literal Types

```typescript
type EventName<T extends string> = `on${Capitalize<T>}`;
// EventName<"click"> = "onClick"
```

**String manipulation types:**
- `Uppercase<S>`, `Lowercase<S>`
- `Capitalize<S>`, `Uncapitalize<S>`

---

## Part 5: Implementation Strategy

### 5.1 Recommended Order

```
Phase 5.1: Core Types
├── TypeFlags enum
├── Type struct with variants
├── TypeArena for allocation
└── Basic type constructors

Phase 5.2: Primitive Type Checking
├── Literal types
├── String, number, boolean
├── null, undefined, void
├── any, unknown, never
└── Simple assignability

Phase 5.3: Object Types
├── Property lookup
├── Method signatures
├── Index signatures
└── Structural comparison

Phase 5.4: Union and Intersection
├── Union type creation
├── Intersection type creation
├── Reduction/simplification
└── Distributive checks

Phase 5.5: Generics
├── Type parameters
├── Type instantiation
├── Generic constraints
└── Basic inference

Phase 5.6: Advanced Features
├── Conditional types
├── Mapped types
├── Template literals
└── Recursive types

Phase 5.7: Control Flow
├── Type narrowing
├── Type guards
├── Assertion functions
└── Discriminated unions
```

### 5.2 Key Invariants to Maintain

1. **Type identity**: Same structure = same type (for caching)
2. **Symmetry**: `isAssignableTo(A, B)` same result with same inputs
3. **Termination**: Handle recursive types without infinite loops
4. **Soundness trade-offs**: Document where unsound for practicality

### 5.3 Critical Caches

```rust
struct TypeChecker {
    // Avoid recomputing types
    symbol_type_cache: HashMap<SymbolId, TypeId>,

    // Avoid recomputing relationships
    assignable_cache: HashMap<(TypeId, TypeId), bool>,
    subtype_cache: HashMap<(TypeId, TypeId), bool>,

    // Avoid recomputing apparent types
    apparent_type_cache: HashMap<TypeId, TypeId>,

    // Instantiation cache
    instantiation_cache: HashMap<(TypeId, TypeMapper), TypeId>,
}
```

---

## Part 6: Complexity Considerations

### 6.1 Known Hard Problems

1. **Recursive types**: Need cycle detection
2. **Distributive conditionals**: Can explode combinatorially
3. **Inference with multiple candidates**: Choosing best match
4. **Excess property checking**: Only at certain positions
5. **Contextual typing flow**: Bidirectional, context-sensitive

### 6.2 TypeScript-Specific Quirks

1. **Bivariant function parameters**: Unsound but intentional
2. **Covariant arrays**: Unsound but matches JavaScript
3. **`any` is both top and bottom**: Opt-out of type checking
4. **Freshness**: Object literals get stricter checking
5. **Declaration merging**: Affects symbol-to-type resolution

---

## References

1. Pierce, B. "Types and Programming Languages" (TAPL)
2. TypeScript Handbook: https://www.typescriptlang.org/docs/handbook/
3. TypeScript Deep Dive: https://basarat.gitbook.io/typescript/
4. Anders Hejlsberg's talks on TypeScript type system
5. TypeScript source: `src/compiler/checker.ts`
