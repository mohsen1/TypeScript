# Worker 1 Task List

## Squad: Parser (Syntax) - Reassigned after CFA completion

## Current Task
- [ ] Audit TS1005 ("expected X") emission patterns - find specific parser locations emitting false positives

## Queue
- [ ] Focus on parse_statement.rs and parse_expr.rs error emission points
- [ ] Identify the top 5 patterns causing TS1005 false positives
- [ ] Implement fixes to reduce TS1005 from 439 to <100

## Completed
- [x] Integrate with Checker for flow queries
  - Add `check_flow_usage()` method to Checker
  - Wire FlowGraph into type checking pipeline
- [x] Add basic block identification logic
  - Detect block boundaries (if/else, loops, try/catch)
  - Track variable declarations and assignments
- [x] Design FlowGraph data structure as side-table (separate from AST nodes)
  - Create `wasm/crates/swc_typescript/src/checker/flow_graph.rs`
  - Define `FlowGraph`, `FlowNode`, `FlowEdge` types
  - Ensure no mutation of AST nodes (ThinNode SoA architecture)
  - Add to Checker module exports
- [x] Implement flow graph builder interface
  - Add `build_flow_graph()` function that traverses bound AST
  - Create entry points for function bodies, block statements
