# Worker 3 Task List - CFA Squad

## Current Task
- [ ] **CFA-9: Write conformance tests for CFA**
  - Create test file: `tests/conformance/cfa_tests.ts`
  - Add 50+ cases covering all CFA error codes (TS2454, TS2564)
  - Verify error counts match tsc output
  - Goal: Reduce missing TS2454/TS2564 errors by 90%

## Queue
- [ ] **CFA-13: Test complex control flow scenarios**
  - Add tests for nested try/catch/finally blocks
  - Test loops with break/continue and definite assignment
  - Verify switch statement fallthrough tracking
  - Test conditional assignments with all branch paths
- [ ] **CFA-14: Coordinate CFA integration with Worker 1**
  - Work with Worker 1 to merge FlowGraph integration
  - Ensure Checker integration is complete
  - Verify all CFA components work together

## Completed
- [x] **CFA-6: Handle try/catch/finally flow**
  - Verified try/catch/finally flow edges in FlowGraphBuilder
  - Track variable state across catch blocks
  - Ensured finally blocks affect all exit paths
  - Tested `let x; try { x = 1; } finally { } console.log(x);`
- [x] **CFA-3: Integrate FlowGraph into Checker**
  - Added `flow_graph: Option<FlowGraph>` field to Checker
  - Called `FlowGraphBuilder` after binding phase
  - Added `check_flow_usage` method to query the graph
  - Stored FlowGraph reference in TypeCheck context
- [x] **CFA-6: Handle try/catch/finally flow**
  - Verified try/catch/finally flow edges in FlowGraphBuilder are correct
  - Tracked variable state across catch blocks
  - Ensured finally blocks affect all exit paths
  - Tested: `let x; try { x = 1; } finally { } console.log(x);`
  - Coordinated with Worker 2 on FlowGraphBuilder implementation
