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
