# Worker 4 Plan - Squad Forge

## Mission
Improve TS2339 property access diagnostics.

Status: Active
Priority: 1

## Current Assignment
TS2339 - Property does not exist errors (CONTINUED).

**Error Code:** TS2339 - "Property 'x' does not exist on type 'Y'"

**Impact:** 142 conformance tests affected

### Steps
1. **Resume TS2339 work** - continue property access checking improvements
2. **Handle optional chaining** - `?.` should not emit TS2339 when optional
3. **Handle union types** - property must exist on all union members
4. **Handle index signatures** - string/number index types allow any property
5. **Add tests** in `wasm/src/thin_checker_tests.rs` for property access patterns
6. **Run focused tests** with `./wasm/test.sh` and record delta

### Key Files
- `wasm/src/thin_checker.rs`
- `wasm/src/checker/expressions.rs`
- `wasm/src/thin_checker_tests.rs`

### Success Criteria
- TS2339 emitted when accessing non-existent properties
- Correct handling of unions, intersections, index signatures, optional chaining
- No new regressions

## Task Queue
- Complete remaining TS2339 patterns (optional chaining, computed properties)

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
- [x] Fixed union object literal excess property handling (TS2322 vs TS2353)
- [x] Added 6 union contextual typing tests for object literals
- [x] Added stub implementations for control flow fall-through functions
- [x] Fixed ambient module tracking in external modules (binder bug)
- [x] Added 6 module resolution tests (TS2792 vs TS2307)

## Ready for Merge
No

## Notes
- Progress: Completed TS2792 module resolution error handling
- Found and fixed binder bug: ambient modules weren't tracked in files with imports
- Implementation already existed, just needed bug fix and tests
- Tests: All 6 new module resolution tests pass
- Fix: Removed `&& !self.is_external_module` check in binder
- Impact: Ambient modules now correctly suppress TS2792 errors
- Commit format: `[wasm] checker: TS2792 module resolution`
- Push to: `origin/worker/forge-4`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`

## Resume
- Branch/state: `worker/forge-4`, work completed on TS2792, ready for merge.
- Session work: Fixed binder ambient module bug, added comprehensive tests.
- Unit tests: 6 new tests added and passing.
