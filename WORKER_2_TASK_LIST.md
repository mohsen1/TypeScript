# Worker 2 Task List - CFA Squad

## Current Task
- [ ] **CFA-20: Add support for async generators**
  - Handle async generator functions with yield/await
  - Track variable state through async generator control flow
  - Test complex async generator scenarios
- [ ] **CFA-26: Fix destructuring pattern definite assignment**
  - Analyze nested destructuring: `const { a: { b } } = obj;`
  - Handle array destructuring with defaults: `const [x = 1] = arr;`
  - Track assignment through complex object patterns
  - Test with flow analysis: `let x; if (cond) { ({ x } = obj); } console.log(x);`
- [ ] **CFA-27: Implement catch clause variable binding analysis**
  - Catch clause variables should be definitely assigned in catch block
  - Verify `try {} catch (e) { console.log(e); }` works correctly
  - Handle nested try/catch blocks with same variable names
- [ ] **CFA-28: Add temporal dead zone (TDZ) detection**
  - Detect usage of let/const before declaration (TDZ)
  - Emit TS2448 error for TDZ violations
  - Test: `console.log(x); let x;` should error
  - Verify TDZ works with block scoping and loops

## Queue
(none - await new task assignment)

## Completed
- [x] **CFA-19: Test callback closure flow tracking**
  - Added 12 comprehensive tests for closure/callback flow tracking
  - Tested variable capture in arrow functions and callbacks
  - Tested definite assignment across callback boundaries
  - Tested nested closure scenarios
  - Verified flow analysis for array methods (forEach, map, filter)
  - Verified flow analysis for setTimeout callbacks
  - Tested multiple closures capturing same variable at different points
  - Tested closure with conditional capture in if branches
- [x] **CFA-16: Add generator function flow tracking**
  - Implemented yield expression handling in FlowGraph
  - Tracked variable state across yield points
  - Tested generator function definite assignment
- [x] **CFA-15: Test async/await flow analysis**
  - Added FlowGraphBuilder support for async functions
  - Added async_depth tracking in FlowGraphBuilder
  - Implemented handle_await_expression() for creating AWAIT_POINT flow nodes
  - Implemented handle_expression_for_await() for recursive await detection
  - Added 6 comprehensive async/await flow tests
  - Tested variable state across await boundaries
  - Ensured promise rejection paths are tracked
  - Verified definite assignment with async control flow
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
