# Squad Forge Goals

Updated: 2026-01-08

Priority: 1

## 🚨 OPERATION CRUCIBLE - SWARM THE BLOCKER

**Redux/Lodash Generics blocker must pass. All 5 workers on this.**

### Root Cause Identified
The bug is **eager evaluation of conditional types when `InferenceVar`s are not yet bound**.

### The Fix
Refactor `solver/evaluate.rs` to introduce a **`Deferred` state** for `ConditionalResult`:
- When `check_type` contains an unbound `InferenceVar`, do NOT return `Any` or `Never`
- Return a `TypeKey::Conditional` that preserves the constraint
- Only evaluate when inference context is finalized

## Current Milestone
Phase 8 - Conformance, Convergence, and Hardening: Type-system correctness in the integrated pipeline, driven by conformance tests.

## Project Direction Alignment
- Solver is the correctness bottleneck (estimated 55% complete)
- 🚨 BLOCKER: `test_check_redux_lodash_style_generics` must pass
- All workers swarm until it's green

## Focus Areas
- `wasm/src/solver/evaluate.rs` - **PRIMARY: Deferred state for conditionals**
- `wasm/src/solver/infer.rs` - Inference context finalization
- `wasm/src/solver/` - Type inference, constraint solving

## Objectives (Ranked)

1. **🚨 BLOCKER: Redux/Lodash Generics**
   - Context: This test combines mapped types, conditional inference, AND cross-file resolution
   - Root Cause: Eager evaluation of conditionals when InferenceVars aren't bound
   - Fix: Introduce `Deferred` state in `solver/evaluate.rs`
   - Key Files: `solver/evaluate.rs`, `solver/infer.rs`
   - **SWARM THIS UNTIL GREEN**

2. **Generic Inference Hardening**
   - Context: Solver is the correctness bottleneck per Project Direction
   - Success Criteria: Inference from usage and context-sensitive typing match `tsc`
   - Key Files: `solver/infer.rs`, `solver/infer_tests.rs`

3. **Conditional Type Evaluation**
   - Context: Distributive conditional types over unions must match TypeScript
   - Success Criteria: `solver/evaluate.rs` handles distributive conditionals correctly
   - Key Files: `solver/evaluate.rs`, `solver/evaluate_tests.rs`

## Anti-Priorities
- New LSP features
- CLI argument parsing or UX changes
- Performance micro-optimizations
- New AST nodes

## Cross-Squad Dependencies
- Anvil workers 3-5 are on **Crucible Tasks** porting tests from official TS repo
- These tests will help triangulate correct behavior for Forge workers

## Notes to EM
- **All 5 workers on Redux blocker**
- Focus on `solver/evaluate.rs` Deferred state implementation
- Read `wasm/specs/SOLVER.md` for solver architecture
- Use Docker for tests: `./wasm/test.sh`

## Management Strategy
Per Project Direction: **Autocratic Scheduling + Bisect-on-Merge**
- PRs that regress ANY existing baseline are auto-rejected
- Zero-Idle: All workers swarm the blocker

## Squad Status
- Last EM Report: 2026-01-08 - Operation Crucible activated
- Workers Active: 5/5
- Current Focus: 🚨 Redux/Lodash Generics blocker - Deferred state in evaluate.rs
- Direction: SWARM THE BLOCKER
- Blockers: `test_check_redux_lodash_style_generics` (6 diagnostics vs 0)
- Root Cause: Eager evaluation of conditionals with unbound InferenceVars
- Worker Assignments:
  - W1: Implement Deferred state in solver/evaluate.rs
  - W2: Update inference context finalization in solver/infer.rs
  - W3: Add TypeKey::Conditional preservation logic
  - W4: Write regression tests for deferred conditional evaluation
  - W5: Investigate cross-file resolution with deferred conditionals
