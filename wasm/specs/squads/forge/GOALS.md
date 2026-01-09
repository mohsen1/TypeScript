# Squad Forge Goals

Updated: 2026-01-09

Priority: 1

---
## 📢 EM-FORGE: DIRECTIVE UPDATE

**Redux/Lodash blocker is handled by senior staff. DO NOT work on it.**

Focus on these per Project Direction:

### 1. Generic Inference (`solver/infer.rs`)
- Inference from usage
- Context-sensitive typing
- Circular constraints in `extends` clauses

### 2. Conditional Types Stress Testing (`solver/evaluate.rs`)
- Distributive conditional types over unions
- This is where most "toy" compilers fail

### 3. Fix Failing Tests
- `test_conditional_infer_function_optional_param_distributive`
- `test_conditional_infer_function_optional_param_non_distributive_union_input`
- `compile_class_with_generic_constructor`

---

## Forge Focus: Generic Inference + Conditional Types

**Solver is 55% complete. Focus on inference and conditional type evaluation.**

## Current Milestone
Phase 8 - Conformance, Convergence, and Hardening: Type-system correctness in the integrated pipeline, driven by conformance tests.

## Project Direction Alignment
- Solver is the correctness bottleneck (estimated 55% complete)
- Redux blocker handled separately by senior staff
- Focus on Generic Inference and Conditional Types

## Focus Areas
- `wasm/src/solver/infer.rs` - **PRIMARY: Generic inference from usage**
- `wasm/src/solver/evaluate.rs` - Distributive conditional types over unions
- `wasm/src/solver/` - Type inference, constraint solving

## Objectives (Ranked)

1. **Generic Inference Hardening**
   - Inference from usage and context-sensitive typing
   - Circular constraints in `extends` clauses
   - Key Files: `solver/infer.rs`, `solver/infer_tests.rs`

2. **Conditional Types Stress Testing**
   - Distributive conditional types over unions
   - Key Files: `solver/evaluate.rs`, `solver/evaluate_tests.rs`

3. **Fix Failing Tests**
   - `test_conditional_infer_function_optional_param_distributive` - FAILING
   - `test_conditional_infer_function_optional_param_non_distributive_union_input` - FAILING
   - `compile_class_with_generic_constructor` - FAILING

4. **Template Literal Types**
   - Context: Template literal inference and pattern matching
   - Key Files: `solver/evaluate.rs`

## Anti-Priorities
- ⛔ Redux/Lodash blocker (handled by senior staff)
- New LSP features
- CLI argument parsing or UX changes
- Performance micro-optimizations
- New AST nodes

## Cross-Squad Dependencies
- Anvil workers 3-5 are on **Crucible Tasks** porting tests from official TS repo
- These tests will help triangulate correct behavior for Forge workers

## Notes to EM
- **⛔ DO NOT assign workers to Redux blocker** - senior staff handling it
- Focus on Generic Inference and Conditional Types stress testing
- Read `wasm/specs/SOLVER.md` for solver architecture
- Use Docker for tests: `./wasm/test.sh`

## Management Strategy
Per Project Direction: **Autocratic Scheduling + Bisect-on-Merge**
- PRs that regress ANY existing baseline are auto-rejected

## Squad Status
- Last EM Report: 2026-01-09 - Reassigned from Redux blocker
- Workers Active: 5/5
- Current Focus: Generic Inference + Conditional Types
- Direction: Solver hardening (NOT Redux)
- Worker Assignments:
  - W1: Generic inference from usage (`solver/infer.rs`)
  - W2: Context-sensitive typing edge cases
  - W3: Circular constraints in `extends` clauses
  - W4: Distributive conditional types over unions
  - W5: Fix failing conditional infer tests
