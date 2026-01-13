# Worker 3 Task List - CFA Squad

## Current Task
- [ ] **CFA-13: Test complex control flow scenarios**
  - Add tests for nested try/catch/finally blocks
  - Test loops with break/continue and definite assignment
  - Verify switch statement fallthrough tracking
  - Test conditional assignments with all branch paths

## Queue
- [ ] **CFA-14: Coordinate CFA integration with Worker 1**
  - Work with Worker 1 to merge FlowGraph integration
  - Ensure Checker integration is complete
  - Verify all CFA components work together
- [ ] **CFA-29: Add loop-induced definite assignment tests**
  - Test `for (let x of arr) { }` definite assignment in loop body
  - Verify `for (const x of arr)` is always definitely assigned
  - Handle `for (;;)` with break statements: `let x; for (;;) { if (cond) { x = 1; break; } } console.log(x);`
  - Test while/do-while loop exit analysis
- [ ] **CFA-30: Implement class property definite assignment in derived classes**
  - Verify derived class properties are initialized
  - Handle super() calls in relation to property initialization
  - Test: `class Derived extends Base { prop: number; }` should error
  - Ensure `this` is used after super() in property initializers
- [ ] **CFA-31: Write final CFA conformance report**
  - Document all TS2454/TS2564 improvements
  - Create before/after comparison with tsc
  - List any remaining edge cases or known limitations
  - Provide recommendations for future enhancements

## Completed
- [x] **CFA-9: Write conformance tests for CFA**
  - Created test file: `tests/conformance/cfa_tests.ts`
  - Added 50+ cases covering all CFA error codes (TS2454, TS2564)
  - Verified error counts match tsc output
  - Goal: Reduce missing TS2454/TS2564 errors by 90%
- [x] **CFA-6: Handle try/catch/finally flow**
  - Verified try/catch/finally flow edges in FlowGraphBuilder
  - Tracked variable state across catch blocks
  - Ensured finally blocks affect all exit paths
  - Tested: `let x; try { x = 1; } finally { } console.log(x);`
  - Coordinated with Worker 2 on FlowGraphBuilder implementation
- [x] **CFA-3: Integrate FlowGraph into Checker**
  - Added `flow_graph: Option<FlowGraph>` field to Checker
  - Called `FlowGraphBuilder` after binding phase
  - Added `check_flow_usage` method to query the graph
  - Stored FlowGraph reference in TypeCheck context
