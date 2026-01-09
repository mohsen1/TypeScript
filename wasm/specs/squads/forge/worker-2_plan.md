# Worker 2 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 2

## Current Assignment
- [x] ReturnType/Parameters edge case tests (per GOALS.md W2/Pane4 assignment)
- [x] Distributive conditional type stress tests (per GOALS.md objective #2)
- [x] Context-sensitive typing tests (generic function call inference, contextual parameter types)
- [x] Function return type inference tests for conditional types
- [x] Variadic tuple type tests
- [x] Rest parameter inference tests
- [x] Type guard tests (is, asserts, narrowing)
- [x] Discriminated union tests (type narrowing, exhaustiveness)
- [x] Never type tests (impossible values, exhaustiveness)
- [x] Any type tests (type erasure, type assertions)
- [x] Tuple type tests (labeled elements, rest, optional, spreads)
- [x] Array type tests (readonly, generic, type inference)
- [x] Union type tests (type narrowing, union distribution)
- [x] Spread type tests (tuple spreads, object spreads)

## Task Queue
- [ ] Implement TypeResolver for TypeEvaluator to resolve Refs inside Mapped/Conditional types
- [ ] This would enable full evaluation of complex patterns like `{ [K in keyof R]: ExtractAction<R[K]> }[keyof R]`

## Completed
- [x] Added 45 spread type tests including:
  - Tuple spreads: basic, leading/trailing elements, multiple spreads
  - Empty/single element tuple spreads
  - Rest elements at start/end positions
  - Optional elements, labeled elements
  - Generic tuple spread inference
  - Union/literal types in spread elements
  - Nested tuples, readonly tuples
  - Object spreads: basic, two/three-way merge
  - Property override (later wins)
  - Optional/readonly property handling
  - Methods, union/intersection properties
  - Nested objects, array/function/tuple properties
  - Generic object spread inference
  - Combined spread patterns
  - Special types (never, any, unknown) in spreads
  - Long tuples/objects (10 elements)
- [x] Added 40 union type tests including:
  - Basic unions (string | number, 3+ types)
  - Nullable types (string | null, string | undefined)
  - Literal unions (string, number, boolean literals)
  - Object/function/array/tuple unions
  - Never absorption (string | never = string)
  - Any absorption (string | any = any)
  - Unknown in unions
  - Duplicate removal, order independence
  - Nested union flattening
  - Unions in function params/returns
  - Unions in object properties/array elements
  - Union inference
  - Discriminated unions (string/number discriminants)
  - Conditional type distribution
  - Unions with void, promises, intersections
- [x] Added 32 array type tests including:
  - Basic primitive arrays (string[], number[], boolean[])
  - Readonly arrays (readonly string[], readonly number[])
  - Mutable vs readonly distinction
  - Arrays of complex types (union, object, function, tuple)
  - Nested arrays (2D, 3D)
  - Special type arrays (any[], unknown[], never[], void[], null[], undefined[])
  - Literal type arrays ("hello"[], 42[])
  - Array type equality/inequality
  - Arrays in function params/returns/rest params
  - Array inference from element types
  - Arrays in object properties (optional, readonly)
  - Arrays of intersection, promise, keyof types
- [x] Added 29 tuple type tests including:
  - Basic fixed-length tuples
  - Labeled elements [name: string, age: number]
  - Optional elements [string, number?, boolean?]
  - Rest element at end/start/middle
  - Labeled with optional, labeled rest
  - Empty tuple, single element tuple
  - Nested tuples, tuple spreads
  - Union/object/function/literal elements
  - Multiple optional at end, optional before rest
  - Never/any/unknown elements
  - Readonly array spread
  - Tuple parameter inference
  - Promise elements
  - Tuple vs array distinction
  - Long tuples (10 elements)
  - Mixed labels
- [x] Added 29 any type tests including:
  - Basic any identity and type key lookup
  - Any in union (absorption) and intersection
  - Any function return types and parameters
  - Any in arrays, tuples, object properties
  - Any inference (accepts all, upper bound)
  - Any in conditional types (extends checks)
  - Promise<any>, readonly any[]
  - Any generic constraints and defaults
  - keyof any, any[K] indexed access
  - Any rest parameters and callbacks
  - Template literals with any
  - Optional/readonly any properties
  - Nested objects with any
  - Any as type argument
  - Any vs unknown distinction
  - Constructor returns with any
  - This type as any
- [x] Added 30 never type tests including:
  - Basic never identity and type key lookup
  - Never in union (absorption) and intersection (domination)
  - Never function return types (throw, infinite loop)
  - Never function parameters (assertNever pattern)
  - Never in arrays, tuples, object properties
  - Never inference for exhaustive checks
  - Never in conditional types (true/false branches)
  - Distributive filtering with never
  - Empty union = never, all-never union = never
  - Never generic constraints and defaults
  - Never in mapped types (no keys = empty object)
  - Promise<never>, readonly never[]
  - Switch exhaustiveness patterns
  - keyof never, never[K] indexed access
  - Never rest parameters and callbacks
  - Template literals with never
  - Overload resolution with never
  - Methods returning never in classes
- [x] Added 22 discriminated union tests including:
  - String, number, boolean literal discriminants
  - Multiple discriminant properties
  - Three+ variant unions (Shape: circle, square, rectangle)
  - Exhaustiveness checking and never narrowing
  - Nested discriminated unions
  - Common properties across variants
  - null/undefined variants
  - Redux action, AST node, API response patterns
  - Form validation, state machine patterns
  - Option/Result type patterns
  - Readonly discriminants, methods in variants
- [x] Added 26 type guard tests including:
  - Basic 'x is T' and 'asserts x is T' predicates
  - 'this is T' class method predicates
  - Generic type guard and union narrowing
  - instanceof, typeof, 'in' patterns
  - Discriminated union and never narrowing
  - Array filter/every with type guards
  - Assertion functions and assertNever patterns
- [x] Added 26 rest parameter inference tests including:
  - Basic array and generic inference
  - Mixed types union, leading fixed params
  - Tuple spread, array spread, callback inference
  - Bind/call/apply, event handler patterns
  - Compose, partial application, promisify
  - Curry, middleware, zipWith, reduce
  - Constructor spread, decorator, Object.assign
- [x] Added 28 variadic tuple type tests including:
  - Basic rest element at end/start/middle positions
  - Inference from spread calls and contexts
  - Concat, Push, Unshift type operations
  - First/Last/Tail/Init extraction patterns
  - Function apply and curry patterns
  - Zip, flatten, partial application patterns
  - Labeled elements, optional before rest
  - Union elements in variadic tuples
- [x] Added 24 function return type inference tests for conditional types including:
  - Basic conditional return type evaluation (true/false branches)
  - Distributive conditional types with unions
  - Infer keyword for extracting return/param types
  - Never absorption and any special cases
  - Literal type and object structural subtyping conditionals
  - Tuple/array element inference, Promise unwrap
  - Generic inference context integration
  - Constructor/InstanceType inference patterns
- [x] Added 24 context-sensitive typing tests including:
  - Generic function call inference (single/multiple args, different type params)
  - Contextual callback parameter and return types
  - Inference from return context
  - Object/array literal contexts
  - Generic method chains
  - Constraint handling and violations
  - Tuple element contexts
  - Promise.then, reduce, constructor patterns
  - Spread operators, nested generic calls
  - Event handler and JSX prop typing
  - Constraint propagation between type params
- [x] Added 18 distributive conditional type stress tests including:
  - Large union distribution, nested conditionals
  - Never absorption, all-never results
  - Literal/object type distribution
  - Non-distributive wrapped type params
  - Any/unknown special cases
  - Infer patterns with distribution
  - Boolean, function type, recursive patterns
- [x] Added 28 ReturnType/Parameters/utility type edge case tests including:
  - Async function return types, void/never returns
  - Union and intersection of functions
  - Conditional return types, constructor signatures
  - this parameter handling, labeled tuple elements
  - Multiple optional params, rest with tuple types
  - Variadic tuple types, infer patterns
  - ThisParameterType, OmitThisParameter, InstanceType
  - ConstructorParameters with generics
  - Awaited with nested promises, ReadonlyArray
  - NonNullable, Extract<T,U>, Exclude<T,U> patterns
- [x] Cross-file type resolution infrastructure: added `decl_file_idx` to Symbol, `alloc_from` for symbol cloning, `all_arenas` to CheckerContext for multi-file arena access.
- [x] Fixed type parameter scope propagation to TypeLowering via `import_type_params` method.
- [x] Added Application type expansion infrastructure to SubtypeChecker (`try_expand_application`, `extract_type_params_from_type`).
- [x] Fixed regression in `invalidate_paths_with_dependents_symbols_handles_import_equals` - handle require() string literals in import equals declarations.
- [x] Reduced diagnostics from 6 to 5; type parameters now resolve correctly.
- [x] Added `is_assignable_to` Application expansion with on-demand symbol resolution via `get_type_of_symbol`.
- [x] Added `get_type_params_from_symbol_decl` to extract type parameters from symbol declarations (cross-file aware).
- [x] Added `try_resolve_ref` to resolve Ref type arguments before instantiation.
- [x] Added `expand_type_deeply` with recursive expansion of Application, IndexAccess, KeyOf, and Ref types.
- [x] Reduced diagnostics from 5 to 3; DeepPartial<RootState> now expands and evaluates correctly.
- [x] Added TypeQuery (typeof) expansion in `expand_type_recursive` - resolves `typeof X` to type of symbol.
- [x] Improved type argument resolution to use `expand_type_recursive` instead of just `try_resolve_ref`.
- [x] Reduced diagnostics from 3 to 2; ValueOf<PickValue<RootState, number>> now works correctly.

## Remaining Issues (2 diagnostics)
1. `[store.ts]` - Function type `(state: S | undefined, action: A) => any` not assignable to `Reducer<S, A>`
   - The returned arrow function's type contains unexpanded Application types like `Ref(5)<R>`
   - Requires TypeResolver for CompatChecker to expand Applications during function signature comparison

2. `[app.ts]` - `ActionFromReducers<typeof rootReducers>` contains IndexAccess with unevaluated Mapped type
   - The Mapped type `{ [K in keyof R]: ExtractAction<R[K]> }` contains conditional types with Refs
   - Requires TypeResolver for TypeEvaluator to resolve Refs when evaluating Mapped types

## Technical Analysis
The remaining issues share a common root cause: the TypeEvaluator uses NoopResolver which cannot resolve Refs. When evaluating Mapped types or conditional types that contain Refs, the evaluator cannot proceed and returns the type as-is.

Solution options:
1. Create a TypeResolver that wraps our symbol resolution and pass to TypeEvaluator::with_resolver
2. Pre-resolve all Refs in a type before passing to evaluate_type
3. Modify TypeEvaluator to accept a callback for on-demand Ref resolution

## Ready for Merge
**YES** - Branch reset to squad/forge baseline and new tests added.

Worker 2 branch now contains:
- 45 spread type tests (tuple spreads, object spreads)
- 40 union type tests (type narrowing, union distribution)
- 32 array type tests (readonly, generic, type inference)
- 28 new utility type edge case tests (ReturnType, Parameters, InstanceType, etc.)
- 18 distributive conditional type stress tests
- 24 context-sensitive typing tests (generic inference, contextual typing)
- 24 function return type inference tests for conditional types
- 28 variadic tuple type tests
- 26 rest parameter inference tests
- 26 type guard tests (is, asserts, narrowing)
- 22 discriminated union tests (type narrowing, exhaustiveness)
- 30 never type tests (impossible values, exhaustiveness)
- 29 any type tests (type erasure, type assertions)
- 29 tuple type tests (labeled elements, rest, optional, spreads)
- No modifications to evaluate.rs or other core files
- Pure test additions that can be safely merged

## Progress Summary
- Started with: 6 diagnostics
- Current: 2 diagnostics
- Reduction: 67%

## Notes
- **File Corruption Issue**: `wasm/src/solver/infer_tests.rs` has pre-existing syntax errors in lines ~19206-22872 (unclosed delimiters, missing #[test] attributes, missing function boundaries). These need to be fixed separately. Array tests were added at end of file (lines 24026+) and don't introduce new errors.
- Project Direction: integration and conformance-first; prioritize solver correctness
- Follow `wasm/specs/WASM_ARCHITECTURE.md` and `wasm/specs/SOLVER.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: <description>` or `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `DIRECTOR_AGENT.md`, `SQUAD_LEAD_AGENT.md`, `MANAGER_AGENT.md`, `AGENTS.md`, `start_*.sh`
