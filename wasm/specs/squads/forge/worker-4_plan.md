# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 4

## Current Assignment
- [x] Generic Inference Hardening per GOALS.md Objective 1 - IN PROGRESS

## Task Queue
- [x] Fix `test_namespace_value_member_access` - DONE (tests were passing after fixing compilation error)
- [x] Add coverage for namespace type member access patterns - DONE (added 5 tests)
- [x] Add circular constraints in extends clauses tests - DONE (added 11 tests)
- [x] Add inference from usage pattern tests - DONE (added 15 tests)
- [x] Add context-sensitive typing tests - DONE (added 20 tests)
- [x] Add advanced generic inference tests - DONE (added 25 tests)
- [x] Add distributive conditional types stress tests - DONE (added 20 tests)
- [x] Add circular constraint edge case tests - DONE (added 20 tests)
- [x] Add mapped type edge case tests - DONE (added 16 tests: homomorphic modifiers, key remapping)
- [x] Add index signature tests - DONE (added 22 tests: string/number keys, intersection, readonly, value types)
- [x] Add generic constraint tests - DONE (added 25 tests: extends, keyof constraints)
- [x] Add recursive type tests - DONE (added 20 tests: self-referential types)
- [x] Add readonly/optional modifier tests - DONE (added 25 tests)
- [x] Add unknown type tests - DONE (added 30 tests: type guards, narrowing)
- [x] Add class type tests - DONE (added 22 tests: extends, implements, protected)
- [x] Add interface type tests - DONE (added 28 tests: extends, merge declarations, excess property checks)
- [x] Add type alias tests - DONE (added 30 tests: generic, recursive, circular references)
- [x] Add literal type tests - DONE (added 35 tests: string, number, boolean, template literal)
- [x] Add object type tests - DONE (added 28 tests: optional properties, excess property checks, type widening)

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

## Ready for Merge
Yes - 401+ solver tests added (circular constraints, inference from usage, context-sensitive typing, advanced generic inference, distributive conditional types, circular edge cases, mapped types, index signatures, generic constraints, recursive types, readonly/optional modifiers, unknown type narrowing, class types, interface types, type aliases, literal types, object types)

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
