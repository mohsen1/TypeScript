# Worker 4 Plan - Squad Forge

## Mission
Implement TS2322 improvements (type not assignable) for conformance.

Status: Active
Priority: 1

## Current Assignment
TS2300 duplicate identifier in class accessor get/set pairs (compile_class_accessors).

**Error Code:** TS2300 - "Duplicate identifier 'X'"

**Impact:** `cli::driver_tests::compile_class_accessors` failing

### Steps
1. **Add focused tests** in `wasm/src/thin_checker_tests.rs` for accessor pairs vs duplicate getters.
2. **Fix duplicate identifier handling** for accessors in `wasm/src/thin_checker.rs`.
3. **Run focused tests** and re-check `compile_class_accessors`.

### Key Files
- `wasm/src/solver/compat.rs`
- `wasm/src/solver/compat_tests.rs`
- `wasm/src/thin_checker.rs`
- `wasm/src/thin_checker_tests.rs`
- `wasm/src/checker/types/assignability.rs` (if present)

### Success Criteria
- No TS2300 for getter/setter pairs
- `cli::driver_tests::compile_class_accessors` passes

## Task Queue
- Verify getter/setter pairs don't report TS2300.
- Ensure duplicate getters still report TS2300.

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
- Progress: duplicate identifier logic now treats accessors with GET/SET excludes; added class accessor pair/duplicate getter tests.
- Tests: `./wasm/test.sh class_accessor_pair_no_duplicate_2300`; `./wasm/test.sh class_duplicate_getter_2300`; `./wasm/test.sh compile_class_accessors`.
- Commit format: `[wasm] checker: fix accessor duplicate identifier`
- Push to: `origin/worker/forge-4`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`

## Resume
- Branch/state: `worker/forge-4`, changes committed/pushed, ready for merge.
- Session work: fix duplicate identifier handling for accessors (GET/SET excludes) to allow getter+setter pairs; added class accessor tests.
- Unit tests: `./wasm/test.sh class_accessor_pair_no_duplicate_2300`; `./wasm/test.sh class_duplicate_getter_2300`; `./wasm/test.sh compile_class_accessors`.
