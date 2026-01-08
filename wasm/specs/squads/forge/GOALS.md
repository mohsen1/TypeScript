# Squad Forge Goals

Updated: 2026-01-08

Priority: 1

## Current Milestone
Phase 8 - Conformance, Convergence, and Hardening: solver correctness in the integrated pipeline, driven by conformance tests.

## Project Direction Alignment
- Strategic shift: integration and correctness across the pipeline; conformance tests drive work.
- Top priority: Solver is the correctness bottleneck; fix inference and subtype gaps before new features.

## Focus Areas (Director can reassign)
- `wasm/src/solver/` - Type inference, constraint solving
- `wasm/src/checker/` - Type checking logic
- `wasm/src/binder/` - Symbol binding, scope analysis
- `wasm/src/types/` - Type representations

## Objectives (Ranked)

1. **Generic Inference Hardening**
   - Context: Solver is the correctness bottleneck per Project Direction
   - Success Criteria: Inference from usage and context-sensitive typing match `tsc`, including circular constraints in `extends` clauses; redux/lodash-type suites compile without panics
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
   - Success Criteria: Compile a non-trivial generic library (e.g., redux/lodash types) without panics and track conformance pass-rate deltas
   - Key Files: `checker/mod.rs`, `solver/mod.rs`, `binder/mod.rs`
   - Estimated Complexity: Medium

## Anti-Priorities
- New LSP features (unless they expose a Solver bug)
- CLI argument parsing
- Performance micro-optimizations
- New AST nodes or isolated features outside solver correctness

## Cross-Squad Dependencies
- Anvil squad conformance/emitter runs may surface solver bugs; coordinate on failure triage and ownership

## Notes to EM
- Re-anchor worker tasks to conformance-driven integration; avoid feature work that does not close solver correctness gaps.
- Read `wasm/specs/SOLVER.md` for solver architecture
- Read `wasm/specs/TS_UNSOUNDNESS_CATALOG.md` for known TypeScript unsoundness
- Use Docker for tests: `./wasm/test.sh`

## Squad Status
- Last EM Report: 2026-01-08
- Workers Active: 5/5
- Branches Pending Merge: worker/forge-1 (blocked by management-file edits)
- Current Focus: conditional type evaluation (function + template literal inference), subtype `this` variance, end-to-end generic regressions, optional/variadic tuple inference
- Direction: Conformance-first integration; solver correctness before feature work
- Blockers: `emitter_edge_case_tests::test_export_assignment_suppresses_other_exports` failure; worker/forge-1 contains management-file edits
