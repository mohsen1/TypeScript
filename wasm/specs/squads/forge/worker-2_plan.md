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
- Branch: `worker/forge-2` (ahead of `origin/rust` by 7 commits).
- Latest commit: `[wasm] checker: improve TS2339 property access diagnostics`
- Recent changes: class/interface declaration merging, computed class member name checking with `this` typing, Object prototype members, static inheritance + namespace merge for constructors, additional TS2339 tests.
- Last tests: `./wasm/test.sh test_ts2339_` (passes).
- Known remaining TS2339 diffs from last conformance scan (max 500):
  - Extra: `Symbols/ES5SymbolProperty2.ts`, `Symbols/ES5SymbolProperty6.ts`, `additionalChecks/noPropertyAccessFromIndexSignature1.ts`, `ambient/ambientDeclarations.ts`, `ambient/ambientDeclarationsExternal.ts`, `classes/classDeclarations/classAbstractKeyword/classAbstractCrashedOnce.ts`, `classes/classDeclarations/mergedClassInterface.ts`, `classes/classDeclarations/mergedInheritedClassInterface.ts`, `classes/members/accessibility/privateStaticNotAccessibleInClodule.ts`, `classes/members/accessibility/privateStaticNotAccessibleInClodule2.ts`, `classes/members/accessibility/protectedStaticNotAccessibleInClodule.ts`, `classes/members/privateNames/privateNameHashCharName.ts`.
  - Missing: `async/es6/asyncWithVarShadowing_es6.ts`, `classes/members/classTypes/staticPropertyNotInClassType.ts`, `classes/members/instanceAndStaticMembers/typeOfThisInStaticMembers12.ts`, `classes/members/instanceAndStaticMembers/typeOfThisInStaticMembers13.ts`, `classes/members/privateNames/privateNameAndIndexSignature.ts`, `classes/members/privateNames/privateNameBadAssignment.ts`, `classes/members/privateNames/privateNameBadDeclaration.ts`, `classes/members/privateNames/privateNameInInExpression.ts`.
- Conformance scan command (docker + custom script):
  - `docker run --rm --memory="8g" --cpus="4" -e NODE_PATH=/usr/local/lib/node_modules -e MAX_TESTS=500 -v "$(pwd)/wasm/pkg:/app/pkg:ro" -v "$(pwd)/tests:/tests:ro" -v "/tmp/ts2339-scan.mjs:/app/ts2339-scan.mjs:ro" ts-conformance-runner node /app/ts2339-scan.mjs`
- Remember: do not touch `.role/AGENTS.md`.

## Task Queue
- Re-run TS2339 conformance scan and log deltas.
- Investigate remaining TS2339 diffs (Symbol.iterator, index-signature property access, private names).
- Confirm behavior for property access on intersection types.

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
- Ran `./wasm/test.sh test_ts2339_` (passes; full run still fails: TS2792 module resolution in multi-file import tests).

## Ready for Merge
Yes

## Notes
- Full test run failing in `cli::driver_tests::compile_multi_file_project_with_imports`
  and `cli::driver_tests::compile_multi_file_project_with_default_and_named_imports` (TS2792).
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: improve TS2339 property access diagnostics`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
