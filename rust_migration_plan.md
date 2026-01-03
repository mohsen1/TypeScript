# Task List

## Status

| Phase | Component      | Status | Tests |
|-------|----------------|--------|-------|
| 0     | Infrastructure | ✅ DONE | - |
| 1     | Utilities      | ✅ DONE | 21 Rust |
| 2     | Scanner        | ✅ DONE | 22 Rust |
| 3     | Parser         | ✅ 98%  | 81 Rust + 19 TS |
| 4     | Binder         | ✅ DONE | 10 TS |
| 5     | Type Checker   | 🟡 85%  | 313 Rust |
| 6     | Emitter        | 🟡 40%  | 35 Rust |
| 7     | Language Svc   | ⬜ 0%   | - |
| 8     | Full Rust      | ⬜ 0%   | - |

---

## Phase 5: Type Checker (IN PROGRESS)

### ✅ Completed (5.1 - 5.29)

All of the following are complete:

- **5.1** Type struct, TypeFlags, ObjectFlags, TypeArena, 14 intrinsic types
- **5.2** CheckerState, type caching, assignability, basic inference
- **5.3** Function types, parameters, return types, optional/rest params
- **5.4** Generic types, type parameters, type arguments, instantiation
- **5.5** Object types, property access, type literals, interfaces
- **5.6** Type narrowing (typeof guards, nullable guards)
- **5.7** Generic call inference, explicit type args, inference from args
- **5.8** instanceof type guards
- **5.9** Control flow type guard analysis
- **5.10** Class types, constructor types, new expressions
- **5.11** Index signature parsing
- **5.12** Index signature type checking
- **5.13** Array and tuple types
- **5.14** Array/tuple assignability
- **5.15** Readonly array/tuple types
- **5.16** Rest and optional tuple elements
- **5.17** Spread in array literals
- **5.18** Conditional type evaluation
- **5.19** Template literal types (basic)
- **5.20** Mapped type evaluation
- **5.21** Infer types in conditional types
- **5.22** Keyof type evaluation (basic)
- **5.23** Keyof object types (full), mapped type instantiation, infer pattern matching
- **5.25** Intersection type simplification
- **5.26** Template literal type instantiation
- **5.27** Union type simplification
- **5.28** Distributive conditional types
- **5.29** Index access on arrays/tuples
- **5.30** Property access on unions (get common property type across all members)
- **5.31** Excess property checks (basic detection of extra properties in fresh object literals)
- **5.32** Index signature support in type literals
- **5.33** Discriminated union narrowing (x.kind === "circle" filters union types)
- **5.34** Switch statement exhaustiveness checking
- **5.35** Exhaustiveness checking diagnostic integration
- **5.36** Contextual typing for object literals
- **5.37** Contextual typing for return statements
- **5.38** Contextual typing for callback parameters
- **5.39** Contextual typing for array literals
- **5.40** Type incompatibility diagnostic details
- **5.41** Promise<T> type structure support
- **5.42** 'this' type in class contexts (basic)
- **5.43** User-defined type predicates (x is Type) parsing
- **5.44** RegExp literal type support
- **5.45** Optional property handling in type relations
- **5.46** Missing property detection
- **5.47** Function parameter mismatch diagnostics
- **5.48** Assignment narrowing in control flow
- **5.49** Assertion functions (asserts x is Type) parsing
- **5.50** Call/construct signature diagnostics
- **5.51** Nested discriminated union narrowing
- **5.52** Definite assignment assertion parsing

### 🚧 In Progress / Next Up

- [x] **Full contextual typing implementation**
    - Infer complete parameter types from context
    - Handle all callback scenarios
    - Contextual typing for object literal properties and methods
    - Contextual typing for array literal elements

- [x] **Complete diagnostics integration**
    - Port error message generation with TypeScript-style templates
    - Match exact TypeScript error codes (diagnostic_codes module)
    - Related information spans (DiagnosticRelatedInformation)
    - format_message() for {0}, {1} placeholder replacement

- [x] **Fix blockers from code review**
    - Scope shadowing bug in type parameter handling
    - Class type recursion - placeholder updated in-place
    - TypeReference resolution returns instance type for classes

### 📋 Remaining Type Checker Features

#### Enums & Overloads
- [x] **5.53** Numeric/string/const enum type checking
- [x] **5.54** Enum reverse mappings
- [x] **5.55** Function overload resolution (selecting correct overload)

#### Class & Member Checking
- [x] **5.56** Private/protected visibility enforcement
- [x] **5.57** Abstract member implementation verification
- [x] **5.58** `override` keyword validation
- [ ] **5.59** Static blocks and auto-accessors (`accessor` keyword) - BLOCKED: needs parser support

#### Modern Type Features (TS 4.x-5.x)
- [x] **5.60** Variance annotations (`in`/`out` modifiers) - basic extraction, full enforcement TODO
- [x] **5.61** `satisfies` operator type checking
- [ ] **5.62** `const` type parameters (literal inference in generics)
- [ ] **5.63** `NoInfer<T>` utility type
- [ ] **5.64** `using` declarations (disposable resources)
- [ ] **5.65** Decorator metadata type support

#### Advanced Tuple & Template Types
- [ ] **5.66** Variadic tuple types (`[...T, ...U]`)
- [ ] **5.67** Named tuple elements (`[name: string, age: number]`)
- [ ] **5.68** String manipulation types (`Uppercase<T>`, `Lowercase<T>`, etc.)

