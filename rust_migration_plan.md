# Task List

## Status

| Phase | Component      | Status | Tests |
|-------|----------------|--------|-------|
| 0     | Infrastructure | ✅ DONE | - |
| 1     | Utilities      | ✅ DONE | 21 Rust |
| 2     | Scanner        | ✅ DONE | 22 Rust |
| 3     | Parser         | ✅ 98%  | 81 Rust + 19 TS |
| 4     | Binder         | ✅ DONE | 10 TS |
| 5     | Type Checker   | 🟡 96%  | 367 Rust |
| 6     | Emitter        | 🟡 40%  | 38 Rust |
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
- **5.24** Indexed access type resolution (`T[K]`) - lookup property type by key type
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
- [x] **5.62** `const` type parameters (literal inference in generics)
- [x] **5.63** `NoInfer<T>` utility type
- [ ] **5.64** `using` declarations (disposable resources)
- [ ] **5.65** Decorator metadata type support

#### Advanced Tuple & Template Types
- [x] **5.66** Variadic tuple types (`[...T, ...U]`)
- [x] **5.67** Named tuple elements (`[name: string, age: number]`)
- [x] **5.68** String manipulation types (`Uppercase<T>`, `Lowercase<T>`, etc.)

#### Type Inference & Relations
- [x] **5.69** Recursive conditional type depth limits
- [x] **5.70** Circular reference detection
- [x] **5.71** Covariance/contravariance checking in functions (basic: bivariance + variance modifiers)
- [x] **5.72** Type widening control (`as const`)
- [x] **5.73** `Awaited<T>` recursive unwrapping
- [x] **5.74** `ThisType<T>` for object literal methods

#### Type Syntax Interpretation (Parsed but needs checker support)
- [x] **5.103** `typeof` type operator (get type of runtime expression)
- [x] **5.104** Mapped type modifiers (`+readonly`, `-readonly`, `+?`, `-?`)
- [x] **5.105** `unique symbol` type for const symbol declarations
- [x] **5.106** `this` parameter types (`function foo(this: T, ...)`)
- [ ] **5.107** Abstract construct signatures (`abstract new () => T`)
- [x] **5.108** Call/construct signatures in type literals (`{ (): void }`, `{ new(): T }`)
- [ ] **5.109** Getter/setter signatures in type literals
- [ ] **5.110** Type imports (`typeof import("module")`)
- [ ] **5.111** Instantiation expressions (`fn<string>` without calling)
- [x] **5.112** `infer` with `extends` constraints (`infer T extends U`)
- [ ] **5.113** Recursive type alias detection and handling

#### Advanced Narrowing & Control Flow (from test patterns)
- [ ] **5.133** Narrowing by boolean comparison (`if (x === true)`, `if (!x)`)
  - **Tests:** `narrowByBooleanComparison.ts`, `narrowingTruthyObject.ts`
  - Handle `===`, `!==`, `==`, `!=` with boolean literals
  - Narrow nullable types when compared to `true`/`false`

- [ ] **5.134** Narrowing by `switch(true)` with case expressions
  - **Tests:** `narrowByClauseExpressionInSwitchTrue*.ts` (10 tests)
  - Each case clause with boolean expression narrows the type
  - Handle fallthrough and break semantics

- [ ] **5.135** Narrowing in nested/chained property access
  - **Tests:** `narrowingOfDottedNames.ts`, `narrowingOfQualifiedNames.ts`
  - `if (a.b.c)` should narrow `a.b.c` but not invalidate `a` or `a.b`
  - Track dotted paths in flow analysis

- [ ] **5.136** Narrowing past last assignment in scope
  - **Tests:** `narrowingPastLastAssignment.ts`, `narrowingPastLastAssignmentInModule.ts`
  - After final assignment to variable, maintain narrowed type
  - Handle module-scoped variables differently

- [ ] **5.137** Narrowing constrained type parameters
  - **Tests:** `narrowingConstrainedTypeParameter.ts`
  - `T extends string | number` can be narrowed with typeof
  - Intersection with constraint after narrowing

- [ ] **5.138** Narrowing with `NoInfer<T>` interaction
  - **Tests:** `narrowingNoInfer1.ts`
  - NoInfer wrapper should not prevent narrowing
  - Unwrap NoInfer for flow analysis

- [ ] **5.139** Narrowing in destructuring patterns
  - **Tests:** `narrowingDestructuring.ts`, `arrayDestructuringInSwitch*.ts`
  - Narrow union types when destructuring with type guards
  - Handle rest patterns in narrowing

- [ ] **5.140** Narrowing with `typeof` function check
  - **Tests:** `narrowingTypeofFunction.ts`, `narrowingTypeofObject.ts`
  - `typeof x === "function"` narrows to callable types
  - Handle `typeof x === "object"` (includes null)

