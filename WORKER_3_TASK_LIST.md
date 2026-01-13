# Worker 3 Task List

## Current Task
- [ ] Integrate flow analysis with error reporting
  - Emit TS2454 errors at correct source locations
  - Emit TS2564 errors at correct source locations
  - Ensure error messages match tsc output format

## Queue
- [ ] Add conformance tests for CFA
  - Create test cases from missing TS2454/TS2564 errors
  - Verify FlowGraph captures all control flow paths
  - Benchmark performance impact

## Completed
- [x] Implement loop flow analysis
  - Track variable state across loop iterations
  - Handle break/continue statements
  - Detect unreachable code after returns/throws
- [x] Add try/catch/finally flow tracking
  - Model control flow through exception paths
  - Track variable state in catch blocks
  - Handle finally block side effects
