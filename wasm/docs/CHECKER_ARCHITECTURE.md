# Type Checker Architecture Design Document

## Overview

This document describes the architecture for the Rust implementation of TypeScript's type checker. The type checker is the most complex component of the compiler, representing approximately 50% of the total complexity.

## Goals

1. **Exact Compatibility**: Error codes, messages, and spans must match TypeScript exactly
2. **Incrementality**: Support for watch mode and incremental compilation from day one
3. **Performance**: At least parity with TypeScript, with potential for 2-3x improvement
4. **Maintainability**: Clean separation of concerns, well-documented algorithms

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                          CheckerState                               │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                 │
│  │ TypeArena   │  │SymbolArena  │  │ DiagArena   │                 │
│  │ (all types) │  │ (binder)    │  │ (errors)    │                 │
│  └─────────────┘  └─────────────┘  └─────────────┘                 │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Type Operations                           │   │
│  ├──────────────────┬──────────────────┬──────────────────────┤   │
│  │ isTypeRelatedTo  │ instantiateType  │ getTypeOfSymbol      │   │
│  │ (subtyping)      │ (generics)       │ (lazy resolution)    │   │
│  └──────────────────┴──────────────────┴──────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Inference Engine                          │   │
│  ├──────────────────┬──────────────────┬──────────────────────┤   │
│  │ InferenceContext │ Constraint Solver│ Candidate Collection │   │
│  └──────────────────┴──────────────────┴──────────────────────┘   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                  Control Flow Analysis                       │   │
│  ├──────────────────┬──────────────────┬──────────────────────┤   │
│  │ Type Narrowing   │ Discriminated    │ Assertion Functions  │   │
│  │                  │ Unions           │                       │   │
│  └──────────────────┴──────────────────┴──────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Core Data Structures

### Type Arena

The type arena is the central storage for all types. Types are referenced by `TypeId` indices.

```rust
pub struct TypeArena {
    types: Vec<Type>,
    // Singleton intrinsic types (created once)
    any_type: TypeId,
    unknown_type: TypeId,
    string_type: TypeId,
    number_type: TypeId,
    boolean_type: TypeId,
    void_type: TypeId,
    undefined_type: TypeId,
    null_type: TypeId,
    never_type: TypeId,
    // ... more intrinsics
}
```

### Type Enum

Types are represented as an enum with variants for each type kind:

```rust
pub enum Type {
    Intrinsic(IntrinsicType),      // string, number, boolean, etc.
    Literal(LiteralType),           // "hello", 42, true
    Object(ObjectType),             // { x: number }
    Reference(TypeReference),       // Array<T>, Map<K, V>
    Union(UnionType),               // A | B
    Intersection(IntersectionType), // A & B
    TypeParameter(TypeParameter),   // T in <T>
    Conditional(ConditionalType),   // T extends U ? X : Y
    IndexedAccess(IndexedAccessType), // T[K]
    Index(IndexType),               // keyof T
    Mapped(MappedType),            // { [K in T]: U }
    TemplateLiteral(TemplateLiteralType), // `hello${T}`
    Substitution(SubstitutionType), // Internal: base + substitute
    Tuple(TupleType),              // [T, U, V]
    Signature(SignatureType),       // (x: T) => U
}
```

## Core Algorithms

### 1. isTypeRelatedTo (Structural Subtyping)

The core algorithm for determining if one type is assignable to another.

```
isTypeRelatedTo(source: Type, target: Type, relation: Relation) -> bool

Relation = Assignable | Comparable | Identical | Subtype

Algorithm:
1. Identity check: if source === target, return true
2. Check type flags for early exits (any, unknown, never)
3. Handle special cases:
   - source is never -> always true
   - target is any/unknown -> usually true
   - source is any -> depends on strictness
4. Structural comparison:
   - For objects: compare members recursively
   - For unions: source must match at least one target constituent
   - For intersections: source must match all constituents
   - For generics: check constraints, handle variance
5. Cache results for performance
```

### 2. Type Inference

Type inference collects constraints and solves for type parameters.

```
InferenceContext {
    inferences: Map<TypeParameter, Inference>,
    priority: InferencePriority,
    flags: InferenceFlags,
}

Inference {
    type_parameter: TypeId,
    candidates: Vec<TypeId>,        // Covariant positions
    contra_candidates: Vec<TypeId>, // Contravariant positions
    inferred_type: Option<TypeId>,
}

Algorithm:
1. Create inference context for call/instantiation
2. Walk arguments and parameters, inferring at each position
3. For covariant positions (return types): collect candidates
4. For contravariant positions (params): collect contra-candidates
5. Fix each type parameter:
   - If only covariant: union of candidates
   - If only contravariant: intersection of contra-candidates
   - If both: use covariant if more specific
6. Handle circular references with priority levels
```