#### Type Inference & Relations
- [ ] **5.69** Recursive conditional type depth limits
- [ ] **5.70** Circular reference detection
- [ ] **5.71** Covariance/contravariance checking in functions
- [ ] **5.72** Type widening control (`as const`)
- [ ] **5.73** `Awaited<T>` recursive unwrapping
- [ ] **5.74** `ThisType<T>` for object literal methods

#### Type Syntax Interpretation (Parsed but needs checker support)
- [ ] **5.103** `typeof` type operator (get type of runtime expression)
- [ ] **5.104** Mapped type modifiers (`+readonly`, `-readonly`, `+?`, `-?`)
- [ ] **5.105** `unique symbol` type for const symbol declarations
- [ ] **5.106** `this` parameter types (`function foo(this: T, ...)`)
- [ ] **5.107** Abstract construct signatures (`abstract new () => T`)
- [ ] **5.108** Call/construct signatures in type literals (`{ (): void }`, `{ new(): T }`)
- [ ] **5.109** Getter/setter signatures in type literals
- [ ] **5.110** Type imports (`typeof import("module")`)
- [ ] **5.111** Instantiation expressions (`fn<string>` without calling)
- [ ] **5.112** `infer` with `extends` constraints (`infer T extends U`)
- [ ] **5.113** Recursive type alias detection and handling

#### Declaration & Module Features
- [ ] **5.75** Declaration merging (interfaces, namespaces)
- [ ] **5.76** Module augmentation (`declare module`)
- [ ] **5.77** Global augmentation (`declare global`)
- [ ] **5.78** Type-only import/export elision

#### JSDoc Support (for `checkJs`)
- [ ] **5.79** JSDoc type annotations (`@type`, `@param`, `@returns`)
- [ ] **5.80** JSDoc template tags (`@template`)
- [ ] **5.81** JSDoc typedef/callback definitions

#### ECMAScript Runtime Types & Built-ins
- [ ] **5.87** `Symbol` type and well-known symbols (`Symbol.iterator`, `Symbol.asyncIterator`, etc.)
- [ ] **5.88** `BigInt` type checking and literal types
- [ ] **5.89** `WeakRef<T>` and `FinalizationRegistry<T>` types
- [ ] **5.90** Iterator/Generator type inference (`Generator<T, TReturn, TNext>`)
- [ ] **5.91** AsyncIterator/AsyncGenerator types
- [ ] **5.92** `Proxy` and `Reflect` type handling
- [ ] **5.93** `SharedArrayBuffer` and `Atomics` types
- [ ] **5.94** `DataView` and TypedArray types (`Uint8Array`, `Float32Array`, etc.)
- [ ] **5.95** `AggregateError` and error cause chains

#### ES2024-2026 Features
- [ ] **5.96** Import attributes (`import x from "y" with { type: "json" }`)
- [ ] **5.97** `RegExp` `/v` flag (set notation) type support
- [ ] **5.98** Resizable `ArrayBuffer` types
- [ ] **5.99** `Iterator.prototype` methods (`.map`, `.filter`, `.take`, etc.)
- [ ] **5.100** `Set` methods (`.union`, `.intersection`, `.difference`, etc.)
- [ ] **5.101** `Promise.try` type inference
- [ ] **5.102** `Float16Array` typed array

#### Performance & Architecture (from Go lessons)
- [ ] **5.82** Type relation caching (`(source, target) → result` map)
- [ ] **5.83** Flow state recycling (object pooling)
- [ ] **5.84** Apparent type cache
- [ ] **5.85** Awaited type cache
- [ ] **5.86** Literal union base type cache

---

## Phase 6: Emitter (IN PROGRESS)

### 6.1 Printer
- [x] Port AST → text printing logic (basic expressions, statements, declarations)
- [x] Handle formatting and whitespace (indentation, newlines)
- [x] Roundtrip tests (parse → emit → parse) verified
- [ ] Source map generation

### 6.2 Transformers
- [ ] Port downlevel transforms (ES2015 → ES5)
- [ ] Module system transforms (ESM ↔ CJS)
- [ ] JSX transform

### 6.3 Declaration Emit
- [ ] Port `.d.ts` generation
- [ ] Handle visibility and export pruning

---

## Phase 7: Language Service (NOT STARTED)

### 7.1 Completions
- [ ] Port completion entry generation
- [ ] Symbol filtering and ranking

### 7.2 Quick Info / Hover
- [ ] Port display parts generation
- [ ] Type-to-string rendering

### 7.3 Navigation
- [ ] Go to definition
- [ ] Find all references
- [ ] Rename support

---

## Phase 8: Full Rust Mode (NOT STARTED)

### 8.1 Standalone Binary
- [ ] Create native `tsc` binary (no Node.js required)
- [ ] CLI argument parsing in Rust
- [ ] File system abstraction

### 8.2 Performance Optimization
- [ ] Profile and optimize hot paths
- [ ] Implement parallel type checking
- [ ] Memory usage optimization

### 8.3 Compatibility Mode
- [ ] Maintain wasm build for Node.js users
- [ ] Ensure identical behavior between native and wasm builds

---

## Blocked

_None currently_

---

## Deferred

- Experimental syntax support
- Collections (`Map`, `Set`, `MultiMap` equivalents)

## Performance Optimizations (DONE)

- [x] Zero-copy UTF-8 scanner (String instead of Vec<char>)
- [x] String interner with Atom handles
- [ ] SIMD-accelerated whitespace skipping
- [ ] Integrate Interner with Parser (Identifier.escaped_text)