- [ ] **5.141** Narrowing `unknown` by type predicate
  - **Tests:** `narrowUnknownByTypePredicate.ts`, `narrowUnknownByTypeofObject.ts`
  - `unknown` can be narrowed to any type via predicate
  - Multiple sequential narrowings accumulate

- [ ] **5.142** Narrowing order independence
  - **Tests:** `narrowingOrderIndependent.ts`
  - `if (a && b)` same result as `if (b && a)` for narrowing
  - Commutative narrowing in logical expressions

- [ ] **5.143** Narrowing union to union
  - **Tests:** `narrowingUnionToUnion.ts`, `narrowingMutualSubtypes.ts`
  - Narrow `A | B | C` to `A | B` via type guard
  - Handle mutual subtype cases

- [ ] **5.144** Narrowing with optional chaining containment
  - **Tests:** `narrowSwitchOptionalChainContainmentEvolvingArrayNoCrash1.ts`
  - `x?.prop` in switch/case narrowing
  - Handle undefined branch from optional chain

#### Circular Reference & Recursion Handling (from test patterns)
- [ ] **5.145** Circular base type detection
  - **Tests:** `circularBaseTypes.ts`, `circularBaseConstraint.ts`
  - Detect `class A extends B` where `B extends A`
  - Report appropriate error without infinite loop

- [ ] **5.146** Circular mapped type constraints
  - **Tests:** `circularConstrainedMappedTypeNoCrash.ts`, `circularMappedTypeConstraint.ts`
  - Mapped type where constraint references the mapped type itself
  - Use placeholder type to break cycle

- [ ] **5.147** Circular contextual return types
  - **Tests:** `circularContextualReturnType.ts`, `circularContextualMappedType.ts`
  - Function return type inferred from context that depends on the function
  - Break cycle with widened type

- [ ] **5.148** Circular accessor annotations
  - **Tests:** `circularAccessorAnnotations.ts`, `circularGetAccessor.ts`
  - Getter return type depends on itself
  - Detect and report circularity

- [ ] **5.149** Infinite expansion termination
  - **Tests:** `checkInfiniteExpansionTermination.ts`, `checkInfiniteExpansionTermination2.ts`
  - Generic type that expands infinitely when instantiated
  - Use depth counter and return error type

- [ ] **5.150** Circular instantiation expression handling
  - **Tests:** `circularInstantiationExpression.ts`
  - `const x = f<typeof x>` creates circular reference
  - Detect during instantiation and report error

- [ ] **5.151** Circular conditionals simplification without crash
  - **Tests:** `circularlySimplifyingConditionalTypesNoCrash.ts`
  - Conditional type that references itself in branches
  - Return deferred type instead of infinite recursion

- [ ] **5.152** Circular type arguments local and outer scope
  - **Tests:** `circularTypeArgumentsLocalAndOuterNoCrash1.ts`
  - Type argument references outer scope type with same name
  - Proper scope resolution to avoid false circularity

#### Best Common Type & Type Inference (from test patterns)
- [ ] **5.153** Best common type for return statements
  - **Tests:** `bestCommonTypeReturnStatement.ts`, `bestChoiceType.ts`
  - Multiple return statements with different types
  - Find common supertype or union

- [ ] **5.154** Best common type with optional properties
  - **Tests:** `bestCommonTypeWithOptionalProperties.ts`
  - Object literals with different optional properties
  - Merge optional properties in best common type

- [ ] **5.155** Best common type with contextual typing
  - **Tests:** `bestCommonTypeWithContextualTyping.ts`
  - Use contextual type to guide best common type selection
  - Prefer contextual type over computed common type

- [ ] **5.156** Array literal type inference edge cases
  - **Tests:** `arrayLiteralTypeInference.ts`, `arrayBestCommonTypes.ts`
  - Mixed literals `[1, "a", true]` infer `(number | string | boolean)[]`
  - Handle spread in inference

- [ ] **5.157** Contextual typing for callbacks preservation
  - **Tests:** `callbacksDontShareTypes.ts`, `callbackArgsDifferByOptionality.ts`
  - Each callback gets fresh contextual type
  - Don't share mutable inference state between callbacks

- [ ] **5.158** Cached contextual types
  - **Tests:** `cachedContextualTypes.ts`
  - Cache contextual type computation per expression
  - Invalidate on type parameter instantiation

#### Assignment Compatibility (from test patterns)
- [ ] **5.159** Assignment compatibility with overloads
  - **Tests:** `assignmentCompatWithOverloads.ts`
  - Function with overloads assignable to single signature
  - Match against each overload

