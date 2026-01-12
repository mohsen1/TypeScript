# Worker 1 Task List - CFA Squad (Lead)

## Current Task
- [ ] **CFA-4: Implement definite assignment analysis algorithm**
  - Add `DefiniteAssignmentAnalyzer` struct to `src/checker/flow_analyzer.rs`
  - Implement forward dataflow analysis over FlowGraph
  - Track variable states: DefinitelyAssigned, MaybeAssigned, Unassigned
  - Add union/merge operations for join points
  - Write unit tests for analysis algorithm

## Queue
- [ ] **CFA-7: Check for use-before-definite-assignment (TS2454)**
  - Add `check_variable_usage` in Checker
  - Query FlowGraph for variable state at usage point
  - Emit TS2454 error for variables used before definite assignment
  - Test with cases: `let x; console.log(x);` and `let x; x = 1; console.log(x);`
- [ ] **CFA-11: Merge and coordinate CFA Squad work**
  - Review Workers 2-3 implementations for consistency
  - Merge completed FlowGraph components into main branch
  - Ensure FlowGraph API is stable across all CFA components
  - Coordinate integration testing with Worker 3
- [ ] **CFA-12: Verify TS2454/TS2564 reduction goal**
  - Run conformance tests targeting 573 TS2454 and 443 TS2564 missing errors
  - Measure improvement toward 90% reduction goal
  - Document remaining gaps

## Completed
- [x] **CFA-1: Design Flow Graph data structure**
  - Created `FlowGraph` struct in `wasm/src/checker/flow_graph.rs`
  - Designed nodes for: BlockEntry, BlockExit, Assignment, Condition, Branch
  - Implemented `FlowEdge` struct with condition flags
  - Added `FlowGraphBuilder` trait
  - Wrote unit tests for basic graph construction
