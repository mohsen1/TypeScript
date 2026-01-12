# Worker 2 Task List - CFA Squad

## Current Task
- [ ] **CFA-8: Check property initialization (TS2564)**
  - Add `check_property_init` in Checker
  - Query FlowGraph for class property state at constructor exit
  - Emit TS2564 error for non-optional properties not definitely assigned
  - Test with: `class A { x: number; }` and `class B { x!: number; }`

## Queue
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
- [x] **CFA-5: Add reachability analysis**
  - Created `ReachabilityAnalyzer` in `wasm/src/checker/reachability_analyzer.rs`
  - Added `unreachable_nodes: FxHashSet<u32>` field to FlowGraph
  - Added `is_unreachable()` and `mark_unreachable()` methods to FlowGraph
  - Updated `record_node_flow()` to mark nodes as unreachable when current_flow is unreachable
  - ReachabilityAnalyzer provides API for querying unreachable code from FlowGraph
  - Added 6 comprehensive tests for unreachable code detection (return, throw, break, continue)
