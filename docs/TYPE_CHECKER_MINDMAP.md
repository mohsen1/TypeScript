# TypeScript Type Checker Mind Map

```
                                    ┌─────────────────────────────────────────┐
                                    │         TYPE CHECKER (checker.ts)        │
                                    │            ~45,000 lines                 │
                                    └───────────────────┬─────────────────────┘
                                                        │
            ┌───────────────────────────────────────────┼───────────────────────────────────────────┐
            │                                           │                                           │
            ▼                                           ▼                                           ▼
   ┌────────────────┐                         ┌────────────────┐                         ┌────────────────┐
   │   TYPE SYSTEM  │                         │  TYPE CHECKING │                         │ TYPE INFERENCE │
   └───────┬────────┘                         └───────┬────────┘                         └───────┬────────┘
           │                                          │                                          │
           │                                          │                                          │
```

---

## 1. TYPE SYSTEM

```
TYPE UNIVERSE
│
├── PRIMITIVE TYPES
│   ├── string        ─── "hello" (literal)
│   ├── number        ─── 42, 3.14 (literals)
│   ├── boolean       ─── true, false (literals)
│   ├── bigint        ─── 100n (literal)
│   ├── symbol        ─── unique symbol
│   ├── null          ─── only null
│   ├── undefined     ─── only undefined
│   ├── void          ─── undefined or no return
│   ├── never         ─── ⊥ bottom (empty set)
│   └── unknown       ─── ⊤ top (universal set)
│
├── COMPOUND TYPES
│   ├── Union         ─── A | B (either A or B)
│   ├── Intersection  ─── A & B (both A and B)
│   └── Tuple         ─── [A, B, C] (ordered)
│
├── OBJECT TYPES
│   ├── Interface     ─── { x: T, y: U }
│   ├── Class         ─── class Foo { }
│   ├── Array         ─── T[] or Array<T>
│   ├── Function      ─── (x: T) => U
│   ├── Constructor   ─── new (x: T) => U
│   └── Object literal
│
├── PARAMETRIC TYPES (Generics)
│   ├── Type Parameter    ─── T, K extends keyof T
│   ├── Generic Type      ─── Array<T>, Map<K,V>
│   ├── Mapped Type       ─── { [K in keyof T]: T[K] }
│   ├── Conditional Type  ─── T extends U ? X : Y
│   └── Template Literal  ─── `hello ${T}`
│
└── TYPE OPERATORS
    ├── keyof T       ─── union of property keys
    ├── T[K]          ─── indexed access type
    ├── typeof x      ─── type of value
    ├── infer R       ─── infer type in conditional
    └── readonly T    ─── immutable version
```

---

## 2. TYPE FLAGS (Discriminators)

```
TypeFlags (bitmask)
│
├── PRIMITIVES
│   ├── Any            = 1 << 0    (any)
│   ├── Unknown        = 1 << 1    (unknown)
│   ├── String         = 1 << 2    (string)
│   ├── Number         = 1 << 3    (number)
│   ├── Boolean        = 1 << 4    (boolean)
│   ├── BigInt         = 1 << 6    (bigint)
│   ├── ESSymbol       = 1 << 12   (symbol)
│   ├── Void           = 1 << 14   (void)
│   ├── Undefined      = 1 << 15   (undefined)
│   ├── Null           = 1 << 16   (null)
│   └── Never          = 1 << 17   (never)
│
├── LITERALS
│   ├── StringLiteral  = 1 << 7    ("hello")
│   ├── NumberLiteral  = 1 << 8    (42)
│   ├── BooleanLiteral = 1 << 9    (true/false)
│   └── BigIntLiteral  = 1 << 11   (100n)
│
├── COMPOUND
│   ├── Object         = 1 << 19   (object type)
│   ├── Union          = 1 << 20   (A | B)
│   └── Intersection   = 1 << 21   (A & B)
│
├── PARAMETRIC
│   ├── TypeParameter  = 1 << 18   (T)
│   ├── Index          = 1 << 22   (keyof T)
│   ├── IndexedAccess  = 1 << 23   (T[K])
│   ├── Conditional    = 1 << 24   (T extends U ? X : Y)
│   └── Substitution   = 1 << 25   (substituted type)
│
└── COMPOSITE FLAGS
    ├── Primitive      = String | Number | Boolean | ...
    ├── Literal        = StringLiteral | NumberLiteral | ...
    ├── StringLike     = String | StringLiteral | TemplateLiteral
    ├── NumberLike     = Number | NumberLiteral | Enum
    ├── Narrowable     = Any | Unknown | StructuredOrInstantiable | ...
    └── ...
```

