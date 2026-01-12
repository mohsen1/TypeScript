# Worker 2 Task List - CFA Squad

## Current Task
- [ ] **CFA-5: Add reachability analysis**
  - Implement `ReachabilityAnalyzer` for code paths
  - Detect unreachable code after return/throw/break/continue
  - Mark unreachable statements in FlowGraph
  - Test with cases: `return; x = 1;` (x=1 is unreachable)

## Queue
- [ ] **CFA-8: Check property initialization (TS2564)**
  - Add `check_property_init` in Checker
  - Query FlowGraph for class property state at constructor exit
  - Emit TS2564 error for non-optional properties not definitely assigned
  - Test with: `class A { x: number; }` and `class B { x!: number; }`
- [ ] **CFA-9: Add unit tests for FlowGraphBuilder**
  - Test all control flow structures (if/else, loops, try/catch)
  - Test variable tracking across branches
  - Test edge cases (empty blocks, nested structures)

## Completed
- [x] **CFA-2: Build FlowGraph from AST**
  - Created `FlowGraphBuilder` in `wasm/src/checker/flow_graph_builder.rs`
  - Implemented AST traversal post-binding
  - Built graph for: if/else, switch, for/while/do-while, try/catch, blocks
  - Tracked variable declarations and assignments
  - Returned `FlowGraph` side-table