- [ ] **5.160** Assignment compatibility for constrained type parameters
  - **Tests:** `assignmentCompatibilityForConstrainedTypeParameters.ts`
  - `T extends U` constraints in assignability
  - Constraint satisfaction checking

- [ ] **5.161** Assignment to expanding array type
  - **Tests:** `assignmentToExpandingArrayType.ts`
  - Array that grows in type with each push
  - Handle evolving array types

- [ ] **5.162** Assignment nested in literals
  - **Tests:** `assignmentNestedInLiterals.ts`
  - `{ x: a = 1 }` default value in destructuring
  - Type check assignment within patterns

- [ ] **5.163** Assignment compatibility with index signatures
  - **Tests:** `assignmentCompatInterfaceWithStringIndexSignature.ts`
  - Object with properties assignable to index signature type
  - All properties must be assignable to index type

#### Class & Inheritance Checking (from test patterns)
- [ ] **5.164** Class extends null
  - **Tests:** `classExtendsNull.ts`, `classExtendsNull2.ts`, `classExtendsNull3.ts`
  - `class C extends null {}` valid ES6 pattern
  - Constructor must return object or call super()

- [ ] **5.165** Class extends interface (error)
  - **Tests:** `classExtendsInterface.ts`, `classExtendsInterface_not.ts`
  - Cannot extend interface (must use implements)
  - Appropriate error message

- [ ] **5.166** Class implements class
  - **Tests:** `classImplementsClass*.ts` (7 tests)
  - Class can implement another class (use as interface)
  - Check structural compatibility

- [ ] **5.167** Inherited property checking
  - **Tests:** `checkInheritedProperty.ts`, `nonConflictingRecursiveBaseTypeMembers.ts`
  - Override must be assignable to base
  - Handle recursive base type member references

- [ ] **5.168** Class side inheritance
  - **Tests:** `classSideInheritance*.ts` (3 tests)
  - Static members inherited from base class
  - Constructor compatibility checking

- [ ] **5.169** Class variance circularity resolution
  - **Tests:** `classVarianceCircularity.ts`, `classVarianceResolveCircularity*.ts`
  - Variance computation may create circularity
  - Break cycle and compute best variance

- [ ] **5.170** Clodule (class + module) type checking
  - **Tests:** `clodule*.ts` (15+ tests)
  - Class merged with namespace
  - Access static members from namespace

#### Covariance/Contravariance Tests (from test patterns)
- [ ] **5.171** Covariant and contravariant inference combined
  - **Tests:** `coAndContraVariantInferences*.ts` (8 tests)
  - Same type parameter in both positions
  - Resolve to invariant or pick best candidate

#### Advanced Overload Resolution (from test patterns)
- [ ] **5.172** Ambiguous overload resolution
  - **Tests:** `ambiguousOverload.ts`, `ambiguousOverloadResolution.ts`
  - Multiple overloads match equally well
  - Report ambiguity error or pick first match

- [ ] **5.173** Overloads added to base signature
  - **Tests:** `addMoreOverloadsToBaseSignature.ts`, `addMoreCallSignaturesToBaseSignature*.ts`
  - Derived interface adds overloads
  - Merge and order overload lists

- [ ] **5.174** Call signature resolution before specialization
  - **Tests:** `callSignaturesShouldBeResolvedBeforeSpecialization.ts`
  - Resolve overload before instantiating type parameters
  - Avoid premature specialization

#### Advanced Object Literal Features (from test patterns)
- [ ] **5.175** Nested excess property checking
  - **Tests:** `nestedExcessPropertyChecking.ts`, `objectLiteralExcessProperties.ts`
  - Excess property check applies recursively
  - Check nested object literals

- [ ] **5.176** Object literal freshness with spread
  - **Tests:** `objectLiteralFreshnessWithSpread.ts`, `nestedFreshLiteral.ts`
  - Spread loses freshness for excess property checks
  - Nested fresh literals maintain freshness

- [ ] **5.177** Ambiguous calls where return types agree
  - **Tests:** `ambiguousCallsWhereReturnTypesAgree.ts`
  - Multiple signatures match, but return same type
  - No error if result is unambiguous

#### Declaration & Module Features (from test patterns)
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
- [x] **5.88** `BigInt` type checking and literal types
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

#### Ambient & Declaration Context (from test patterns)
- [ ] **5.178** Ambient class with extends
  - **Tests:** `ambientClassDeclarationWithExtends.ts`, `ambientModuleWithClassDeclarationWithExtends.ts`
  - `declare class` can extend another class
  - No implementation checking

- [ ] **5.179** Ambient enum element initializers
  - **Tests:** `ambientEnumElementInitializer*.ts` (6 tests)
  - Ambient enums can have computed initializers
  - Must be constant expressions

