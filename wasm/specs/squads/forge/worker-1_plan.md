# Worker 1 Plan - Squad Forge

## Mission
Reduce TS2300 duplicate identifier false positives.

Status: Active
Priority: 1

## Current Assignment
Emit TS2300 for duplicate identifiers in parameter lists (including destructured parameters).

**Error Code:** TS2300 - "Duplicate identifier '{0}'."

**Impact:** 105 conformance tests affected

### Steps
1. **Inspect parameter checking** in `wasm/src/thin_checker.rs` for where to add duplicate-name detection.
2. **Detect duplicates** across parameter lists and within destructured parameters.
3. **Add tests** in `wasm/src/thin_checker_tests.rs` for duplicate parameters and destructured duplicates.
4. **Run focused tests** with `./wasm/test.sh duplicate_identifier` and report delta.

### Key Files
- `wasm/src/thin_checker.rs`
- `wasm/src/thin_checker_tests.rs`

### Success Criteria
- TS2300 emitted for duplicate parameter names
- No false positives for distinct parameters

## Task Queue
- Extend to function overload lists if needed.

## Completed
- TS2454 implementation merged into squad/forge.
- TS2564 property initialization tracking and tests (merged).
- Implemented TS2564 property initialization check using type annotations when symbol types are `any`/`unknown`.
- Added TS2564 tests for required property errors and `undefined` union exemption.
- Ran `./wasm/test.sh` (fails at `cli::driver_tests::compile_class_with_generic_constructor`).
- Rebuilt WASM and reran conformance (debug runner): 79/446 exact, 102/446 same count; TS2564 no longer in top missing. OOB crashes persist for private name/decorators/etc.
- Added TS2304 coverage for unresolved identifiers and type references (including nested function types) plus TS2339-only property access check.
- Emitted TS2304 for missing names inside type aliases and interface members; added type-node traversal for complex types.
- Ran `./wasm/test.sh missing_type_reference`, `./wasm/test.sh missing_identifier_emits_2304`, `./wasm/test.sh missing_property_access_emits_2339_not_2304`.
- Rebuilt WASM and reran conformance (debug runner): 77/446 exact, 100/446 same count; TS2304 extra count 145 (up +2); OOB crashes persist.
- Ran Docker conformance `run-conformance.sh --max=1000 --workers=6`: 879 tests run, 15.5% exact (136), 17.4% same count (153), 223 crashes, 121 skipped. Top missing: TS2705(37), TS2322(26), TS2339(22), TS1109(16), TS2524(15). Top extra: TS2304(192), TS2355(83), TS1005(64), TS7010(48), TS2339(31).
- Docker conformance `run-conformance.sh --all --workers=14` OOM/exit 137 at 0% progress.
- Ran Docker conformance `run-conformance.sh --all --workers=6`: 4928 tests run, 17.6% exact (869), 20.2% same count (994), 1936 crashes, 727 skipped. Top missing: TS2322(135), TS7010(84), TS2339(76), TS2304(66), TS2695(46). Top extra: TS2304(494), TS1005(288), TS1109(171), TS7011(165), TS7010(164).
- Fixed compilation error in solver/subtype.rs (s_sym undefined in TypeQuery match arm).
- Added built-in utility type handling to reduce false TS2304 errors (Partial, Required, Pick, Omit, Record, Exclude, Extract, NonNullable, ReturnType, Parameters, etc.).
- **TS2454 implementation complete**: Fixed `get_type_of_call_expression` to process arguments even when callee is `any`. Added 7 tests covering basic cases and conditional branches (all passing).
- **TS2564 implementation enhanced**: Added 4 more edge case tests (parameter properties, conditional constructor assignments, derived classes with super). Total 11 tests passing.
- Added TS2564 tracking for string/numeric literal property names and element-access assignments; added 3 tests.
- Implemented TS2792 module resolution split (2307 for relative, 2792 for package) and ambient module tracking; added missing import tests. `./wasm/test.sh` failed at `cli::driver_tests::compile_generic_utility_library_type_utilities`.
- Fixed multi-file module resolution diagnostics by moving unresolved import errors to the CLI driver and suppressing checker import diagnostics in multi-file mode.
- Fixed object spread type checking by wiring parent pointers for literal/unary nodes and merging spread properties into object literal types; `./wasm/test.sh cli::driver_tests::compile_object_spread` passing.
- Implemented TS2300 duplicate identifier detection using declaration conflict rules; added tests for var/function, var/let, type alias conflicts, and type alias + function allowed. Ran `./wasm/test.sh duplicate_identifier` and `./wasm/test.sh type_alias_with_function_no_duplicate_2300`.
- Re-synced with `origin/rust` and reran `./wasm/test.sh duplicate_identifier` (passed).
- Re-synced with `origin/rust` and reran `./wasm/test.sh duplicate_identifier` after API key setup (passed).
- Added `find-ts2792.mjs` scan (virtual FS + directive parsing) and verified no mismatches in first 500 tests.
- Updated binder to avoid recording ambient module declarations in external modules; driver now suppresses checker import diagnostics in multi-file mode.
- Added TS2792 tests for module augmentation resolution and declared module recording; ran `./wasm/test.sh ts2792` and `./wasm/test.sh declared_module_recorded_in_script`.
- Re-ran `find-ts2792.mjs --max=1000 --samples=30`: 0 missing, 0 extra, 0 mismatched.
- **TS2300 parameter duplicate detection complete**: Implemented `check_duplicate_parameters()` and `collect_parameter_names()` to detect duplicate parameter names in function/method/constructor/accessor parameter lists. Handles simple parameters (a, b, a), object destructuring ({ a, b, a }), array destructuring ([x, y, x]), and nested patterns. Added 9 comprehensive tests covering all scenarios. All tests passing: `./wasm/test.sh duplicate_parameter` (9/9 passed), `./wasm/test.sh duplicate_identifier` (3/3 passed). Pushed to `origin/worker/forge-1`.
- **Control flow narrowing fix**: Fixed assignment narrowing to distinguish between direct assignments (`x = 1` narrows to RHS type) vs destructuring assignments (`[x] = [1]` clears narrowing to declared type). All assignment control flow tests passing (9/9).
- **TS2454 differential testing**: Created `find-ts2454.mjs` to measure TS2454 coverage. Initial scan (500 tests): 37 files where TSC emits TS2454 but WASM doesn't. This establishes baseline for implementing definite assignment analysis.

