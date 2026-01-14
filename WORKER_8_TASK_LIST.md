# Worker 8 Task List

**Maintained by:** EM-2
**Branch:** worker-8 → rust
**Worktree:** /tmp/orchestrator-workspace/worktrees/worker-8

---

## Current Task (PENDING)

### Task 1: Implement TS2454 - Variable Use Before Assignment Detection

**Priority:** HIGH (Priority #1 from README)
**Error Code:** TS2454 ("Variable '{0}' is used before being assigned")
**Impact:** 573 missing errors in conformance tests
**Location:** `wasm/src/checker/control_flow.rs`, `wasm/src/checker/flow_graph_builder.rs`

#### Background
The compiler currently fails to detect when variables are used before being assigned. This is a critical soundness issue. The architecture has Control Flow Analysis (CFA) infrastructure but it's not properly connected to the checker.

#### Requirements
1. Ensure `FlowNodeArena` in `control_flow.rs` properly tracks variable assignments
2. Connect `check_identifier` in `thin_checker.rs` to query the Flow Graph
3. For every identifier usage, check if the variable has been definitely assigned on all paths
4. Default to "Unassigned" (report error) when flow state is uncertain

#### Acceptance Criteria
- [ ] Running `./wasm/differential-test/run-conformance.sh --max=10000` shows significant reduction in TS2454 missing errors
- [ ] Test cases in `tests/cases/compiler/variableDeclarations` pass correctly
- [ ] No regressions in existing conformance (Exact Match doesn't drop below 30%)
- [ ] `cargo test --lib` passes in wasm/ directory

#### Testing Strategy
```bash
# Run conformance focused on TS2454
./wasm/differential-test/run-conformance.sh --max=10000

# Find specific TS2454 issues
node wasm/differential-test/find-ts2454.mjs

# Run unit tests
cd wasm && cargo test control_flow
```

#### Implementation Notes
- The Flow Graph infrastructure exists but may need side-table enhancements
- Look at `wasm/src/checker/flow_analyzer.rs` for reachability analysis patterns
- The `thin_checker.rs` `check_identifier` function is where checks happen
- TypeScript's behavior: report error if ANY path doesn't assign before use

---

## Queue (Future Tasks)

### Task 2: Implement TS2564 - Property Initialization Detection
**Priority:** HIGH (Priority #1 from README)
**Error Code:** TS2564 ("Property '{0}' has no initializer and is not assigned in constructor")
**Impact:** 443 missing errors
**Dependencies:** Task 1 (reuses CFA infrastructure)

### Task 3: Fix TS2322 - Solver Strictness Improvements
**Priority:** MEDIUM (Priority #2 from README)
**Error Code:** TS2322 ("Type '{0}' is not assignable to type '{1}'")
**Impact:** 310 missing errors
**Location:** `wasm/src/solver/`
**Approach:** Change solver fallback from `Any` to `Unknown/Error`

### Task 4: Reduce TS2339 False Positives
**Priority:** MEDIUM (Priority #3 from README)
**Error Code:** TS2339 ("Property '{0}' does not exist on type '{1}'")
**Impact:** 292 extra errors (false positives)
**Approach:** Improve type narrowing, implement apparent members for primitives

---

## Anti-Priorities (DO NOT WORK ON)
- New emitter transforms (ES3, obscure module formats)
- LSP features (semantic tokens, code actions)
- CLI argument parsing
- Performance micro-optimizations

---

## Status Updates
- **Created:** 2026-01-14
- **Last Updated:** 2026-01-14
- **Current Focus:** TS2454 Control Flow Analysis
