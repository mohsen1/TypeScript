# Worker 8 Task List

## Current Task
- [ ] Implement "Lawyer" layer for Any propagation (referenced in specs/SOLVER.md)
  - Read `specs/SOLVER.md` to understand requirements
  - Create `wasm/crates/swc_typescript/src/solver/lawyer.rs`
  - Implement rules for when Any can silence errors
  - Any should NOT silence structural mismatches unless explicitly required

## Queue
- [ ] Harden generic subtype checking
  - Fix cases where generic type parameters are incorrectly unified
  - Handle covariant/contravariant variance correctly
  - Test with complex generic constraints
- [ ] Add subtype strictness flags
  - Allow strict mode that rejects borderline cases
  - Use this for lib.d.ts type checking
  - Verify no regressions in valid code

## Completed
(none yet)
