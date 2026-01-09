# Worker 1 Plan - Squad Forge

## Mission
Implement TS2304 cannot find name diagnostics.

Status: Active
Priority: 1

## Current Assignment
Implement missing name diagnostics for unresolved identifiers.

**Error Code:** TS2304 - "Cannot find name 'X'"

**Impact:** 138 conformance tests affected

### Steps
1. **Identify emit sites** in `thin_checker.rs` for identifier resolution and missing symbol paths.
2. **Add tests first** in `wasm/src/thin_checker_tests.rs`:
   - Unresolved identifier in expression
   - Unresolved type reference
   - Unresolved enum/member access should not emit TS2304 if TS2339 is expected
3. **Emit TS2304** when a name lookup fails and no other higher-priority diagnostic applies.
4. **Run conformance tests** and report delta.

### Key Files
- `wasm/src/thin_checker.rs`
- `wasm/src/checker/types/diagnostics.rs`
- `wasm/src/thin_checker_tests.rs`

### Success Criteria
- TS2304 emitted for missing names
- No increase in extra errors

## Task Queue
(empty - single focused task)

## Completed
- TS2564 property initialization tracking and tests (merged).
- Implemented TS2564 property initialization check using type annotations when symbol types are `any`/`unknown`.
- Added TS2564 tests for required property errors and `undefined` union exemption.
- Ran `./wasm/test.sh` (fails at `cli::driver_tests::compile_class_with_generic_constructor`).
- Rebuilt WASM and reran conformance (debug runner): 79/446 exact, 102/446 same count; TS2564 no longer in top missing. OOB crashes persist for private name/decorators/etc.

## Ready for Merge
No

## Notes
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: implement TS2304 missing name diagnostics`
- Push to: `origin/worker/forge-1`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
