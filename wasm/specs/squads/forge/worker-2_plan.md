# Worker 2 Plan - Squad Forge

## Mission
TS2339 property access improvements (type parameter constraints + class instance expansion).

Status: Complete
Priority: 1

## Current Assignment
Improve TS2339 property access, focusing on:
- Resolve property access on constrained type parameters
- Expand Application types using class instance types for assignability
- Prevent recursion when resolving self-referential class instance types
- Update generic-constraint tests

### Steps
1. Update property access resolution in checker + solver for TypeParameter constraints.
2. Ensure type_env uses class instance types + params for Application expansion, with recursion guard.
3. Update thin_checker_tests cross-scope generic constraints to expect no errors.
4. Run `./wasm/test.sh` and record failures.

### Key Files
- `wasm/src/thin_checker.rs`
- `wasm/src/solver/operations.rs`
- `wasm/src/checker/context.rs`
- `wasm/src/thin_checker_tests.rs`

## Completed
- Implemented property access on constrained type parameters in checker and solver.
- Stored class instance types + type params in type_env to expand Application types.
- Added recursion guard for class instance type resolution to avoid stack overflow.
- Updated cross-scope generic constraints test to expect no errors.
- Ran `./wasm/test.sh` (fails: TS2792 module resolution in multi-file import tests).

## Ready for Merge
Yes

## Notes
- Full test run failing in `cli::driver_tests::compile_multi_file_project_with_imports`
  and `cli::driver_tests::compile_multi_file_project_with_default_and_named_imports` (TS2792).
- Must rerun tests after module resolution fix.
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
