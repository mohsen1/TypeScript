# Worker 1 Task List

## Current Task
- [ ] **CFA-1: Design Flow Graph data structure**
  - Create `FlowGraph` struct in `src/checker/flow_graph.rs`
  - Design nodes for BasicBlock, Assignment, Read, Branch, Merge
  - Add `FlowEdge` connections between blocks
  - Implement `FlowGraphBuilder` to traverse `ThinNode` AST
  - Focus on structure first - no checker integration yet

## Queue
- [ ] **CFA-2: Build control flow edges from if/else statements**
  - Parse `IfStatement` nodes and create branching blocks
  - Connect true/false branches to merge point
  - Handle nested control flow
  - Add unit tests for branch construction

## Completed
(none yet)
