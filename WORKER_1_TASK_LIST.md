# Worker 1 Task List - CFA Squad

## Current Task
- [ ] **CFA-1: Design Flow Graph data structure**
  - Create `FlowGraph` struct in `src/checker/flow_graph.rs`
  - Design nodes for: BlockEntry, BlockExit, Assignment, Condition, Branch
  - Implement `FlowEdge` struct with condition flags
  - Add `FlowGraphBuilder` trait
  - Write unit tests for basic graph construction

## Queue
- [ ] **CFA-4: Implement definite assignment analysis algorithm**
  - Add `DefiniteAssignmentAnalyzer` struct
  - Implement forward dataflow analysis
  - Track variable states: DefinitelyAssigned, MaybeAssigned, Unassigned
  - Add union/merge operations for join points
- [ ] **CFA-7: Check for use-before-definite-assignment (TS2454)**
  - Add `check_variable_usage` in Checker
  - Query FlowGraph for variable state at usage point
  - Emit TS2454 error for variables used before definite assignment
  - Test with cases: `let x; console.log(x);` and `let x; x = 1; console.log(x);`

## Completed
(none yet)
