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
- Last EM Report: 2026-01-09 - Operation Crucible in progress
- Workers Active: 1/5 (pane 4 actively debugging type param substitution)
- Current Focus: 🚨 Redux/Lodash Generics blocker - type parameter registration
- Direction: SWARM THE BLOCKER
- Blockers: `test_check_redux_lodash_style_generics` (4 diagnostics vs 0)
- Progress: 6 → 4 diagnostics (33% reduction)
- Issues:
  - Worker 2's changes cause REGRESSION (removes Ref handling in evaluate.rs)
  - Type params not being registered in TypeEnvironment for cross-file symbols
  - Symbols resolving to TypeId(4) (any) instead of actual types
- Worker Assignments:
  - W1: (idle)
  - W2: BLOCKED - must restore Ref/TypeQuery handling in evaluate.rs
  - W3: (merged docs update)
  - W4: (merged namespace fix) - now investigating type param registration
  - W5: (idle)
