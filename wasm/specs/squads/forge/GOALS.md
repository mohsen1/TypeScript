# Squad Forge Goals

Updated: 2026-01-08

Priority: 1

## Current Milestone
Phase 8 - Conformance, Convergence, and Hardening: Type-system correctness in the integrated pipeline.

## Focus Areas (Director can reassign)
- `wasm/src/solver/` - Type inference, constraint solving
- `wasm/src/checker/` - Type checking logic
- `wasm/src/binder/` - Symbol binding, scope analysis
- `wasm/src/types/` - Type representations

## Objectives (Ranked)

1. **Generic Inference Hardening**
   - Context: Solver is the correctness bottleneck per Project Direction
   - Success Criteria: Inference from usage and context-sensitive typing match `tsc`, including circular constraints in `extends` clauses
   - Key Files: `solver/infer.rs`, `solver/infer_tests.rs`
   - Estimated Complexity: High

2. **Conditional Type Evaluation**
   - Context: Distributive conditional types over unions must match TypeScript
   - Success Criteria: `solver/evaluate.rs` handles distributive conditionals and template literal inference without regressions
   - Key Files: `solver/evaluate.rs`, `solver/evaluate_tests.rs`
   - Estimated Complexity: High

3. **Structural Compatibility (Variance)**
   - Context: Variance handling must be correct for all edge cases
   - Success Criteria: Covariant returns and contravariant parameters match `tsc` across subtype edge cases
   - Key Files: `solver/subtype.rs`, `solver/subtype_tests.rs`
   - Estimated Complexity: Medium

4. **End-to-End Validation**
   - Context: Stop adding AST nodes; compile real code end-to-end
   - Success Criteria: Compile a non-trivial generic library (e.g., redux/lodash types) without panics
   - Key Files: `checker/mod.rs`, `solver/mod.rs`, `binder/mod.rs`
   - Estimated Complexity: Medium

## Anti-Priorities
- New LSP features (unless they expose a Solver bug)
- CLI argument parsing
- Performance micro-optimizations
- New AST nodes or isolated features outside solver correctness

## Cross-Squad Dependencies
- Anvil squad may find Forge bugs via emitter tests; coordinate on fixes

## Notes to EM
- Read `wasm/specs/SOLVER.md` for solver architecture
- Read `wasm/specs/TS_UNSOUNDNESS_CATALOG.md` for known TypeScript unsoundness
- Use Docker for tests: `./wasm/test.sh`

## Squad Status
- Last EM Report: Not yet started
- Workers Active: 0/5
- Branches Pending Merge: None
- Current Focus: Awaiting EM assignment
- Blockers: None
