# Worker 2 Plan - Squad Forge

## Mission
Implement TS2339 property access checking improvements (property does not exist).

Status: Active
Priority: 1

## Current Assignment
Reduce TS2339 false positives for **private names** and **mixin classes**, plus missing errors on `globalThis`.

**Error Code:** TS2339 - "Property 'X' does not exist on type 'Y'"

**Impact:** 142 conformance tests affected

### Steps
1. **Private names:** ensure `#prop` lookups on class types resolve correctly (static and instance).
2. **Mixin classes:** fix property resolution for mixin-generated types (avoid TS2339 on valid props).
3. **globalThis:** add missing property checks for `globalThis` access.
4. **Add tests** in `wasm/src/thin_checker_tests.rs` and re-run focused TS2339 tests.

### Key Files
- `wasm/src/thin_checker.rs`
- `wasm/src/solver/operations.rs`
- `wasm/src/thin_checker_tests.rs`

### Success Criteria
- TS2339 extra errors reduced in private name + mixin cases
- Missing `globalThis` errors emitted correctly

## Resume Notes
- Branch: `worker/forge-2` (ahead of `origin/rust`).
- Latest commit: `[wasm] checker: re-apply static index signature collection after merge`
- Recent changes: Re-applied static index signature collection after squad/forge merge overwrote changes.
- Last tests: `./wasm/test.sh test_ts2339_` (10 tests pass).
- **Latest TS2339 conformance scan results (1000 files):**
  - Extra (false positives): 22 files (was 27)
  - Main categories of false positives:
    1. **Control flow narrowing** (11 files) - assertion predicates, const locals, various control flow tests
    2. **Private names** (6 files) - `#prop` access on class types
    3. **Mixin classes** (4 files) - Properties not found on mixin types
    4. **Enum merging** (1 file) - enumMerging.ts
- Conformance scan command: `cd wasm/differential-test && node find-ts2339.mjs --max=1000 --samples=30`
- Remember: do not touch `.role/AGENTS.md`.

## Task Queue
- Fix control flow narrowing for property access after type guards (highest priority - affects 10 files)
  - **Root cause identified:** In `handle_assignment` in control_flow.rs, when an assignment affects a reference, we return the declared type instead of the assigned expression's type. For example, after `x = ""` where `x: string | number`, we should narrow `x` to `string`, but we return `string | number`.
  - **Fix needed:** Track assigned types in flow nodes and use them for narrowing.
- Investigate private name (#prop) handling - both extra and missing errors
- Fix mixin class property resolution
- Add globalThis property checking

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
- Re-applied static index signature collection after squad/forge merge.
- Conformance scan after re-apply: 22 extra (from 1000 files).

## Ready for Merge
No

## Notes
- Full test run failing in `cli::driver_tests::compile_multi_file_project_with_imports`
  and `cli::driver_tests::compile_multi_file_project_with_default_and_named_imports` (TS2792).
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: improve TS2339 property access diagnostics`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
