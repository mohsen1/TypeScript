# Squad Forge Goals

Updated: 2026-01-09

Priority: 1

---
## 🛑 GRACEFUL EXIT - WRAP UP NOW

**Director has called for graceful exit. Wrap up all worker tasks NOW.**

1. Let workers finish their current atomic task (commit what's done)
2. Do NOT assign new tasks
3. Ensure all work is committed and pushed to worker branches
4. Mark workers as complete when done
5. Final merge to squad branch

**Session is ending. No new work assignments.**

---
## ⚠️ TMUX REMINDER - CHECK FOR HANGING PROMPTS

**NEVER forget to pause 1 second before pressing Enter in tmux!**

1. After sending any message, wait 1 second, THEN send Enter (C-m)
2. Check all worker panes for prompts that may be hanging (message sent but no activity)
3. If a prompt is hanging, send Enter again: `sleep 1 && tmux send-keys -t <pane> C-m`

**Do this check NOW and periodically throughout your session.**

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

3. **Fix Failing Tests** - ALL FIXED ✅
   - `test_conditional_infer_function_optional_param_distributive` - ✅ FIXED
   - `test_conditional_infer_function_optional_param_non_distributive_union_input` - ✅ FIXED
   - `compile_class_with_generic_constructor` - ✅ FIXED

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
- Last EM Report: 2026-01-09 - EM session active
- Redux Baseline: **2 diagnostics** (improved from 4, target: 0)
- Workers Active: 5/5
- Current Focus: Generic Inference + Conditional Types
- Direction: Solver hardening per directive
- Session Merges:
  - forge-3: try_expand_type_arg fix
  - forge-3: generic class type expansion fix
  - forge-3: InferSubstitutor Function type support (Redux 4→2!)
- Blocked Commits:
  - forge-2: Removes Ref/TypeQuery handling (regression)
  - forge-5: Large diff with merge conflicts
- Worker Assignments:
  - W1/Pane3: Template literal type inference
  - W2/Pane4: ReturnType/Parameters edge cases
  - W3/Pane5: Function parameter inference tests
  - W4/Pane6: Circular constraints in extends clauses
  - W5/Pane7: Distributive conditional stress tests
