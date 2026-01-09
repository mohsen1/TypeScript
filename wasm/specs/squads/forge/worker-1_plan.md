# Worker 1 Plan - Squad Forge

## Mission
Implement TS2454 variable used before assigned diagnostics.

Status: Active
Priority: 1

## Current Assignment
Implement TS2454 "Variable 'x' is used before being assigned" error.

**Error Code:** TS2454 - "Variable 'x' is used before being assigned"

**Impact:** 573 conformance tests affected

### Steps
1. **Track variable assignments** in control flow analysis
2. **Before each variable read**, check if definitely assigned
3. **Handle conditional branches** (if/else, switch, loops)
4. **Run conformance tests** and report delta.

### Key Files
- `wasm/src/thin_checker.rs`
- `wasm/src/checker/control_flow.rs`
- `wasm/src/thin_checker_tests.rs`

### Success Criteria
- TS2454 emitted for unassigned variable reads
- Control flow properly tracks assignments across branches

## Task Queue
(empty - single focused task)

## Completed
- TS2564 property initialization tracking and tests (merged).
- Implemented TS2564 property initialization check using type annotations when symbol types are `any`/`unknown`.
- Added TS2564 tests for required property errors and `undefined` union exemption.
- Ran `./wasm/test.sh` (fails at `cli::driver_tests::compile_class_with_generic_constructor`).
- Rebuilt WASM and reran conformance (debug runner): 79/446 exact, 102/446 same count; TS2564 no longer in top missing. OOB crashes persist for private name/decorators/etc.
- Added TS2304 coverage for unresolved identifiers and type references (including nested function types) plus TS2339-only property access check.
- Emitted TS2304 for missing names inside type aliases and interface members; added type-node traversal for complex types.
- Ran `./wasm/test.sh missing_type_reference`, `./wasm/test.sh missing_identifier_emits_2304`, `./wasm/test.sh missing_property_access_emits_2339_not_2304`.
- Rebuilt WASM and reran conformance (debug runner): 77/446 exact, 100/446 same count; TS2304 extra count 145 (up +2); OOB crashes persist.
- Ran Docker conformance `run-conformance.sh --max=1000 --workers=6`: 879 tests run, 15.5% exact (136), 17.4% same count (153), 223 crashes, 121 skipped. Top missing: TS2705(37), TS2322(26), TS2339(22), TS1109(16), TS2524(15). Top extra: TS2304(192), TS2355(83), TS1005(64), TS7010(48), TS2339(31).
- Docker conformance `run-conformance.sh --all --workers=14` OOM/exit 137 at 0% progress.
- Ran Docker conformance `run-conformance.sh --all --workers=6`: 4928 tests run, 17.6% exact (869), 20.2% same count (994), 1936 crashes, 727 skipped. Top missing: TS2322(135), TS7010(84), TS2339(76), TS2304(66), TS2695(46). Top extra: TS2304(494), TS1005(288), TS1109(171), TS7011(165), TS7010(164).
- Fixed compilation error in solver/subtype.rs (s_sym undefined in TypeQuery match arm).
- Added built-in utility type handling to reduce false TS2304 errors (Partial, Required, Pick, Omit, Record, Exclude, Extract, NonNullable, ReturnType, Parameters, etc.).

## Ready for Merge
Yes

## Notes
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: implement TS2304 missing name diagnostics`
- Push to: `origin/worker/forge-1`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`

## Resume Notes
- Branch: `worker/forge-1`
- Unit tests: 4921 total, 4851 passed, 69 failed, 1 skipped.
- Merged type parameter scope fix from EM (origin/squad/forge).
- TS2304 work complete: added utility type handling to reduce false positives.
- TS2454 implementation done:
  - Fixed `get_type_of_call_expression` to process arguments even when callee is `any`
  - This ensures definite assignment checking for args like `console.log(x)`
  - 7 tests passing:
    1. Variable used before assigned (error)
    2. Variable assigned before use (no error)
    3. Variable initialized at declaration (no error)
    4. Function parameter (no error)
    5. Assigned in both if/else branches (no error)
    6. Assigned in only if branch (error)
    7. Var declaration (no error - only let/const)
  - Control flow analysis working for conditional branches
- Next: Run conformance tests to measure impact on 573 affected tests.
