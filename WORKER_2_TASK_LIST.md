# Worker 2 Task List

## Current Task
- [ ] **CFA-3: Build data flow edges for variable assignments and reads**
  - Track variable declarations in `FlowGraph`
  - Create assignment edges when variables are assigned
  - Create read edges when variables are accessed
  - Handle block-scoped variables (let/const)
  - Add flow tracking tests

## Queue
- [ ] **CFA-4: Implement definite assignment analysis algorithm**
  - Add `DefiniteAssignmentChecker` in `src/checker/flow_analysis.rs`
  - Implement forward dataflow analysis (reachability)
  - Mark variables as "definitely assigned" on all paths
  - Return list of variables used before assignment

## Completed
(none yet)