- [ ] **5.180** Ambient overloads merging
  - **Tests:** `ambientClassMergesOverloadsWithInterface.ts`
  - Ambient class overloads merge with interface
  - Order preserved

- [ ] **5.181** Ambient module reopening
  - **Tests:** `ambientExternalModuleReopen.ts`
  - Multiple `declare module "x"` blocks merge
  - Cross-file augmentation

- [ ] **5.182** Ambient const literals
  - **Tests:** `ambientConstLiterals.ts`
  - `declare const x: "literal"` preserves literal type
  - No widening in ambient context

#### Advanced Inference & Relationships
- [ ] **5.114** Inference priority levels (`InferencePriority` enum for candidate ranking)
  - **TS Source:** `checker.ts:26710` - `inferTypes()`
  - **Priority enum values:** `None=0`, `NakedTypeVariable=1`, `SpeculativeTuple=2`, `SubstituteSource=3`, `HomomorphicMappedType=4`, `PartialHomomorphicMappedType=5`, `MappedTypeConstraint=6`, `ContravariantConditional=7`, `ReturnType=8`, `LiteralKeyof=9`, `NoConstraints=10`, `AlwaysStrict=11`, `MaxValue=12`
  - When multiple inference sites exist for same type parameter, higher priority wins
  - Lower priority candidates are discarded when higher priority candidate found
  - Implement `InferencePriority` enum and track `inferencePriority` in context

- [ ] **5.115** Bidirectional type inference (synthesis → vs checking ←)
  - **Synthesis (→):** Infer type from expression structure bottom-up
    - `checkExpression(node)` without contextual type
    - Produces "apparent" type from literal structure
  - **Checking (←):** Verify expression against expected type top-down
    - `checkExpression(node, contextualType)` with target type
    - Enables lambda parameter inference, object literal inference
  - Both directions must be implemented for full contextual typing
  - **Key insight:** Contextual type flows DOWN, inferred type flows UP

- [ ] **5.116** Contravariant candidate intersection vs covariant candidate union
  - **TS Source:** `checker.ts` - `getInferredType()`
  - When fixing type parameter after inference:
    - **Covariant positions (return types):** Union of all candidates (`A | B | C`)
    - **Contravariant positions (parameters):** Intersection of candidates (`A & B & C`)
  - If both exist:
    - Use covariant if it's more specific (subtype of contravariant result)
    - Otherwise use contravariant result
  - Implement `inference.candidates` (covariant) and `inference.contraCandidates` (contravariant)

- [ ] **5.117** Variance computation with marker types (`getVariancesWorker`)
  - **TS Source:** `checker.ts:24942` - `getVariances()` and `getVariancesWorker()`
  - Algorithm:
    1. Create special marker types: `markerSuperType` and `markerSubType` where `Sub <: Super`
    2. Instantiate generic `T<markerSuperType>` and `T<markerSubType>`
    3. Check relationships:
       - If `T<Sub> <: T<Super>` → Covariant
       - If `T<Super> <: T<Sub>` → Contravariant
       - If both → Bivariant
       - If neither → Invariant
    4. For bivariant, check independence with unrelated markers
  - Cache computed variances per symbol

- [ ] **5.118** Unmeasurable/unreliable variance tracking
  - **Flags:** `VarianceFlags.Unmeasurable`, `VarianceFlags.Unreliable`
  - **Unmeasurable:** Variance couldn't be determined (circular, complex)
  - **Unreliable:** Variance is known but may not be accurate in all cases
  - Track in `RelationComparisonResult` flags
  - Report in diagnostics when variance affects assignability decisions

#### Conditional Type Edge Cases
- [ ] **5.119** Tail recursion optimization for nested conditionals (max 1000 iterations)
  - **TS Source:** `checker.ts:19712` - `getConditionalType()` main loop
  - When false branch is another conditional with same/no distribution:
    - Don't recurse, instead loop with updated parameters
    - Prevents stack overflow on deep chains like `A extends B ? X : C extends D ? Y : ...`
  - Implementation:
    ```rust
    let mut iterations = 0;
    loop {
        if iterations > 1000 { return error_type; }
        // evaluate conditional...
        if result_is_nested_conditional { continue; }
        break;
    }
    ```
  - Same optimization applies to true branch

- [ ] **5.120** Deferred evaluation for generic conditional types
  - **When to defer:** Check type contains unresolved type parameters
  - **Create ConditionalType instance** instead of resolving immediately:
    ```rust
    ConditionalType {
        root: ConditionalRoot { node, checkType, extendsType, ... },
        check_type: instantiated_check,
        extends_type: instantiated_extends,
        mapper: current_mapper,
        resolved_true_type: None,  // Lazy
        resolved_false_type: None, // Lazy
    }
    ```
  - Resolution happens when type is used in assignability check
  - **Key insight:** Conditional stays "suspended" until type params are known

