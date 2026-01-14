# Worker 8 Task List

## Squad: Solver - Strictness & Subtyping

## Current Task
- [ ] Change `lower_type` to return `Error` instead of `Any` when resolution fails
- [ ] Identify all locations where solver defaults to `TypeId::ANY`

## Queue
- [ ] Harden `solve_subtype` logic to match tsc behavior
- [ ] Implement "Lawyer" layer for TypeScript quirks:
  - Function bivariance
  - Void return exceptions
  - Generic inference rules
- [ ] Test that stricter solver exposes real errors (will temporarily spike metrics)
- [ ] Document solver fallback behavior in `specs/SOLVER.md`

## Completed
- None

## Context
The solver is "too nice" - it defaults to `Any` when confused. In TypeScript, `Any` disables type checking, hiding all downstream errors.

By switching the default from `Any` to `Unknown`/`Error`, we will:
1. Temporarily spike error counts (breaking the build)
2. Expose exactly where the binder/solver is failing
3. Allow real fixes instead of silent `Any` poisoning

### Key Files
- `src/solver/mod.rs` - main solver entry
- `src/solver/subtype.rs` - subtyping logic
- `specs/SOLVER.md` - specification

### Goal
Stop silent `Any` fallback. Better to be too strict (extra errors) than unsound (missing errors).
