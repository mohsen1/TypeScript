# Worker 6 Task List

## Current Task
- [ ] Implement scope chain traversal for symbol resolution
  - Ensure Binder walks scope chain correctly: local -> module -> global
  - Add tests for shadowing scenarios
  - Verify block-scoped declarations (let/const) are handled

## Queue
- [ ] Fix import/export symbol visibility
  - Ensure imported symbols are visible in importing module
  - Handle re-exports correctly
  - Test with circular imports
- [ ] Add symbol table validation
  - Run post-binding validation checks
  - Detect orphaned symbols or broken links
  - Ensure all referenced symbols have valid declarations
- [ ] Integrate with Solver to prevent Error Poisoning
  - When TS2304 occurs, don't default to `Any`
  - Propagate error type instead of silencing downstream errors
  - This should reveal missing TS2322/TS7006 errors

## Completed
(none yet)
