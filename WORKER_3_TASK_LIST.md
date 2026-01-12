# Worker 3 Task List - CFA Squad

## Current Task
- [ ] **CFA-6: Handle try/catch/finally flow**
  - Verify try/catch/finally flow edges in FlowGraphBuilder are correct
  - Track variable state across catch blocks
  - Ensure finally blocks affect all exit paths
  - Test: `let x; try { x = 1; } finally { } console.log(x);`

## Queue
- [ ] **CFA-9: Write conformance tests for CFA**
  - Create test file: `tests/conformance/cfa_tests.ts`
  - Add 50+ cases covering all CFA error codes (TS2454, TS2564)
  - Verify error counts match tsc output
  - Goal: Reduce missing TS2454/TS2564 errors by 90%

## Completed
- [x] **CFA-3: Integrate FlowGraph into Checker**
  - Added `flow_graph: Option<FlowGraph>` field to Checker
  - Called `FlowGraphBuilder` after binding phase
  - Added `check_flow_usage` method to query the graph
  - Stored FlowGraph reference in TypeCheck context
