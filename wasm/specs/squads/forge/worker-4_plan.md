# Worker 4 Plan - Squad Forge

## Mission
Fix TS2322 return-type assignability (void/undefined).

Status: Active
Priority: 1

## Current Assignment
TS2322 return-type assignability (void/undefined).

**Error Code:** TS2322 - "Type 'X' is not assignable to type 'Y'"

**Impact:** return type diagnostics/conformance

### Steps
1. **Add focused tests** in `wasm/src/solver/compat_tests.rs`.
2. **Fix assignability** in `wasm/src/solver/compat.rs` and/or `wasm/src/solver/subtype.rs`.
3. **Run focused tests** and record delta.

### Key Files
- `wasm/src/solver/compat.rs`
- `wasm/src/solver/compat_tests.rs`
- `wasm/src/solver/subtype.rs`
- `wasm/src/thin_checker.rs` (if needed)

### Success Criteria
- void/undefined return assignability matches TypeScript
- No extra TS2322 regressions

## Task Queue
- Capture void/undefined return-type cases in compat tests.
- Validate no regressions in existing assignability tests.

## Completed

- [x] Fixed method type parameter scope in check_method_declaration
- [x] Added solver coverage for all major utility type patterns (Partial, Required, Pick, Omit, Record, Exclude, Extract, etc.)
- [x] Added solver coverage for template literal patterns and string intrinsics
- [x] Added solver coverage for recursive conditional types and variadic tuples
- [x] Added solver edge case tests (intersections, unions, generics, conditionals)
- [x] Added checker coverage for class/enum/function namespace merges
- [x] Fixed namespace value/type member access tests (all passing)
- [x] Fixed compilation error in subtype_tests.rs (object_shape_with_index -> object_with_index)
- [x] Added 5 namespace type member access pattern tests
- [x] Added 11 circular constraints in extends clauses tests (F-bounded polymorphism)
- [x] Added 10 additional circular constraint edge cases (polymorphic this, promise, event emitter, fluent interface, recursive JSON, linked list, state machine, visitor, expression tree, repository patterns)
- [x] Added 15 inference from usage pattern tests
- [x] Added 20 context-sensitive typing tests
- [x] Added 25 advanced generic inference tests (mapped types, conditional infer, variadic tuples)
- [x] Added 20 distributive conditional types stress tests
- [x] Added 20 circular constraint edge case tests (5-way cycles, diamond pattern, mutual recursion, index signatures)
- [x] Added 16 mapped type edge case tests (homomorphic modifiers, key remapping, Pick/Omit/Record patterns)
- [x] Added 22 index signature tests (string/number keys, intersection, readonly, value types)
- [x] Added 25 generic constraint tests (extends, keyof constraints)
- [x] Added 20 recursive type tests (self-referential types)
- [x] Added 25 readonly/optional modifier tests
- [x] Added 30 unknown type tests (type guards, narrowing)
- [x] Added 22 class type tests (extends, implements, protected)
- [x] Added 28 interface type tests (extends, merge declarations, excess property checks)
- [x] Added 30 type alias tests (generic, recursive, circular references)
- [x] Added 35 literal type tests (string, number, boolean, template literal)
- [x] Added 28 object type tests (optional properties, excess property checks, type widening)
- [x] Added 30 bigint type tests (literal bigints, arithmetic operations)
- [x] Added 30 typeof type tests (type queries on values, expressions)
- [x] Added 30 infer type tests (conditional type inference)
- [x] Added 30 this type tests (this in classes, fluent interfaces)
- [x] Added 30 readonly property tests (readonly modifiers, Readonly<T>)
- [x] Enforced protected access receiver checks and added TS2445 coverage (base instance + static constructor)
- [x] Fixed BindResult import issue in lib.rs
- [x] Cleaned up duplicate check_property_accessibility code from rebase conflict
- [x] Fixed let...else syntax error in protected access check (converted to match expression)
- [x] Merged with origin/rust, fixed s_sym scope bug in solver/subtype.rs
- [x] Added get_type_of_assignment_target function for binary expression checking
- [x] Added check_parameter_initializers function for TS2322 on default parameter values
- [x] Applied check_parameter_initializers to constructors, methods, accessors, functions

## Ready for Merge
Yes

## Notes
- Progress: added compat coverage for void/undefined return-type assignability; no compat/subtype changes required.
- Tests: `./wasm/test.sh void_undefined_return_assignability`.
- Commit format: `[wasm] solver: cover void/undefined return assignability`
- Push to: `origin/worker/forge-4`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`

## Resume
- Branch/state: `worker/forge-4`, changes committed/pushed, ready for merge.
- Session work: added compat tests for void/undefined return-type assignability in functions and call signatures.
- Unit tests: `./wasm/test.sh void_undefined_return_assignability`.
