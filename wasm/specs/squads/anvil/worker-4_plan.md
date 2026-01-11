# Anvil Worker 4 - Mapped Type Recursion Guard

## Current Assignment (2026-01-11) - Recursive Mapped Types Property Access Guard

Status: Completed

- [x] Add recursion guard/fallback for property access on recursive mapped types.
- [x] Add regression test (Transform<T> property access).
- [x] Run targeted conformance (types/mapped) and report crash delta.
- Queue: TS2769 variadic tuple false positives (see follow-up sections).

## Follow-up (2026-01-11) - Property Access Guard Implementation

Status: COMPLETED

### Changes

- Added recursion guard for mapped/application property access in `wasm/src/solver/operations.rs`.
- Capped mapped eval depth and wrapped property access with instantiation depth guard in `wasm/src/thin_checker.rs`.
- Avoided recursive expansion of mapped property types in `evaluate_mapped_type_with_resolution_inner`.
- Regression test: `test_recursive_mapped_property_access_no_crash`.

### Tests

- `./wasm/test.sh test_recursive_mapped_property_access_no_crash` (PASS)

### Conformance (types/mapped, max=200)

- `node wasm/differential-test/conformance-runner.mjs types/mapped --max=200 -v`
- Crashes: 2 → 0 (`mappedTypes2.ts`, `recursiveMappedTypes.ts`)
- Exact match: 3 → 5
- Same error count: 3 → 5
- `recursiveMappedTypes.ts` still missing TS2456/TS2313/TS2589/TS2502/TS2615; extra TS2322/TS2339/TS2304

Ready for Merge: Yes

## Follow-up (2026-01-11) - Recursive Mapped Types Crash Verification

Status: COMPLETED

### Conformance

- `node wasm/differential-test/conformance-runner.mjs types/mapped --max=200 -v`
- Crashed: `types/mapped/mappedTypes2.ts`, `types/mapped/recursiveMappedTypes.ts` (unreachable)

### Crash Stack (Node + wasm)

```
RuntimeError: unreachable
    at wasm://wasm/0082356e:wasm-function[1689]:0x1ede7e
    at wasm://wasm/0082356e:wasm-function[1565]:0x1ec242
    at wasm://wasm/0082356e:wasm-function[156]:0xd194f
    at wasm://wasm/0082356e:wasm-function[1284]:0x1d0bc2
    at wasm://wasm/0082356e:wasm-function[179]:0xdef74
    at wasm://wasm/0082356e:wasm-function[289]:0x12db93
    at wasm://wasm/0082356e:wasm-function[179]:0xdde3f
```

### Minimal Repro (crashes)

```
type Transform<T> = { [K in keyof T]: Transform<T[K]> };
interface Product { users: string[]; }
declare var product: Transform<Product>;
product.users;
```

### Notes / Proposed Fix Direction

- Crash only triggers on property access of recursive mapped types; declaring the type without property access is ok.
- Likely in property access evaluation of mapped types (`wasm/src/solver/operations.rs` `PropertyAccessEvaluator` + `evaluate_type`).
- Proposed direction: add a recursion guard in property access evaluation (for `TypeKey::Mapped`/`TypeKey::Application`) and/or short-circuit recursive mapped property access to `TypeId::ANY`/`TypeId::ERROR` to avoid panic until full recursive mapped semantics are implemented.

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

## Follow-up (2026-01-09) - TS2769 Overload Matching False Positives

**Mission**: Fix TS2769 overload mismatch false positives involving variadic tuples.

**Status**: IN PROGRESS

### Checklist

- [x] Reproduce in `variadicTuples1.ts` and other samples
- [x] Trace `collect_call_argument_types_with_context` and overload resolution in `thin_checker.rs` / `solver`
- [x] Implement fix for spread/tuple expansion or overload matching
- [x] Add regression test for TS2769 case
- [x] Run `./wasm/test.sh` for new test
- [ ] Investigate remaining TS2769 extras in `variadicTuples1.ts` (23 occurrences after tuple conformance run)

### Notes

- Gemini suggests expanding tuple elements for spread arguments, including type parameter constraints and type refs.
- Implemented variadic tuple rest tail handling in call evaluator and spread type resolution for spreads.
- Tests: `./wasm/test.sh test_call_spread_tuple_type_param`, `./wasm/test.sh test_call_tuple_rest_with_fixed_tail` (PASS).
- Conformance: `node wasm/differential-test/conformance-runner.mjs types/tuple --verbose` (variadicTuples1 still has extra TS2769).
- Built WASM package for conformance: `./wasm/build-wasm.sh` (warnings only).
- Avoid committing `.role/AGENTS.md` change (expected local modification).