- [ ] **5.121** Permissive vs restrictive instantiation for definitely true/false checks
  - **TS Source:** `checker.ts:19712` - wildcardType and restrictive checks
  - **Permissive instantiation (wildcardType):**
    - Replaces type params with `any`-like wildcard
    - If check NOT assignable to extends → DEFINITELY FALSE
    - Used to prove conditional can never be true
  - **Restrictive instantiation:**
    - Replaces type params with their constraints (or empty)
    - If check IS assignable to extends → DEFINITELY TRUE
    - Used to prove conditional is always true
  - **Example:**
    ```typescript
    type Test<T> = T extends string ? "yes" : "no";
    // Permissive: any extends string? → inconclusive
    // Restrictive: unknown extends string? → no → but we can't conclude
    ```

- [ ] **5.122** Self-referential conditionals with infer
  - **Pattern:** `type Flatten<T> = T extends Array<infer U> ? Flatten<U> : T`
  - **Issue:** Inferred `U` may reference the conditional being evaluated
  - **Solution:**
    1. Detect when inferred type contains reference to enclosing conditional
    2. Create deferred type reference with proper substitution
    3. Use instantiation depth limits (typically 50)
  - Track `inferTypeParameters` array on ConditionalRoot
  - Combine mappers properly: `combineTypeMappers(mapper, inferenceMapper)`

- [ ] **5.123** Nested distribution chains
  - **Pattern:** Conditionals distributed multiple times:
    ```typescript
    type Deep<T> = T extends infer U ? U extends infer V ? V : never : never;
    type Result = Deep<"a" | "b">; // Distributes at each level
    ```
  - **Algorithm:**
    1. Check if `checkType` is union AND conditional is distributive
    2. Map over each union member: `mapType(checkType, t => getConditionalType(...))`
    3. Each mapped result may itself distribute further
  - **Distributive condition:** `root.isDistributive` = checkType is naked type parameter
  - **Watch for:** Exponential blowup with many nested distributions

#### Mapped Type Advanced Features
- [ ] **5.124** Homomorphic vs non-homomorphic distinction (preserve modifiers)
  - **TS Source:** `checker.ts:14746` - `resolveMappedTypeMembers()`
  - **Homomorphic:** Constraint is `keyof T` for some type parameter `T`
    ```typescript
    type Homo<T> = { [K in keyof T]: T[K] };  // ✓ Homomorphic
    type NonHomo = { [K in "a" | "b"]: K };   // ✗ Not homomorphic
    ```
  - **Homomorphic properties:**
    1. Preserves `readonly`/`optional` modifiers from source (unless overridden)
    2. Enables better error messages (links to source declaration)
    3. Supports reverse inference (see 5.125)
  - **Detection:** Check if constraint `K in keyof T` where T is type parameter
  - **Implementation:** Set `ObjectFlags.Mapped | ObjectFlags.Homomorphic` on type
  - Store `modifiersType` pointing to the `T` in `keyof T`

