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
- `wasm/src/checker/types/diagnostics.rs`
- `wasm/src/thin_checker_tests.rs`

### Success Criteria
- TS2339 missing errors reduced
- Extra errors do not increase (no regressions)

## Task Queue
- Add targeted tests for union property access and `any`/`unknown` behavior.
- Confirm behavior for property access on intersection types.
- Verify no regressions in existing TS2339 tests.

## Completed
- Added 5 additional edge case tests for TS2564:
  - Optional properties (?)
  - Definite assignment assertion (!)
  - Properties with initializers
  - Static properties
  - Simple constructor assignment
- Fixed missing BindResult import in lib.rs
- Fixed format! macro usage with format_message helper
- Fixed NodeList vs Vec type mismatch in find_constructor_body call

Note: Worker 1 had already implemented comprehensive TS2564 checking including
control flow analysis. My contribution adds complementary edge case tests and
compilation fixes.

## Ready for Merge
No

## Notes
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: improve TS2339 property access diagnostics`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
