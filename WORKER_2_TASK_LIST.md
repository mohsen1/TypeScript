# Worker 2 Task List - CFA Squad

## Current Task
- [ ] **CFA-2: Build FlowGraph from AST**
  - Create `FlowGraphBuilder` in `src/checker/flow_graph_builder.rs`
  - Traverse ThinNode AST post-binding (don't mutate nodes)
  - Build graph for: if/else, switch, for/while/do-while, try/catch, blocks
  - Track variable declarations and assignments
  - Return `FlowGraph` side-table

## Queue
- [ ] **CFA-5: Add reachability analysis**
  - Implement `ReachabilityAnalyzer` for code paths
  - Detect unreachable code after return/throw/break/continue
  - Mark unreachable statements in FlowGraph
  - Test with cases: `return; x = 1;` (x=1 is unreachable)
- [ ] **CFA-8: Check property initialization (TS2564)**
  - Add `check_property_init` in Checker
  - Query FlowGraph for class property state at constructor exit
  - Emit TS2564 error for non-optional properties not definitely assigned
  - Test with: `class A { x: number; }` and `class B { x!: number; }`

## Completed
(none yet)
