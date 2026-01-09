# Squad Forge Goals

Updated: 2026-01-09

Priority: 1

---
## 📢 EM-FORGE: DIRECTIVE UPDATE

**Director is handling Redux blocker personally.** Forge workers reassigned to:

### Failing Tests to Fix (Priority Order):
1. `solver::evaluate::tests::test_conditional_infer_function_optional_param_distributive`
2. `solver::evaluate::tests::test_conditional_infer_function_optional_param_non_distributive_union_input`
3. `cli::driver_tests::compile_class_with_generic_constructor`

### Other High-Value Solver Work:
- Template literal type inference edge cases
- Generic constraint satisfaction edge cases
- Mapped type key remapping

---

## Forge Focus: Solver Test Fixes + Hardening

**Redux blocker is being handled by Director. Focus on other failing tests.**

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

1. **Fix Failing Solver Tests**
   - `test_conditional_infer_function_optional_param_distributive` - FAILING
   - `test_conditional_infer_function_optional_param_non_distributive_union_input` - FAILING
   - Key Files: `solver/evaluate.rs`, `solver/evaluate_tests.rs`

2. **Fix CLI Generic Constructor Test**
   - `compile_class_with_generic_constructor` - FAILING
   - Key Files: `cli/driver_tests.rs`, `solver/infer.rs`

3. **Generic Inference Hardening**
   - Context: Solver is the correctness bottleneck per Project Direction
   - Success Criteria: Inference from usage and context-sensitive typing match `tsc`
   - Key Files: `solver/infer.rs`, `solver/infer_tests.rs`

4. **Template Literal Types**
   - Context: Template literal inference and pattern matching
   - Key Files: `solver/evaluate.rs`

## Anti-Priorities
- New LSP features
- CLI argument parsing or UX changes
- Performance micro-optimizations
- New AST nodes

## Cross-Squad Dependencies
- Anvil workers 3-5 are on **Crucible Tasks** porting tests from official TS repo
- These tests will help triangulate correct behavior for Forge workers

## Notes to EM
- **Director is handling Redux blocker** - do NOT assign workers to it
- Focus on fixing the 3 failing tests listed above
- Read `wasm/specs/SOLVER.md` for solver architecture
- Use Docker for tests: `./wasm/test.sh`

## Management Strategy
Per Project Direction: **Autocratic Scheduling + Bisect-on-Merge**
- PRs that regress ANY existing baseline are auto-rejected

## Squad Status
- Last EM Report: 2026-01-09 - Reassigned from Redux blocker
- Workers Active: 5/5
- Current Focus: Fix 3 failing solver/CLI tests
- Direction: Test fixes + solver hardening
- Worker Assignments:
  - W1: Fix `test_conditional_infer_function_optional_param_distributive`
  - W2: Fix `test_conditional_infer_function_optional_param_non_distributive_union_input`
  - W3: Fix `compile_class_with_generic_constructor`
  - W4: Template literal type edge cases
  - W5: Generic constraint satisfaction edge cases
