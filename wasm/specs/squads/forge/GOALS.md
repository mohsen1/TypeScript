# Squad Forge Goals

Updated: 2026-01-08

Priority: 1

## Current Milestone
Type System Hardening - Make the solver correct and robust.

## Focus Areas (Director can reassign)
- `wasm/src/solver/` - Type inference, constraint solving
- `wasm/src/checker/` - Type checking logic
- `wasm/src/binder/` - Symbol binding, scope analysis
- `wasm/src/types/` - Type representations

## Objectives (Ranked)

1. **Generic Inference Hardening**
   - Context: Solver is the correctness bottleneck per Project Direction
   - Success Criteria: All `infer_tests.rs` pass; no panics on redux/lodash types
   - Key Files: `solver/infer.rs`, `solver/infer_tests.rs`
   - Estimated Complexity: High

2. **Conditional Type Evaluation**
   - Context: Distributive conditional types over unions must match TypeScript
   - Success Criteria: All `evaluate_tests.rs` pass; template literal inference works
   - Key Files: `solver/evaluate.rs`, `solver/evaluate_tests.rs`
   - Estimated Complexity: High

3. **Structural Compatibility (Variance)**
   - Context: Variance handling must be correct for all edge cases
   - Success Criteria: All `subtype_tests.rs` pass
   - Key Files: `solver/subtype.rs`, `solver/subtype_tests.rs`
   - Estimated Complexity: Medium

## Anti-Priorities
- New LSP features (unless they expose a Solver bug)
- CLI argument parsing
- Performance micro-optimizations

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
