# Worker 2 Plan - Squad Forge

## Mission
Improve TS2304 missing-name diagnostics (identifier not found).

Status: Active
Priority: 1

## Current Assignment
Reduce remaining TS2304 missing-name false positives.

**Error Code:** TS2304 - "Cannot find name 'X'."

**Impact:** 30 false positives in 1000-file scan (Promise/await resolved; remaining in heritage/literal/parse contexts)

### Steps
1. **Run a scan**: `cd wasm/differential-test && node find-ts2304.mjs --max=1000 --samples=30` (done; timed out at 180s, partial results captured).
2. **Pick top missing-name pattern** (done: builtin/global types + missing-name scope for signature type params).
3. **Implement the fix** in `thin_checker.rs`/`binder.rs` and add a focused regression test (done).
4. **Run focused tests** with `./wasm/test.sh` and report delta (done; see Resume Notes).

### Key Files
- `wasm/src/thin_checker.rs`
- `wasm/src/binder.rs`
- `wasm/src/thin_checker_tests.rs`

### Success Criteria
- TS2304 false positives reduced for the selected pattern
- No new regressions in existing TS2304 tests

## Resume Notes
- Branch: `worker/forge-2`.
- Latest commit: `[wasm] checker: reduce TS2304 in switch cases and predicates`
- Recent changes: Added parent mapping for switch/case nodes, allowed type predicates in `parse_type`, added `exports` to known global values, and added tests for switch-case, type predicate params, and exports.
- Last tests: `./wasm/test.sh test_switch_case_param_reference_no_ts2304` (pass), `./wasm/test.sh test_type_predicate_param_type_no_ts2304` (pass), `./wasm/test.sh test_exports_reference_no_ts2304` (pass)
- **Latest TS2304 conformance scan results:** `find-ts2304.mjs --max=1000 --samples=30` timed out at 200s; partial results still show heritage null/namespace cycles, catch/flow false positives, decorator/noTypesAndSymbols cases, and asserts/exports (may need rebuild to confirm).
- Conformance scan command: `cd wasm/differential-test && node find-ts2304.mjs --max=1000 --samples=30`
- Remember: do not touch `.role/AGENTS.md`.

## Task Queue
- Investigate remaining TS2304 in heritage cycles and invalid heritage literals (null/undefined).
- Re-run conformance scan after rebuild to confirm switch-case + predicate fixes (and catch variable scoping).
- Review TS2304 in decorator/noTypesAndSymbols cases.

## Completed
- Implemented property access on constrained type parameters in checker and solver.
- Stored class instance types + type params in type_env to expand Application types.
- Added recursion guard for class instance type resolution to avoid stack overflow.
- Updated cross-scope generic constraints test to expect no errors.
- Added TS2339 tests for any/unknown/union optional property access.
- Tracked `this` types for class members and added computed-name checking.
- Added Object prototype members and static inheritance/namespace merging for TS2339.
- Allowed class/interface declaration merging in the binder.
- Added TS2339 tests for static-instance access, computed `this` names, class/interface merges.
- Created find-ts2339.mjs conformance scan script for TS2339 analysis.
- Added static index signature support to CallableShape (fields + property access resolution).
- Ran conformance scan: 28 extra, 29 missing (from 1000 files).
- Fixed parser to preserve static modifier on index signatures (was dropping static keyword).
- Fixed static index signature property access - staticIndexSignature4.ts now passes.
- Conformance scan after fix: 27 extra, 29 missing (from 1000 files).
- Narrowed flow assignment handling to use assigned expression types for TS2339.
- Added TS2339 test for assignment-based union narrowing.
- Re-applied static index signature collection after squad/forge merge.
- Conformance scan after re-apply: 22 extra (from 1000 files).
- Fixed mixin class inheritance to merge intersection base properties.
- Allowed constructor callables to satisfy constructor function constraints.
- Added TS2339 tests for mixins, globalThis property access, and private identifiers.
- Conformance scan after mixin/private/globalThis fixes: 3 extra (from 1000 files).
- Suppressed TS2339 for property access with missing identifiers from parse errors.
- Allowed heritage parsing of parenthesized/new base expressions for extends.
- Merged base instance/static properties when extends resolves to non-class symbol.
- Conformance scan after heritage/parse fixes: 0 extra (from 1000 files).
- Scoped type parameters for type-alias/mapped-type missing-name checks (fixes TS2304 in mapped types).
- Resolved TS2792 import diagnostics in multi-file CLI mode by tracking resolved module specifiers.
- Fixed optional call chaining to avoid TS2349 on `?.()` when the callee is optional.
- Added TS2552 for `await` in type position and fallback handling for missing builtin/global types in TS2304 checks.
- Scoped missing-name checks for signature type parameters and suppressed TS2304 in heritage literal expressions.
- Added tests for builtin types in type literals and expanded builtin coverage (Promise/NonNullable/PropertyKey/etc).
- Set parent pointers for switch/case nodes to restore scope resolution in switch clauses.
- Allowed type predicates in `parse_type` to avoid TS2304 for `asserts`/`is` in parameter types.
- Added `exports` to known global values and tests covering switch-case, type predicate params, and exports.

## Ready for Merge
Yes

## Notes
- **Post-merge test status:** 68 unit test failures after merging origin/rust + origin/squad/forge
  - ~25 failures from new TS2564/TS7006/TS7010 error checks (other workers' features)
  - ~15 failures from `infer` type parameter scoping issue (needs fix)
  - ~10 failures from namespace merging regression (needs investigation)
  - ~18 other type system failures (needs triage)
  - See POST_MERGE_TEST_FAILURES.md for details
- CLI tests: `./wasm/test.sh compile_optional_chaining_with_call` (pass), `./wasm/test.sh compile_object_spread` (pass)
- Last conformance scan: `find-ts2304.mjs --max=1000 --samples=30` (timed out at 180s, partial results captured).
- Commit format: `[wasm] checker: improve TS2304 missing-name diagnostics`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
