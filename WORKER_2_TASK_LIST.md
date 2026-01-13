# Worker 2 Task List - CFA Squad

## Current Task
- [ ] **CFA-15: Test async/await flow analysis**
  - Add FlowGraphBuilder support for async functions
  - Test variable state across await boundaries
  - Ensure promise rejection paths are tracked
  - Verify definite assignment with async control flow

## Queue
- [ ] **CFA-16: Add generator function flow tracking**
  - Implement yield expression handling in FlowGraph
  - Track variable state across yield points
  - Test generator function definite assignment
- [ ] **CFA-17: Test callback closure flow tracking**
  - Analyze variable capture in closure functions
  - Test definite assignment across callback boundaries
  - Verify flow analysis for arrow functions and callbacks

## Completed
- [x] **CFA-9: Add unit tests for FlowGraphBuilder**
  - Reviewed existing test coverage for FlowGraphBuilder
  - All control flow structures already have basic tests
  - Tests cover if/else, loops, try/catch, switch, blocks
- [x] **CFA-8: Check property initialization (TS2564)**
  - Added `check_property_init` in Checker
  - Query FlowGraph for class property state at constructor exit
  - Emit TS2564 error for non-optional properties not definitely assigned
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