### 3. Control Flow Type Narrowing

TypeScript narrows types based on control flow.

```
getFlowTypeOfReference(reference: Node, initialType: Type) -> Type

Algorithm:
1. Get the flow node for this reference
2. Walk backwards through flow graph
3. At each flow node:
   - Assignment: type becomes assigned type
   - Condition (true): narrow by condition (typeof, instanceof, in)
   - Condition (false): narrow by negation
   - BranchLabel: union of antecedent types
   - LoopLabel: fixed-point iteration
4. Handle special narrowing:
   - typeof x === "string" -> string
   - x instanceof Array -> Array<unknown>
   - "prop" in x -> { prop: unknown }
   - x != null -> NonNullable<T>
   - Discriminated union: x.kind === "a" -> specific variant
```

## Implementation Phases

### Phase 1: Primitive Type Checking (Current)
- [x] Type flags and intrinsic types
- [x] Type arena with singletons
- [x] Basic type creation (union, intersection)
- [ ] Primitive assignability (string to string, etc.)
- [ ] Literal type widening

### Phase 2: Object Type Checking
- [ ] Property lookup
- [ ] Index signatures
- [ ] Call/construct signatures
- [ ] Excess property checks
- [ ] Optional properties

### Phase 3: Generic Instantiation
- [ ] Type parameter constraints
- [ ] Generic type instantiation
- [ ] Default type parameters
- [ ] Variance markers

### Phase 4: Type Inference
- [ ] Inference context creation
- [ ] Candidate collection
- [ ] Constraint solving
- [ ] Contextual typing

### Phase 5: Control Flow Analysis
- [ ] Type narrowing in conditionals
- [ ] Discriminated unions
- [ ] Assertion functions
- [ ] Exhaustiveness checking

### Phase 6: Advanced Types
- [ ] Conditional types
- [ ] Mapped types
- [ ] Template literal types
- [ ] Recursive type references

## Key Functions to Port

From TypeScript's checker.ts, these are the critical functions:

```
// Core type relationships
isTypeRelatedTo(source, target, relation, errorNode?)
checkTypeRelatedTo(source, target, relation, errorNode?, message?)
isAssignableTo(source, target)
isComparableTo(source, target)

// Type inference
inferTypes(context, source, target, flags)
getInferredType(context, typeParameter)
inferFromTypes(source, target, inferenceContext)

// Type instantiation
instantiateType(type, mapper)
instantiateTypeWithoutDepthLimit(type, mapper)
createTypeMapper(sources, targets)

// Symbol resolution
getTypeOfSymbol(symbol)
getTypeOfVariableOrParameterOrProperty(symbol)
getDeclaredTypeOfSymbol(symbol)

// Expression/statement checking
checkExpression(node, contextualType?)
getTypeOfExpression(node)
checkStatement(node)

// Control flow
getFlowTypeOfReference(reference, declaredType)
narrowType(type, expr, assumeTrue)
```

## Error Handling

Diagnostics are accumulated during type checking:

```rust
pub struct Diagnostic {
    pub file: SourceFileId,
    pub start: u32,
    pub length: u32,
    pub message_text: String,
    pub code: u32,
    pub category: DiagnosticCategory,
    pub related_information: Vec<DiagnosticRelatedInformation>,
}
```

Error codes must match TypeScript exactly for tooling compatibility.

## Caching Strategy

The checker uses extensive caching:

1. **Type cache**: Computed types for nodes/symbols
2. **Relation cache**: Results of isTypeRelatedTo
3. **Inference cache**: Inferred types for type parameters
4. **Flow cache**: Narrowed types at flow nodes

Cache invalidation for incremental:
- On source file change, invalidate affected caches
- Symbol table changes trigger type recomputation
- Use dependency tracking for minimal invalidation

## Testing Strategy

1. **Unit tests**: Individual type operations
2. **Baseline tests**: Compare output with TypeScript
3. **Differential tests**: Run both checkers, compare diagnostics
4. **Performance tests**: Track regression in check time

## Open Questions

1. **Recursion limits**: How to handle deeply recursive types?
2. **Memory limits**: Large projects may need streaming/paging
3. **Error recovery**: How to continue checking after errors?
4. **Incrementality granularity**: File-level or finer?

## References

- TypeScript checker.ts: ~50,000 lines of type checking logic
- TypeScript types.ts: Type definitions and flags
- [TypeScript Deep Dive - Type System](https://basarat.gitbook.io/typescript/)
- [Type Inference Paper](https://www.cs.cmu.edu/~rwh/papers/type-inference/toplas.pdf)

---

*Last updated: January 2026*
