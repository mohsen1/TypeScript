
## Files to Modify

1. `wasm/src/checker/mod.rs` - Property resolution
2. `wasm/src/checker/types.rs` - If type structures need updates
3. Test files as needed

## Verification

Run after changes:
```bash
node wasm/differential-test/conformance-runner.mjs --max=200 -v 2>&1 | grep TS2339
```

Target: Reduce TS2339 false positives from 68 to <25.

## Status
Active

## Notes
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-3`
- Focus on false positives (extra TS2339).
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
- Implemented in `wasm/src/thin_checker.rs`, tests in `wasm/src/thin_checker_tests.rs`
- Added interface index signature parsing guard for type members (`wasm/src/thin_parser.rs`)
- Enabled class/interface declaration merging in `ThinBinderState`
- Resolved `import = require('module')` against ambient module exports
- Added default `tests/lib/lib.d.ts` loading in conformance harness scripts

Ready for Merge: No (merged)

## Current Task: TS2339 property access fixes (new assignment)

### Update (2026-01-09)
- Implemented mixin base instance extraction for heritage expressions and merged base properties (handles call-expression bases and type-parameter constructors).
- Preserved callable index signatures in type literal/interface lowering and interface merge logic.
- Added `test_mixin_inheritance_property_access` in `wasm/src/thin_checker_tests.rs`.
- Build: `./wasm/build-wasm.sh` (warnings only).
- Conformance `--max=200`: 2 Missing TS2339 lines:
  - `ambient/ambientDeclarationsPatterns_merging3.ts`
  - `async/es6/asyncWithVarShadowing_es6.ts` (Missing TS7031, TS2339)
  - Prior pre-change scan showed 0 TS2339 lines (delta +2 missing; no extra TS2339 in first 200).

### Failing Samples (extra TS2339)
- Latest scan (`node wasm/differential-test/find-ts2339.mjs --max=300 --samples=5`): none in first 300 tests.

### Emit Sites (thin_checker.rs)
- Property access miss → TS2339: `wasm/src/thin_checker.rs:4901` / `wasm/src/thin_checker.rs:4902`
  - Branch: `PropertyAccessResult::PropertyNotFound { .. }` → `error_property_not_exist_at`
- Diagnostic builder: `wasm/src/thin_checker.rs:7071` (`error_property_not_exist_at`)

### Next Steps
1. Re-run conformance with higher `--max` if needed to confirm failing set.
2. Inspect private name / mixin samples for property lookup path (likely class static/private handling in property access).
3. Trace property access in `get_type_of_property_access_expression` and related type resolution helpers for static/private members.

### Update (2026-01-10)
- Typed class expressions as constructor values so return-type inference and property access see base members.
- Extended heritage parsing to accept parenthesized/new expressions like `extends (new B2<number>().anon)`.
- Avoided emitting property access nodes when `.` is followed by a non-identifier token (prevents TS2339 on `this.`).
- Build: `./wasm/build-wasm.sh` (warnings only).
- Conformance: `node wasm/differential-test/conformance-runner.mjs classes/classExpressions --max=200 -v`.
- Scan: `node wasm/differential-test/find-ts2339.mjs --max=300 --samples=5` (0 extra TS2339 in first 300).

### Update (2026-01-10)
- Fixed TemplateExpression1 crash by guarding missing `}` in template spans and synthesizing a tail literal to avoid infinite loops.
- Added `test_thin_parser_unterminated_template_expression_no_crash` in `wasm/src/thin_parser_tests.rs`.
- Build: `./wasm/build-wasm.sh` (warnings only).
- Conformance: `node wasm/differential-test/conformance-runner.mjs es6/templates --max=200 -v` (no crashes).
- Test: `./wasm/test.sh test_thin_parser_unterminated_template_expression_no_crash`.

### Update (2026-01-10)
- Larger sweep: `node wasm/differential-test/conformance-runner.mjs es6/templates --max=1000 -v` (178 tests, WASM Crashed: 0).
- Repro: `tests/cases/conformance/es6/templates/TemplateExpression1.ts` (unterminated template expression).
- Crash path: `parse_template_expression` in `wasm/src/thin_parser.rs` during template span rescan; now guarded to emit TS1005 and synthesize a tail.

### Update (2026-01-10)
- Broader sweep: `node wasm/differential-test/conformance-runner.mjs --max=500` (497 tests run, 13 multi-file; WASM Crashed: 0).

### Update (2026-01-10)
- Added TS1160 `UNTERMINATED_TEMPLATE_LITERAL` diagnostic and parser reporting for unterminated template literals (template expressions, no-substitution, template literal types).
- Added `test_thin_parser_unterminated_template_literal_reports_ts1160` in `wasm/src/thin_parser_tests.rs`.
- Build: `./wasm/build-wasm.sh` (warnings only).
- Conformance: `node wasm/differential-test/conformance-runner.mjs es6/templates --max=200 -v` (TS1160 no longer missing; extra TS1160 in `templateStringInPropertyName2` and `templateStringInPropertyNameES6_2`).
- Test: `./wasm/test.sh test_thin_parser_unterminated_template_literal_reports_ts1160`.

### Update (2026-01-10)
- Larger crash sweep: `node wasm/differential-test/conformance-runner.mjs --max=1000` (993 tests run, 120 multi-file; WASM Crashed: 1).
- Crashed file: `classes/members/privateNames/privateNamesInterfaceExtendingClass.ts` with `Maximum call stack size exceeded`.

### Update (2026-01-10)
- Root cause: template literal property names were parsed as identifiers, leaving the closing backtick to be scanned as a new unterminated template literal (extra TS1160).
- Fix: in object literal property assignment, emit TS1136 and consume template literals as property names to keep the scanner in sync.
- Added `test_thin_parser_template_literal_property_name_no_ts1160` in `wasm/src/thin_parser_tests.rs`.
- Build: `./wasm/build-wasm.sh` (warnings only).
- Conformance: `node wasm/differential-test/conformance-runner.mjs es6/templates --max=200 -v` (extra TS1160 removed; extra errors down to 13).
- Test: `./wasm/test.sh test_thin_parser_template_literal_property_name_no_ts1160`.

Ready for Merge: No (merged 2026-01-10)
# Worker 3 Plan

## Current Assignment (2026-01-11)
- Continue TS2339 reduction: run `node wasm/differential-test/find-ts2339.mjs --max=500 --samples=5`, pick a sample, add a regression in `wasm/src/thin_checker_tests.rs`, and fix in `wasm/src/thin_checker.rs`.
- Run `./wasm/test.sh <new_test_name>` and report before/after TS2339 samples.
