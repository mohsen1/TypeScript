# Worker 3 Plan - Squad Forge

## Mission
Reduce TS2339 false positives via control flow narrowing.

Status: Active
Priority: 1

## Current Assignment
TS2339 missing errors: private-name access and static computed member cases.

**Error Code:** TS2339 - "Property 'X' does not exist on type 'Y'"

### Steps
1. **Run a missing scan**: `node wasm/differential-test/find-ts2339.mjs --mode=missing --max=2000 --samples=20`.
2. **Target missing patterns** from the scan (private-name access + static computed `this.c`).
3. **Fix private-name access** in `wasm/src/thin_checker.rs` (property access + `in` operator).
4. **Fix class expression computed names** in `wasm/src/thin_checker.rs`.
5. **Add tests** in `wasm/src/thin_checker_tests.rs`.
6. **Run focused tests** with `./wasm/test.sh thin_checker_tests` and report delta.

### Key Files
- `wasm/src/thin_checker.rs`
- `wasm/src/thin_checker_tests.rs`
- `wasm/differential-test/find-ts2339.mjs`

### Success Criteria
- Missing TS2339 reduced for private-name/computed-member patterns
- No new TS2339 extras introduced

## Task Queue
- (empty)

## Completed
- [x] TS2454 error code and message added to diagnostics
- [x] DefiniteAssignmentAnalyzer implemented in control_flow.rs
- [x] Flow-based assignment tracking (ASSIGNMENT, BRANCH_LABEL, LOOP_LABEL, CONDITIONS)
- [x] Integration in thin_checker.rs for block-scoped variables without initializer
- [x] Test cases added in thin_checker_tests.rs
- [x] Fixed BindResult import in lib.rs

### Conformance Test Results (500 tests)
- Exact Match: 90 (18.5%)
- Same Error Count: 104 (21.4%)
- TS2454 false positives FIXED (no longer in top 10 extra errors)

### Conformance Test Results (Full run, 4928 tests)
- Command: `node wasm/differential-test/process-pool-conformance.mjs --max=999999 --workers=4`
- Exact Match: 677 (13.7%) vs baseline 23.3% (1148/4928)
- Same Error Count: 780 (15.8%)
- Missing Errors: 1405 (28.5%)
- Extra Errors: 1022 (20.7%)
- WASM Crashed: 2498
- Top missing errors: TS2564 (160), TS2322 (82), TS2300 (70), TS2304 (70), TS7010 (66)
- Top extra errors: TS2304 (354), TS1005 (265), TS1109 (158), TS7010 (133), TS7011 (123)
- TS2339: missing 53, extra 64

### TS2339 Work (NEW)
- [x] Fixed property access on `any` type (returns `any` without error)
- [x] Fixed property access on `error` type (suppresses cascading errors)
- [x] Resolve application type arguments with type env to enable distributive conditional narrowing
- [x] Substitute polymorphic `this` in call returns and merge interface/base intersections (fixes intersectionThisTypes extra TS2339)
- [x] Add fallback lowering for unresolved utility types `Pick` and `Exclude` in `get_type_from_type_reference`
- [x] Evaluate conditional constraints in mapped evaluation; handle `never` mapped keys as empty object
- [x] Resolve intersection type nodes via checker path (so utility fallbacks apply inside intersections)
- TS2339 extra errors reduced from 35 to 14
- [x] Implement TYPE_OPERATOR handling in `get_type_from_type_node` (keyof/etc) — `get_type_from_type_operator` added
- [x] Fix `intersectionWithIndexSignatures` TS2339 — no longer extra TS2339 errors
- [x] Flow assignment narrowing uses RHS node types (added control flow test)
- [x] Private identifier property access falls back to class owner type (static private members)
- [x] Assignment narrowing falls back to RHS literal/nullish types without node cache; updated flow tests
- [x] Match `this`/`super` reference bases in control flow for property assignment narrowing; added test
- [x] Loop label flow unions entry/back-edge types; added regression test
- [x] Const-aliased condition narrowing uses initializer for flow analysis; added test
- [x] Assertion predicate calls create flow nodes and narrow asserted targets; added test
- [x] Added missing TS2339 mode to `find-ts2339.mjs`
- [x] Catch clause variables default to `unknown` for narrowing; added TS2339 test
- [x] Enforce private identifier access by scope + receiver type (incl. `in` operator)
- [x] Check class expressions for computed `this` in member names; added tests

### Remaining TS2339 False Positives (pending re-run)
- Mixin classes: mixin type inference issues (intersection handling added in new expressions, unit tests pass, conformance tests need more investigation)
- Static index signatures: would require adding index signatures to CallableShape
- Re-run conformance to confirm private names/control-flow narrowing improvements

## Ready for Merge
No

## Notes
- Similar infrastructure to TS2564 (property init) - share patterns with Workers 1-2
- Control flow analysis already exists in `control_flow.rs` - extend it
- `./wasm/test.sh` failed with existing repo errors (BindResult, TemplateLiteralSpan, object_with_index signature) unrelated to TS2454 changes.
- Commit format: `[wasm] checker: implement TS2454 definite assignment analysis`
- Push to: `origin/worker/forge-3`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
- Conformance (500 tests, latest): Exact 105 (21.6%), Same count 126 (25.9%), 0 crashes
- TS2339 no longer in top 10 extra errors (was reduced from 35 to 20)
- Remaining TS2339 issues pending re-run; likely mixins + assertion predicates
- `get_type_from_type_operator` added for proper keyof/readonly/unique handling
- Tests: `./wasm/test.sh control_flow_tests`, `./wasm/test.sh test_ts2339_`
- `./scripts/ask-gemini.mjs` blocked: missing `GCP_VERTEX_EXPRESS_API_KEY`
- `./wasm/test.sh thin_checker_tests` fails with existing abstract class tests (2511 vs 2564)
- Latest run: `test_abstract_class_through_type_alias_2511` + `test_abstract_class_union_type_2511` expect 2511 but got 2564 (pre-existing)
