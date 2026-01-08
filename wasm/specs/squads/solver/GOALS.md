# Squad Solver Goals

Updated: 2026-01-08

## Current Milestone
Solver Hardening - Make the type inference engine correct and robust.

## Objectives (Ranked)

1. **Generic Inference Hardening**
   - Context: Solver is the correctness bottleneck per Project Direction. Generic inference from usage and context-sensitive typing are critical gaps.
   - Success Criteria: All `infer_tests.rs` pass; no panics when compiling `redux` or `lodash` type definitions.
   - Key Files: `wasm/src/solver/infer.rs`, `wasm/src/solver/infer_tests.rs`
   - Estimated Complexity: High

2. **Conditional Type Evaluation**
   - Context: Distributive conditional types over unions are where most compilers fail. Must match TypeScript semantics exactly.
   - Success Criteria: All `evaluate_tests.rs` conditional tests pass; template literal inference works correctly.
   - Key Files: `wasm/src/solver/evaluate.rs`, `wasm/src/solver/evaluate_tests.rs`
   - Estimated Complexity: High

3. **Structural Compatibility (Variance)**
   - Context: Variance handling (covariance for results, contravariance for parameters) must be correct for all edge cases.
   - Success Criteria: All `subtype_tests.rs` pass; mapped type modifiers work correctly.
   - Key Files: `wasm/src/solver/subtype.rs`, `wasm/src/solver/subtype_tests.rs`
   - Estimated Complexity: Medium

4. **Circular Constraint Handling**
   - Context: Circular constraints in `extends` clauses can cause infinite loops or incorrect results.
   - Success Criteria: Circular constraint tests pass without stack overflow or timeout.
   - Key Files: `wasm/src/solver/infer.rs`, `wasm/src/solver/constraints.rs`
   - Estimated Complexity: Medium

## Anti-Priorities
- New LSP features (unless they expose a Solver bug)
- CLI argument parsing
- Performance micro-optimizations (unless we regress significantly)
- Parser/AST changes

## Cross-Squad Dependencies
- Tools squad may find Solver bugs via emitter tests; coordinate on fixes
- Type representations (`wasm/src/types/`) are shared; coordinate large changes

## Notes to EM
- Read `wasm/specs/SOLVER.md` for solver architecture details
- Read `wasm/specs/TS_UNSOUNDNESS_CATALOG.md` for known TypeScript unsoundness
- Use Docker for tests: `./wasm/test.sh`
- Focus on test coverage first, then fixes

## Squad Status
- Last EM Report: Not yet started
- Workers Active: 0/3
- Branches Pending Merge: None
- Current Focus: Awaiting EM assignment
- Blockers: None
