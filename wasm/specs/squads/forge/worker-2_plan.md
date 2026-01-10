# Worker 2 Plan - Squad Forge

## Mission
Implement TS2339 property access checking improvements (property does not exist).

Status: Active
Priority: 1

## Current Assignment
Reduce false positives for property access errors by aligning TS2339 behavior with TypeScript.

**Error Code:** TS2339 - "Property 'X' does not exist on type 'Y'"

**Impact:** 142 conformance tests affected

### Steps
1. **Audit TS2339 emit points** in `thin_checker.rs` to ensure we skip diagnostics for:
   - `any` and `unknown` flows
   - `error` type (suppress cascades)
   - Union members where at least one contains the property
2. **Add tests first** in `wasm/src/thin_checker_tests.rs`:
   - `any` access should not error
   - `unknown` access should error only when narrowed
   - Optional properties on unions should not error when present in some members
3. **Implement fixes** and ensure existing TS2339 tests still pass.
4. **Run conformance tests** and record delta.

### Key Files
- `wasm/src/thin_checker.rs`
- `wasm/src/solver/operations.rs`
- `wasm/src/checker/context.rs`
- `wasm/src/checker/types/diagnostics.rs`
- `wasm/src/thin_checker_tests.rs`

### Success Criteria
- TS2339 missing errors reduced
- Extra errors do not increase (no regressions)

## Resume Notes
- Branch: `worker/forge-2`.
- Latest commit: `[wasm] checker: narrow assignment flow types for TS2339`
- Recent changes: Re-applied static index signature collection after squad/forge merge; narrowed flow types on direct assignments to use assigned expression types; added TS2339 test covering assignment-based narrowing on unions.
- Last tests: `./wasm/test.sh ts2339_assignment_narrows_union_property_access`
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
- Narrowed flow assignment handling to use assigned expression types for TS2339.
- Added TS2339 test for assignment-based union narrowing.
- Re-applied static index signature collection after squad/forge merge.
- Conformance scan after re-apply: 22 extra (from 1000 files).

## Ready for Merge
Yes

## Notes
- Full test run failing in `cli::driver_tests::compile_multi_file_project_with_imports`
  and `cli::driver_tests::compile_multi_file_project_with_default_and_named_imports` (TS2792).
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: improve TS2339 property access diagnostics`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