- [ ] **5.125** Reverse mapped type inference
  - **TS Source:** `checker.ts` - inference from object to mapped type
  - **Scenario:** Infer `T` from object literal when target is `Partial<T>`
    ```typescript
    declare function make<T>(partial: Partial<T>): T;
    make({ x: 1 });  // Infer T = { x: number }
    ```
  - **Algorithm:**
    1. Recognize target is homomorphic mapped type over type param `T`
    2. "Undo" the mapping: collect properties from source
    3. Apply inverse modifiers (if mapped adds `?`, inferred type doesn't have `?`)
    4. Create inferred object type from collected properties
  - **Key function:** `inferFromObjectTypes()` when target is mapped type
  - Only works for homomorphic mapped types

- [ ] **5.126** Key filtering via `as never` in remapped keys
  - **TS 4.1+ feature:** Filter keys using `as` clause with conditional
    ```typescript
    type OnlyStrings<T> = {
      [K in keyof T as T[K] extends string ? K : never]: T[K]
    };
    ```
  - **TS Source:** `checker.ts:14949` - `getMappedTypeNameTypeKind()`
  - **Implementation:**
    1. Parse `as` clause in mapped type: `MappedType.nameType`
    2. For each key, evaluate name type with key substituted
    3. If result is `never`, skip this property entirely
    4. If result is string literal, use that as property name
  - **MappedTypeNameTypeKind enum:**
    - `None`: No `as` clause
    - `Filtering`: `as` clause that may produce `never`
    - `Remapping`: `as` clause that transforms key names

#### Template Literal Advanced Features
- [ ] **5.127** Pattern inference algorithm (`inferFromLiteralPartsToTemplateLiteral`)
  - **TS Source:** `checker.ts:26613` - `inferTypesFromTemplateLiteralType()`
  - **Pattern matching for type inference:**
    ```typescript
    type Parse<T> = T extends `${infer A}-${infer B}` ? [A, B] : never;
    type R = Parse<"hello-world">; // ["hello", "world"]
    ```
  - **Algorithm:**
    1. Match source string against target template pattern
    2. Source text must start with target's first literal segment
    3. Source text must end with target's last literal segment
    4. For each `infer` placeholder:
       - Find delimiter (next literal segment) in remaining source
       - Extract substring between current position and delimiter
       - Create inferred type (string literal or template if complex)
    5. If any delimiter not found → inference fails
  - **Implementation:**
    ```rust
    fn infer_from_template_pattern(
        source_texts: &[String],  // ["hello", "world"]
        source_types: &[TypeId],  // Types between texts
        target: &TemplateLiteralType,
    ) -> Option<Vec<TypeId>>
    ```

- [ ] **5.128** Template literal wildcard handling
  - **TS Source:** `checker.ts:18988` - `getTemplateLiteralType()`
  - **Wildcard type (`wildcardType`):** Represents "any string" in patterns
  - **When wildcard appears:**
    - As input: Template becomes `string` (can't be more specific)
    - In inference: Matches any substring greedily
  - **Handling in `addSpans()`:**
    ```rust
    if type_is_wildcard(type_id) {
        return wildcard_type; // Entire template collapses to wildcard
    }
    ```
  - **Greedy vs non-greedy:** Inference is greedy (matches longest possible)
  - Issue #49839: Lazy matching can cause inference failure

- [ ] **5.129** Recursive template literal depth limits
  - **Problem:** Template literals can cause exponential expansion:
    ```typescript
    type Repeat<S extends string, N extends number> = 
      N extends 0 ? "" : `${S}${Repeat<S, Decrement<N>>}`;
    ```
  - **Limits to implement:**
    - Max recursion depth: ~50 instantiation levels
    - Max result length: Prevent memory explosion
    - Max union members when template contains union
  - **Detection:** Track instantiation depth in checker state
  - **On limit exceeded:** Return `string` type (fallback)
  - **TS issues:** #62937, #62933 - Stack overflow with deep recursion
  - **Implementation:**
    ```rust
    if self.template_literal_depth > MAX_TEMPLATE_DEPTH {
        return self.string_type;
    }
    self.template_literal_depth += 1;
    let result = self.resolve_template_literal(...);
    self.template_literal_depth -= 1;
    ```

#### Performance & Architecture (from Go lessons)
- [x] **5.82** Type relation caching (`(source, target) → result` map)
- [ ] **5.83** Flow state recycling (object pooling)
- [x] **5.84** Apparent type cache
- [x] **5.85** Awaited type cache
- [x] **5.86** Literal union base type cache (widened type cache)

- [ ] **5.130** Error recovery (continue checking after errors)
  - **Goal:** Don't abort checking on first error; collect all diagnostics
  - **Strategies:**
    1. **Error type propagation:** When subexpression has error, propagate `errorType`
       - `errorType` is assignable to/from anything → prevents cascading errors
    2. **Graceful degradation:** If type resolution fails, use `unknownType` or `anyType`
    3. **Continue on diagnostics:** `error()` records diagnostic but doesn't throw
  - **Implementation:**
    ```rust
    fn check_expression(&mut self, node: &Node) -> TypeId {
        match self.check_expression_inner(node) {
            Ok(type_id) => type_id,
            Err(diagnostic) => {
                self.diagnostics.push(diagnostic);
                self.error_type  // Continue with error type
            }
        }
    }
    ```
  - **Key insight:** Never panic during type checking; always return a type

- [ ] **5.131** Incrementality granularity (file-level vs finer caching)
  - **File-level (current TypeScript approach):**
    - On file change, invalidate all types from that file
    - Re-check entire file, reuse cross-file type info
  - **Finer-grained (Salsa-style):**
    - Track dependencies at function/statement level
    - Only re-check affected declarations
  - **Implementation options:**
    1. **Salsa crate:** Rust incremental computation framework
       - Memoizes function calls, auto-invalidates on input change
       - See `salsa = "0.17"` for integration
    2. **Manual dependency tracking:**
       - `TypeId` → `Set<SourceFileId>` reverse mapping
       - On file change, walk dependents and invalidate
  - **Trade-off:** Finer granularity = more memory overhead, better incremental perf
  - **Recommendation:** Start with file-level, add Salsa for LS later

- [ ] **5.132** Type instantiation cache (avoid redundant instantiation)
  - **Problem:** Same generic with same type args instantiated many times
    ```typescript
    let a: Array<string>; // Instantiate Array<string>
    let b: Array<string>; // Should reuse cached instantiation
    ```
  - **Cache key:** `(generic_type_id, type_arguments: Vec<TypeId>)`
  - **Cache location:** On the generic type or global checker state
  - **TS implementation:** `instantiations` map on `GenericType`
  - **Rust implementation:**
    ```rust
    struct GenericType {
        type_parameters: Vec<TypeParameterId>,
        instantiation_cache: FxHashMap<TypeArgumentsKey, TypeId>,
    }
    
    fn instantiate_generic(&mut self, generic: TypeId, args: &[TypeId]) -> TypeId {
        let key = TypeArgumentsKey::new(args);
        if let Some(cached) = self.get_generic(generic).instantiation_cache.get(&key) {
            return *cached;
        }
        let instance = self.create_instantiation(generic, args);
        self.get_generic_mut(generic).instantiation_cache.insert(key, instance);
        instance
    }
    ```
  - **Also cache:** Conditional type instantiations, mapped type instantiations

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
- [ ] Create native `tscrs` binary (no Node.js required)
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

---

## Additional Type Checker Features (from test analysis)

### Error Elaboration & Diagnostics (from test patterns)
- [ ] **5.183** Deep elaboration into arrow expressions
  - **Tests:** `deepElaborationsIntoArrowExpressions.ts`
  - Show which parameter or return in callback doesn't match
  - Traverse into nested arrow types

- [ ] **5.184** Error elaboration for intersection types
  - **Tests:** `errorMessagesIntersectionTypes*.ts` (4 tests)
  - Show which member of intersection fails
  - Report all conflicting properties

- [ ] **5.185** Did-you-mean suggestions for misspellings
  - **Tests:** `didYouMeanSuggestionErrors.ts`, `didYouMeanElaborationsForExpressionsWhichCouldBeCalled.ts`
  - Levenshtein distance for property name suggestions
  - Suggest calling function if forgetting `()`

- [ ] **5.186** Related spans for duplicate identifiers
  - **Tests:** `duplicateIdentifierRelatedSpans*.ts` (7 tests)
  - Show original declaration location
  - Link to all duplicate declarations

- [ ] **5.187** Error elaboration with discriminants
  - **Tests:** `errorMessageOnIntersectionsWithDiscriminants01.ts`, `discriminatedUnionErrorMessage.ts`
  - When discriminated union fails, show which variant and why
  - Point to discriminant property mismatch

- [ ] **5.188** Arity error with binding pattern related span
  - **Tests:** `arityErrorRelatedSpanBindingPattern.ts`
  - When destructuring has wrong arity, point to source
  - Show expected vs actual element count

- [ ] **5.189** Elaboration dives into present props only
  - **Tests:** `errorElaborationDivesIntoApparentlyPresentPropsOnly.ts`
  - Don't elaborate into properties that don't exist
  - Focus on the actual type mismatch

- [ ] **5.190** Error location for interface extension
  - **Tests:** `errorLocationForInterfaceExtension.ts`
  - Point to the extends clause that causes conflict
  - Show incompatible base types

### Async/Await & Promise Types (from test patterns)
- [ ] **5.191** Async function return type inference
  - **Tests:** `asyncFunctionReturnType.ts`, `asyncFunctionReturnType.2.ts`
  - Wrap return type in `Promise<T>` automatically
  - Handle explicit `Promise<T>` return annotation

- [ ] **5.192** Async function with strict null checks
  - **Tests:** `asyncFunctionsAndStrictNullChecks.ts`, `awaitedTypeStrictNull.ts`
  - Awaited type strips `undefined` when appropriate
  - Handle nullable promise types

- [ ] **5.193** Contextual typing for async function returns from union
  - **Tests:** `contextuallyTypeAsyncFunctionReturnTypeFromUnion.ts`
  - When target is union, pick async-compatible branch
  - Infer from Promise type structure

- [ ] **5.194** Await expression in sync function error
  - **Tests:** `awaitCallExpressionInSyncFunction.ts`, `awaitInNonAsyncFunction.ts`
  - Report specific error for await outside async
  - Suggest making function async

- [ ] **5.195** Awaited type with jQuery-style thenables
  - **Tests:** `awaitedTypeJQuery.ts`
  - Handle non-standard Promise-like types
  - Check for `.then()` method signature

### Generator & Iterator Types (from test patterns)
- [ ] **5.196** Generator return type inference
  - **Tests:** `contextuallyTypeGeneratorReturnTypeFromUnion.ts`
  - Infer `Generator<Y, R, N>` from yield/return
  - Handle yield* delegation

- [ ] **5.197** Contextual typing of yield expressions
  - **Tests:** `contextualTypeOnYield1.ts`, `contextualTypeOnYield2.ts`
  - Type of yield expression from generator's TNext
  - Yield with no value has `undefined` type

- [ ] **5.198** Custom async iterator types
  - **Tests:** `customAsyncIterator.ts`
  - User-defined `[Symbol.asyncIterator]()` method
  - Check iterator protocol conformance

- [ ] **5.199** Async yield* contextual typing
  - **Tests:** `asyncYieldStarContextualType.ts`
  - Delegate to async iterable
  - Combine yield and await types

### Optional Property & Nullability (from test patterns)
- [ ] **5.200** Contextually typed optional property
  - **Tests:** `contextuallyTypedOptionalProperty.ts`
  - Object literal with optional property from context
  - Handle `undefined` union automatically

- [ ] **5.201** Discriminate with optional property
  - **Tests:** `discriminateWithOptionalProperty*.ts` (4 tests)
  - Optional property as discriminant (`prop?: "a" | undefined`)
  - Handle missing vs explicitly undefined

- [ ] **5.202** Delete expression must be optional
  - **Tests:** `deleteExpressionMustBeOptional_exactOptionalPropertyTypes.ts`
  - With exactOptionalPropertyTypes, delete requires `?`
  - Error on deleting required property

- [ ] **5.203** Default parameter adds undefined with strict null
  - **Tests:** `defaultParameterAddsUndefinedWithStrictNullChecks.ts`
  - `function f(x = 1)` has `x: number | undefined` for callers
  - Internal type is just `number`

### Exact Optional Property Types (from test patterns)
- [ ] **5.204** Exact optional property types mode
  - **Tests:** `exactOptionalPropertyTypesIdentical.ts`, `declarationEmitExactOptionalPropertyTypesNodeNotReused.ts`
  - Distinguish `{x?: T}` from `{x: T | undefined}`
  - Stricter checking for optional vs undefined

### Correlated Unions (from test patterns)
- [ ] **5.205** Correlated union type checking
  - **Tests:** `correlatedUnions.ts`
  - Track relationship between discriminant and payload
  - `{kind: "a", value: A} | {kind: "b", value: B}` correlation

### Control Flow Edge Cases (from test patterns)
- [ ] **5.206** Control flow in try/catch/finally
  - **Tests:** `controlFlowForCatchAndFinally.ts`, `controlFlowFinallyNoCatchAssignments.ts`
  - Track assignments in catch block
  - Finally block sees all paths

- [ ] **5.207** Control flow comma expression with assertion
  - **Tests:** `controlFlowCommaExpressionAssertionWithinTernary.ts`
  - `(assert(x), x)` narrows x
  - Assertion in comma expression position

- [ ] **5.208** Control flow destructuring in try/catch
  - **Tests:** `controlFlowDestructuringVariablesInTryCatch.ts`
  - Destructured variable narrowing across try/catch
  - Handle potential exceptions

### Special Type Checking Cases (from test patterns)
- [ ] **5.209** Complex recursive collection types
  - **Tests:** `complexRecursiveCollections.ts`
  - Deeply nested generic types like `Map<K, Set<V>>`
  - Handle type parameter substitution chains

- [ ] **5.210** Complicated generic recursive base class reference
  - **Tests:** `complicatedGenericRecursiveBaseClassReference.ts`
  - `class A<T> extends B<A<T>>`
  - Break recursive base type chains

- [ ] **5.211** Composite contextual signature
  - **Tests:** `compositeContextualSignature.ts`
  - Multiple contextual types merged for callback
  - Union/intersection of contextual signatures

- [ ] **5.212** Non-distributive conditional type infer
  - **Tests:** `nondistributiveConditionalTypeInfer.ts`
  - `[T] extends [U]` prevents distribution
  - Infer in non-distributive context

- [ ] **5.213** Non-generic partial instantiation in both directions
  - **Tests:** `nongenericPartialInstantiationsRelatedInBothDirections.ts`
  - Partially instantiated generic assignability
  - Check both subtype directions

- [ ] **5.214** Normalized intersection too complex
  - **Tests:** `normalizedIntersectionTooComplex.ts`
  - Large intersection exceeds complexity limit
  - Return error type and continue

- [ ] **5.215** Comparison of partial deep with indexed access
  - **Tests:** `comparisonOfPartialDeepAndIndexedAccessTerminatesWithoutError.ts`
  - Deep utility type with indexed access
  - Ensure termination

---

## Performance Optimizations (DONE)

- [x] Zero-copy UTF-8 scanner (String instead of Vec<char>)
- [x] String interner with Atom handles
- [ ] SIMD-accelerated whitespace skipping
- [ ] Integrate Interner with Parser (Identifier.escaped_text)