---

## 3. SUBTYPING RULES

```
                    ┌─────────────────────────────────────┐
                    │          SUBTYPING (A <: B)          │
                    │    "A is assignable to B"           │
                    └─────────────────┬───────────────────┘
                                      │
        ┌─────────────────────────────┼─────────────────────────────┐
        │                             │                             │
        ▼                             ▼                             ▼
┌───────────────┐           ┌───────────────┐           ┌───────────────┐
│   IDENTITY    │           │    SPECIAL    │           │  STRUCTURAL   │
│   A <: A      │           │    TYPES      │           │   COMPAT      │
└───────────────┘           └───────┬───────┘           └───────┬───────┘
                                    │                           │
                    ┌───────────────┼───────────────┐           │
                    │               │               │           │
                    ▼               ▼               ▼           │
              never <: T      T <: unknown     T <: any         │
              (bottom)         (top)          (escape)          │
                                                                │
                                    ┌───────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    ▼               ▼               ▼
              ┌─────────┐    ┌──────────┐    ┌──────────┐
              │ OBJECTS │    │  UNIONS  │    │FUNCTIONS │
              └────┬────┘    └────┬─────┘    └────┬─────┘
                   │              │               │
                   ▼              ▼               ▼
              Width+Depth    Distributive    Contravariant
              Subtyping      Checking        Parameters
```

### Subtyping Decision Flow:

```
isTypeAssignableTo(source, target)
│
├── source === target? ───────────────────────────────► YES
│
├── source is never? ─────────────────────────────────► YES (never <: T)
│
├── target is any/unknown? ───────────────────────────► YES (T <: any/unknown)
│
├── target is Union? ─────────────────────────────────► source <: any member?
│
├── source is Union? ─────────────────────────────────► all members <: target?
│
├── both Objects? ────────────────────────────────────► structural comparison
│   ├── All target properties exist in source?
│   ├── Property types compatible? (covariant)
│   ├── Index signatures compatible?
│   └── Call/construct signatures compatible?
│
├── both Functions? ──────────────────────────────────► function comparison
│   ├── Parameter count compatible?
│   ├── Parameter types compatible? (contravariant)
│   └── Return type compatible? (covariant)
│
└── otherwise ────────────────────────────────────────► NO
```

---

## 4. TYPE INFERENCE

```
                        ┌─────────────────────────────────┐
                        │       TYPE INFERENCE            │
                        │    Bidirectional Flow           │
                        └──────────────┬──────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
              ▼                        ▼                        ▼
     ┌────────────────┐      ┌────────────────┐      ┌────────────────┐
     │   SYNTHESIS    │      │    CHECKING    │      │   NARROWING    │
     │   (Bottom-Up)  │      │   (Top-Down)   │      │  (Flow-Based)  │
     └───────┬────────┘      └───────┬────────┘      └───────┬────────┘
             │                       │                       │
             │                       │                       │
```

### 4.1 Synthesis (Bottom-Up)

```
Γ ⊢ e ⇒ τ   (Infer type τ from expression e)

const x = "hello"         ⇒  x: "hello" (literal type)
const y = 42              ⇒  y: 42 (literal type)
const z = x + y           ⇒  z: string (widened)
const arr = [1, 2, 3]     ⇒  arr: number[]
const obj = { a: 1 }      ⇒  obj: { a: number }

function foo() {          ⇒  foo: () => number
    return 42;
}
```

### 4.2 Checking (Top-Down)

```
Γ ⊢ e ⇐ τ   (Check expression e against expected type τ)

const x: string = "hello"     ✓  "hello" <: string
const y: number = "oops"      ✗  "oops" is not <: number

function foo(x: number) {
    return x * 2;             ⇐  expects number return
}
```

