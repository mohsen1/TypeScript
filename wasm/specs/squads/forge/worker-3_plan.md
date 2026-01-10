# Worker 3 Plan - Squad Forge

## Mission
Reduce TS2339 false positives via control flow narrowing.

Status: Active
Priority: 1

## Current Assignment
Extend control-flow narrowing for property access after `in`/`typeof`/`instanceof` guards.

**Error Code:** TS2339 - "Property 'X' does not exist on type 'Y'"

**Impact:** 142 conformance tests affected (control-flow narrowing cases)

### Steps
1. **Guard narrowing:** confirm `in`/`typeof`/`instanceof` paths update flow types in `control_flow.rs`.
2. **Reference matching:** ensure property chains (including `this`/`super`) are matched for narrowing.
3. **Add tests** in `wasm/src/checker/control_flow_tests.rs` for `in`/`typeof` guards and property access.
4. **Run focused tests** (`./wasm/test.sh control_flow_tests`) and report delta.

### Key Files
- `wasm/src/checker/control_flow.rs`
- `wasm/src/thin_checker.rs`
- `wasm/src/thin_checker_tests.rs`

### Success Criteria
- TS2339 extra errors reduced for control-flow narrowing cases
- No regressions in existing TS2339 tests

## Task Queue
(empty - single focused task)

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

### Remaining TS2339 False Positives (pending re-run)
- Mixin classes: mixin type inference issues (intersection handling added in new expressions, unit tests pass, conformance tests need more investigation)
- Static index signatures: would require adding index signatures to CallableShape
- Assertion type predicates: 1 test
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
