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

## Ready for Merge
Yes - 100+ solver tests added (circular constraints, inference from usage, context-sensitive typing, advanced generic inference, distributive conditional types)

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