### 4.3 Narrowing (Control Flow)

```
                    let x: string | number
                              │
                              ▼
                    ┌─────────────────┐
                    │ typeof x === "string" │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │ true                        │ false
              ▼                             ▼
        x: string                     x: number
```

---

## 5. GENERIC INSTANTIATION

```
                    ┌─────────────────────────────────────┐
                    │      GENERIC INSTANTIATION          │
                    │   Array<T> + T=number → number[]    │
                    └──────────────┬──────────────────────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        │                          │                          │
        ▼                          ▼                          ▼
┌───────────────┐         ┌───────────────┐         ┌───────────────┐
│   EXPLICIT    │         │   INFERENCE   │         │  CONSTRAINTS  │
│ Array<number> │         │   infer T     │         │ T extends U   │
└───────────────┘         └───────────────┘         └───────────────┘
```

### Type Parameter Inference:

```
function identity<T>(x: T): T { return x; }

identity("hello")
│
├── Argument: "hello"
├── Parameter: T
├── Inference: T = "hello"
└── Result: identity<"hello">("hello"): "hello"

function map<T, U>(arr: T[], f: (x: T) => U): U[]

map([1, 2, 3], x => x.toString())
│
├── arr type: number[]     → T = number
├── f parameter: number    → confirms T = number
├── f return: string       → U = string
└── Result: string[]
```

---

## 6. CHECKER STATE & CACHING

```
┌─────────────────────────────────────────────────────────────────┐
│                       CheckerState                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐    ┌─────────────────┐                     │
│  │   TypeArena     │    │  SymbolArena    │                     │
│  │  (all types)    │    │  (all symbols)  │                     │
│  └────────┬────────┘    └────────┬────────┘                     │
│           │                      │                              │
│           │    ┌─────────────────┘                              │
│           │    │                                                │
│           ▼    ▼                                                │
│  ┌─────────────────────────────────────────────────────┐       │
│  │                    CACHES                            │       │
│  ├─────────────────────────────────────────────────────┤       │
│  │  node_types:     HashMap<NodeIndex, TypeId>         │       │
│  │  symbol_types:   HashMap<SymbolId, TypeId>          │       │
│  │  relation_cache: HashMap<(TypeId, TypeId), bool>    │       │
│  └─────────────────────────────────────────────────────┘       │
│                                                                 │
│  ┌─────────────────────────────────────────────────────┐       │
│  │               SINGLETON TYPES                        │       │
│  ├─────────────────────────────────────────────────────┤       │
│  │  any_type, unknown_type, string_type, number_type   │       │
│  │  boolean_type, void_type, null_type, undefined_type │       │
│  │  never_type, true_type, false_type, object_type     │       │
│  └─────────────────────────────────────────────────────┘       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 7. KEY ALGORITHMS

```
┌─────────────────────────────────────────────────────────────────┐
│                     CORE ALGORITHMS                              │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 1. getTypeOfNode(node)                                          │
│    └── Dispatch on node kind → specific type getter             │
│        ├── Literal → create literal type                        │
│        ├── Identifier → look up symbol → get symbol type        │
│        ├── BinaryExpr → check operator, operand types           │
│        ├── CallExpr → get signature, check args, return type    │
│        └── ...                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 2. isTypeRelatedTo(source, target, relation)                    │
│    └── Main subtyping/assignability algorithm                   │
│        ├── Identity check                                       │
│        ├── Any/unknown/never handling                           │
│        ├── Union/intersection distribution                      │
│        ├── Object structural comparison                         │
│        └── Recursion with cycle detection                       │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 3. inferTypeArguments(signature, args)                          │
│    └── Generic type parameter inference                         │
│        ├── Create inference context                             │
│        ├── For each (parameter, argument) pair:                 │
│        │   └── inferTypes(paramType, argType, inferences)       │
│        └── Fix inferred types, apply defaults                   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 4. instantiateType(type, mapper)                                │
│    └── Replace type parameters with concrete types              │
│        ├── TypeParameter → mapper(T) or default                 │
│        ├── Union/Intersection → instantiate each constituent    │
│        ├── Object → instantiate property types                  │
│        └── Conditional → evaluate condition, instantiate branch │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ 5. getNarrowedType(type, node)                                  │
│    └── Apply control flow narrowing                             │
│        ├── typeof narrowing: typeof x === "string"              │
│        ├── instanceof narrowing: x instanceof Foo               │
│        ├── Truthiness narrowing: if (x) { }                     │
│        ├── Equality narrowing: x === "hello"                    │
│        └── Type predicate: if (isFoo(x)) { }                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 8. VARIANCE

