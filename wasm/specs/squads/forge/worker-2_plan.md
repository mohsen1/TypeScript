# Worker 2 Plan - Squad Forge

## Mission
Improve missing-name diagnostics (TS2304) in the checker/binder.

Status: Active
Priority: 1

## Current Assignment
Improve TS2304 "Cannot find name" diagnostics for unresolved type positions (heritage clauses, type queries, and type literals).

**Error Code:** TS2304 - "Cannot find name '{0}'."

**Impact:** 138 conformance tests affected

### Steps
1. **Run a scan** for missing TS2304s: `cd wasm/differential-test && node find-ts2304.mjs --max=1000 --samples=30`.
   - If the script doesn't exist, clone `find-ts2339.mjs` into `find-ts2304.mjs` and invert the comparison to report missing (TSC-only) TS2304s.
2. **Pick top missing pattern** (e.g., `extends`/`implements` type references, `typeof` type queries, or type literal members).
3. **Implement the fix** in `thin_checker.rs`/`binder.rs` and add a focused regression test.
4. **Run focused tests** with `./wasm/test.sh` and report delta.

### Key Files
- `wasm/src/thin_checker.rs`
- `wasm/src/binder.rs`
- `wasm/src/thin_checker_tests.rs`
- `wasm/differential-test/find-ts2304.mjs` (new if needed)

### Success Criteria
- TS2304 missing errors reduced for the selected pattern
- Extra TS2304s do not increase

## Resume Notes
- Branch: `worker/forge-2`.
- Latest commit: `[wasm] driver: adapt to collect_module_specifiers tuple return`
- Recent changes: Merged origin/rust and origin/squad/forge; fixed driver.rs to handle collect_module_specifiers returning (String, NodeIndex) tuples; re-applied static index signature collection after squad/forge merge; narrowed flow types on direct assignments to use assigned expression types; added TS2339 test covering assignment-based narrowing on unions; tracked resolved module specifiers in multi-file CLI diagnostics to avoid TS2792 for in-program imports; marked optional call expressions and stripped nullish callee types for optional chaining calls.
- Last tests: `./wasm/test.sh compile_optional_chaining_with_call` (pass), `./wasm/test.sh compile_object_spread` (pass)
- **Latest TS2339 conformance scan results (1000 files):**
  - Extra (false positives): 0 files (was 3)
- Conformance scan command: `cd wasm/differential-test && node find-ts2339.mjs --max=1000 --samples=20`
- Remember: do not touch `.role/AGENTS.md`.

## Task Queue
- None (1000-file TS2339 scan shows 0 extras; waiting on new assignment)

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

## Ready for Merge
No

## Notes
- **Post-merge test status:** 68 unit test failures after merging origin/rust + origin/squad/forge
  - ~25 failures from new TS2564/TS7006/TS7010 error checks (other workers' features)
  - ~15 failures from `infer` type parameter scoping issue (needs fix)
  - ~10 failures from namespace merging regression (needs investigation)
  - ~18 other type system failures (needs triage)
  - See POST_MERGE_TEST_FAILURES.md for details
- CLI tests: `./wasm/test.sh compile_optional_chaining_with_call` (pass), `./wasm/test.sh compile_object_spread` (pass)
- Last conformance scan: `cd wasm/differential-test && node find-ts2339.mjs --max=1000 --samples=20` (0 false positives)
- Commit format: `[wasm] checker: improve TS2339 property access diagnostics`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