Ready for Merge: No (merged 2026-01-10; conformance still shows extra TS2769 in variadicTuples1)

## Follow-up (2026-01-10) - TS2769 Variadic Tuple Rest Parameters

**Mission**: Fix TS2769 false positives for variadic tuple rest parameters with trailing fixed elements.

**Status**: COMPLETED

### Checklist

- [x] Analyze TS2769 errors in `variadicTuples1.ts` (15 false positives identified)
- [x] Create minimal test case reproducing the issue
- [x] Trace root cause in `wasm/src/solver/operations.rs::rest_tuple_inference_target`
- [x] Implement fix to account for trailing fixed elements
- [x] Add regression test: `test_variadic_tuple_rest_param_no_ts2769`
- [x] Run conformance on types/tuple to measure impact

### Root Cause

For signature `foo<T extends unknown[]>(x: number, ...args: [...T, number]): T`:
- Call `foo(1, 2)` should infer T = [] (rest args [2] match [...[], number])
- Bug: `rest_tuple_inference_target` was inferring T from ALL remaining arguments
- It didn't account for trailing fixed elements after the variadic part
- Result: tried to infer T = [2] instead of T = [], causing type mismatch

### Fix

Modified `rest_tuple_inference_target` in `wasm/src/solver/operations.rs:515-573`:
1. Count trailing fixed elements after the variadic type parameter
2. Calculate `infer_count = rest_arg_count - trailing_count`
3. Build inference tuple from `arg_types[start_index..start_index+infer_count]`

### Test Results

- Minimal test: 3 TS2769 errors eliminated (3 → 0)
- variadicTuples1.ts: 5 TS2769 errors eliminated (15 → 10)
- Conformance (types/tuple, max 200): TS2769 extras reduced (6 → 5 occurrences)
- Regression test: `cargo test test_variadic_tuple_rest_param_no_ts2769` (PASS)

### Files Modified

- `wasm/src/solver/operations.rs` - Fixed variadic tuple rest parameter type inference
- `wasm/src/thin_checker_tests.rs` - Added regression test

Ready for Merge: No (merged 2026-01-10)

## Follow-up (2026-01-10) - TS2769 Variadic Tuple Optional Tails

**Mission**: Reduce TS2769 false positives for variadic tuple calls (variadicTuples1 + two extra samples).

**Status**: COMPLETED (partial reduction; remaining TS2769 listed)

### Checklist

- [x] Gather TS2769 samples in `variadicTuples1.ts`, `variadicTuples2.ts`, `typeInferenceWithTupleType.ts`
- [x] Fix spread argument expansion to resolve refs/constraints for tuple spreads
- [x] Fix array literal tuple typing for variadic contexts + spread elements
- [x] Make variadic rest inference optional-tail aware
- [x] Fix tuple subtyping for rest+tail alignment + rest placeholder inference
- [x] Add regression test: `test_variadic_tuple_optional_tail_inference_no_ts2769`
- [x] Tests: `./wasm/test.sh test_variadic_tuple_optional_tail_inference_no_ts2769` (PASS)
- [x] Conformance: `node wasm/differential-test/conformance-runner.mjs types/tuple --max=200 -v`

### Root Cause Notes

- Spread args only expanded direct tuple types; refs/type params/readonly wrappers failed to expand, leading to bogus overload mismatch.
- Array literal tuple typing reused contextual rest flags for every element and typed spread elements as `any`, producing malformed tuples.
- Variadic rest inference always stripped trailing elements (even optional tails), mis-inferred `T` for `...T, number?`.
- Tuple subtype checks treated tail elements as part of the variadic segment and inferred `T` as `[...U]` instead of `U`.

### Fix Summary

- `collect_call_argument_types_with_context` resolves spread types before tuple expansion.
- `get_type_of_array_literal` types spread elements via their expression and only marks rest when the literal has spreads.
- `rest_tuple_inference_target` and `constrain_tuple_types` allow optional tail omission and collapse single rest-element tails to avoid `[...U]`.
- Tuple subtype tail alignment respects optional tails and avoids consuming tail elements in the variadic section.

### Conformance Delta (types/tuple, max 200)

- Extra TS2769 occurrences: 6 → 5
- `variadicTuples1.ts`: WASM errors 90 → 80 (TS2769 count 18 → 10)
- `variadicTuples2.ts`: WASM errors 59 → 49 (TS2769 count 12 → 8)
- `restTupleElements1.ts`: extra TS2769 removed