```
                    ┌─────────────────────────────────────┐
                    │            VARIANCE                  │
                    │   How subtyping propagates           │
                    └──────────────┬──────────────────────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        │                          │                          │
        ▼                          ▼                          ▼
┌───────────────┐         ┌───────────────┐         ┌───────────────┐
│  COVARIANT    │         │ CONTRAVARIANT │         │  INVARIANT    │
│     (+)       │         │     (-)       │         │    (±)        │
└───────┬───────┘         └───────┬───────┘         └───────┬───────┘
        │                         │                         │
        ▼                         ▼                         ▼
  If A <: B then          If A <: B then            No relation
  F<A> <: F<B>            F<B> <: F<A>              preserved
        │                         │                         │
        │                         │                         │
Examples:                 Examples:                 Examples:
• Array<T> (read)         • Function params         • Array<T> (read+write)
• Promise<T>              • Comparator<T>           • Mutable refs
• Return types
```

### Variance Examples:

```typescript
// COVARIANT (output position)
type Producer<T> = () => T;
// Producer<Dog> <: Producer<Animal>  ✓ (Dog is subtype)

// CONTRAVARIANT (input position)
type Consumer<T> = (x: T) => void;
// Consumer<Animal> <: Consumer<Dog>  ✓ (reversed!)

// INVARIANT (both positions)
type Processor<T> = (x: T) => T;
// No subtype relation between Processor<Dog> and Processor<Animal>
```

---

## 9. RUST IMPLEMENTATION MAPPING

```
TypeScript checker.ts          Rust wasm/src/checker.rs
──────────────────────         ─────────────────────────
Type (interface)        →      enum Type { ... }
TypeFlags (enum)        →      mod type_flags { pub const ... }
ObjectFlags (enum)      →      mod object_flags { pub const ... }
Signature (interface)   →      struct Signature { ... }
Symbol (interface)      →      struct Symbol { ... } (in binder.rs)
Checker (class)         →      struct CheckerState { ... }

getTypeOfNode()         →      get_type_of_node(&mut self, node)
isTypeRelatedTo()       →      is_type_assignable_to(&self, source, target)
instantiateType()       →      (TODO) instantiate_type(&mut self, typ, mapper)
inferTypeArguments()    →      (TODO) infer_type_arguments(...)
```

---

## 10. IMPLEMENTATION PHASES

```
Phase 5.1 ✓ Core Infrastructure
│
├── TypeFlags, ObjectFlags modules
├── TypeId, TypeArena
├── Type enum variants
└── Singleton intrinsic types

Phase 5.2 ✓ CheckerState
│
├── CheckerState struct
├── get_type_of_node()
├── get_type_of_symbol()
├── is_type_assignable_to()
└── Symbol table integration

Phase 5.3 □ Function Types
│
├── Function type representation
├── Parameter inference
├── Return type inference
└── Call expression checking

Phase 5.4 □ Generics
│
├── Type parameter handling
├── Generic instantiation
├── Type argument inference
└── Constraint checking

Phase 5.5 □ Advanced
│
├── Conditional types
├── Mapped types
├── Type narrowing
└── Excess property checks
```

---

## Quick Reference

| Concept | TypeScript | Type Theory |
|---------|------------|-------------|
| `any` | Escape hatch | Dynamic type |
| `unknown` | Safe top | ⊤ (top type) |
| `never` | Impossible | ⊥ (bottom type) |
| `A \| B` | Union | Sum type (A + B) |
| `A & B` | Intersection | Product type (A × B) |
| `<T>` | Generic | Universal quantifier ∀T |
| `keyof T` | Key union | Index type |
| `T[K]` | Indexed access | Projection |
| `T extends U ? X : Y` | Conditional | Type-level if |
