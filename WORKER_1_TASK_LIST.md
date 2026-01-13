# Worker 1 Task List

## Current Task
- [ ] Design FlowGraph data structure as side-table (separate from AST nodes)
  - Create `wasm/crates/swc_typescript/src/checker/flow_graph.rs`
  - Define `FlowGraph`, `FlowNode`, `FlowEdge` types
  - Ensure no mutation of AST nodes (ThinNode SoA architecture)
  - Add to Checker module exports

## Queue
- [ ] Implement flow graph builder interface
  - Add `build_flow_graph()` function that traverses bound AST
  - Create entry points for function bodies, block statements
- [ ] Add basic block identification logic
  - Detect block boundaries (if/else, loops, try/catch)
  - Track variable declarations and assignments
- [ ] Integrate with Checker for flow queries
  - Add `check_flow_usage()` method to Checker
  - Wire FlowGraph into type checking pipeline

## Completed
(none yet)
