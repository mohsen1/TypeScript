# Anvil Worker 4 - Mapped Type Recursion Guard

## Operation Conformance Assignment

**Mission**: Fix stack overflow in `types/mapped/recursiveMappedTypes.ts`.

**Status**: COMPLETED

## Task Checklist

- [x] Reproduce stack overflow via conformance runner
- [x] Trace mapped type recursion in solver/thin_checker
- [x] Add recursion guard + memoization
- [x] Add regression test
- [x] Run conformance before/after and capture metrics

## Conformance Metrics (types/mapped)

| Metric | Before | After |
| --- | --- | --- |
| Files Found | 25 | 25 |
| Tests Run | 25 | 25 |
| Exact Match | 4 (16.0%) | 4 (16.0%) |
| Same Error Count | 4 (16.0%) | 5 (20.0%) |
| WASM Crashed | 1 | 0 |
| Tests with missing errors | 10 (40.0%) | 10 (40.0%) |
| Tests with extra errors | 18 (72.0%) | 19 (76.0%) |

**Before** (types/mapped, --max=200):
- Crashed: `types/mapped/recursiveMappedTypes.ts` (Maximum call stack size exceeded)

**After** (types/mapped, --max=200):
- No crashes

## Files Modified

- `wasm/src/solver/evaluate.rs`
- `wasm/src/thin_checker.rs`
- `wasm/src/checker/context.rs`
- `wasm/src/thin_checker_tests.rs`

## Notes

- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-4`
- Ready for Merge: No (merged 2026-01-09)

## Follow-up (2025-01-09) - Recursive Mapped Types

**Mission**: Add mapped type resolution guard/memoization and deepen regression coverage.

**Status**: COMPLETED

### Checklist

- [x] Re-ran conformance: `node wasm/differential-test/conformance-runner.mjs types/mapped --max=200 -v` (no crash; `recursiveMappedTypes.ts` still missing errors)
- [x] Re-ran conformance (post-change): `node wasm/differential-test/conformance-runner.mjs types/mapped --max=200` (no crashes; metrics unchanged)
- [x] Added mapped eval cache + guard in `thin_checker` mapped resolution
- [x] Added regression test: `test_recursive_mapped_type_list_widget_guard`
- [x] Tests: `./wasm/test.sh test_recursive_mapped_type_list_widget_guard` (PASS)

## Resume Notes

- Branch: `worker/anvil-4`
- Last work: mapped type resolution guard + memoization (thin checker), regression test for ListWidget recursion.
- Latest conformance: `node wasm/differential-test/conformance-runner.mjs types/mapped --max=200` (0 crashes; metrics unchanged).
- Regression test: `./wasm/test.sh test_recursive_mapped_type_list_widget_guard` (PASS).

### Files Touched

- `wasm/src/thin_checker.rs` (mapped type resolution guard + cache)
- `wasm/src/checker/context.rs` (mapped eval cache/set fields)
- `wasm/src/thin_checker_tests.rs` (ListWidget recursion test)

### Known Gaps (recursiveMappedTypes.ts)

- Missing diagnostics in conformance: TS2456, TS2313, TS2589, TS2502, TS2615.
- Likely areas: type alias circularity, circular type parameter constraints, deep instantiation limits, mapped type self-reference.

### Next Steps (if continuing)

1. Trace why `type Recurse = { [K in keyof Recurse]: Recurse[K] }` does not emit TS2456/TS2313.
2. Add diagnostics for circular constraints/type aliases in `ThinCheckerState` (look at symbol resolution guards and alias type computation).
3. Ensure depth/excessive instantiation errors (TS2589) surface for recursive mapped types.

## Follow-up (2026-01-09) - Circular Type Alias TS2456 Detection

**Mission**: Add TS2456 (circular type alias) diagnostic emission for self-referential type aliases.

**Status**: IN PROGRESS

### Checklist

- [x] Investigated how circular type alias detection should work
- [x] Traced type alias resolution in `thin_checker.rs` (`get_type_of_symbol` → `symbol_resolution_set`)
- [x] Added TS2456 emission in circular reference detection (`get_type_of_symbol`)
- [x] Added regression test: `test_circular_type_alias_ts2456` (PASS)
- [x] Fixed unrelated bug in `solver/subtype.rs` (dead code referencing undefined variable)
- [ ] Conformance test for `recursiveMappedTypes.ts` - TS2456 still not firing for all cases

### Files Modified

- `wasm/src/thin_checker.rs` (TS2456 emission in circular symbol resolution)
- `wasm/src/solver/subtype.rs` (fixed dead code bug)
- `wasm/src/thin_checker_tests.rs` (circular type alias test)

### Notes

- The basic circular type alias detection works for simple cases (`type Recurse = { [K in keyof Recurse]: Recurse[K] }` in unit test).
- Conformance tests show TS2456 is still marked as "missing" for `recursiveMappedTypes.ts`. This may be due to:
  1. Complex mapped type resolution paths that bypass symbol resolution
  2. Different error reporting locations (TSC may report multiple TS2456 while we only report once)
  3. Additional circular patterns not yet covered by our detection
- Pre-existing issues (not caused by this work):
  - Driver test `compile_class_with_generic_constructor` fails with TS2322 errors
  - Conformance tests `mappedTypes2.ts` and `recursiveMappedTypes.ts` crash with "unreachable" (from upstream merge)

Ready for Merge: No (partial TS2456 implementation; needs investigation of pre-existing crashes)