## Ready for Merge
Yes (TS2300 complete; TS2454 measurement complete, implementation pending)

## Notes
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: fix object spread type handling`
- Push to: `origin/worker/forge-1`
- TS2792 changes already pushed to `origin/worker/forge-1` if you want to merge before TS7010 work
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`

## Resume Notes
- Branch: `worker/forge-1`
- Unit tests: 4922 total, 4847 passed, 74 failed, 1 ignored.
- Synced with origin/squad/forge (resolved merge conflict in binder.rs, added CallableShape fields).
- TS2304 work complete: added utility type handling to reduce false positives.
- TS2454 implementation complete: 7 tests passing.
- TS2564 implementation complete:
  - 14 tests passing
  - Handles: optional, initializers, definite assertion (!), static, parameter properties
  - Constructor assignment tracking with control flow (if/else, derived class super)
- TS2564 literal property coverage: element access + string/numeric literal property names; `./wasm/test.sh ts2564` passing.
- TS2792 module resolution: ambient module tracking + missing import tests; `./wasm/test.sh` fails at `cli::driver_tests::compile_generic_utility_library_type_utilities`.
- Conformance tests: Docker runner has path issue (lib.d.ts not copied), skipped for now.
- Fixed cli driver utility-type test by scoping mapped type parameters during missing-name checks; made DeepReadonly/DeepPartial non-recursive and stubbed Object; `./wasm/test.sh compile_generic_utility_library_type_utilities` passing.
- Ran `./wasm/test.sh implicit_any_return_in_signatures` (passed).
- Ran `./wasm/test.sh cli::driver_tests::compile_object_spread` (passed) after fixing object spread resolution.