### Remaining TS2769 Samples (post-fix)

- `variadicTuples1.ts`: `foo1(...t1, true, 42, 43, 44)`, `foo1(...t1, ...t2, 42, 43, 44)`, `fm1([...])`, `ft([...], [...])` calls
- `variadicTuples2.ts`: `foo('blah1', 1)` and related overloads, `fn1([1])`, `fn2([1])`
- `typeInferenceWithTupleType.ts`: `f1(undefined as ["a"[], "b"[]])`, `f2(undefined as ["a"[], "b"[]])`

### Files Modified

- `wasm/src/thin_checker.rs`
- `wasm/src/solver/operations.rs`
- `wasm/src/solver/subtype.rs`
- `wasm/src/thin_checker_tests.rs`

Ready for Merge: No (merged 2026-01-11)

## Follow-up (2026-01-11) - TS2769 Variadic Tuple Spreads

**Mission**: Eliminate remaining TS2769 false positives from variadic tuple spreads.

**Status**: COMPLETED

### Changes

- Use unary expr data to read spread element expressions (call args + array literals).
- Expand tuple spreads in call argument collection; array literal spread elements use element types outside tuple context.
- Resolve contextual tuple types via `resolve_type_for_property_access` to allow mapped tuple contexts.
- Treat single-signature callables as non-overload calls (avoid TS2769 on non-overloads).

### Tests

- `./wasm/test.sh test_overload_call_handles_tuple_spread_params`
- `./wasm/test.sh test_variadic_tuple_optional_tail_inference_no_ts2769`

### Conformance (types/tuple, max=200)

- `node wasm/differential-test/conformance-runner.mjs types/tuple --max=200 -v`
- Exact Match: 9 (26.5%); Same Error Count: 10 (29.4%); Crashes: 0
- `variadicTuples1.ts`: WASM errors 72; TS2769 count 0
- `variadicTuples2.ts`: WASM errors 46; TS2769 count 0

### Remaining false positives

- `variadicTuples1.ts`: `fm1([...])` (TS2345 at line 123)
- `variadicTuples1.ts`: `ft([...], [...])` (TS2345 at lines 356-359)

Ready for Merge: Yes

## Follow-up (2026-01-09) - TS2339 False Positives (Class-Like Extends)

**Mission**: Remove extra TS2339 errors for class inheritance through constructor-returning expressions.

**Status**: COMPLETED

### Checklist

- [x] Reproduced TS2339 extras via `node wasm/differential-test/find-ts2339.mjs --max=400 --samples=10`
- [x] Traced root causes in `thin_checker` (constructor return types, union base shapes, class expressions) and scanner
- [x] Implemented fixes for constructor-returning base types and class expressions
- [x] Added regression tests for constructor expressions + parse-error property access
- [x] Rebuilt wasm + reran TS2339 scan (0 extras in first 400 files)

### Root Causes

1. `get_class_instance_type_inner` merged base properties only for direct class symbols; constructor-returning expressions (e.g., `getBase()`) returned `Ref`/`TypeParameter`/`TypeQuery` types that were ignored, and unions from overloaded constructors were dropped.
2. `get_type_of_node` lacked class expression typing, so factory-returned classes were inferred as `any`.
3. Scanner retained `token_value` for punctuation tokens, causing `this.` parse errors to produce bogus `this` property names and TS2339.

### Fixes

- Resolve constructor-returning base types through `Ref`/`TypeQuery`/type parameter constraints and meta-types, then collect construct-signature return types.
- Merge base instance properties for unions/intersections of constructor return types.
- Type class expressions as constructor types so factory-returned classes preserve base instance properties.
- Clear scanner `token_value` per token and guard against empty property names in property access.

### Tests

- `./wasm/test.sh test_class_extends_constructor_expression_includes_base_props` (PASS)
- `./wasm/test.sh test_incomplete_property_access_no_ts2339` (PASS)

### Conformance Delta (TS2339)

**Before** (`find-ts2339.mjs --max=400 --samples=10`):
- `classes/classDeclarations/classAbstractKeyword/classAbstractCrashedOnce.ts` (1 extra)
- `classes/classDeclarations/classExtendingClassLikeType.ts` (6 extra)
- `classes/classExpressions/genericClassExpressionInFunction.ts` (3 extra)

**After** (`find-ts2339.mjs --max=400 --samples=10`):
- 0 extra TS2339 in first 400 files

Ready for Merge: No (merged)
