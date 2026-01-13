# Worker 2 Task List

## Current Task
- [ ] Implement definite assignment analysis for variables (TS2454)
  - Track variable assignments in FlowGraph
  - Detect usage-before-assignment patterns
  - Flag errors at variable usage sites
  - Target: 90% reduction in TS2454 missing errors

## Queue
- [ ] Implement definite assignment analysis for class properties (TS2564)
  - Track property initialization in constructors
  - Detect uninitialized properties
  - Handle property modifiers (optional!, definite assignment assertion)
  - Target: 90% reduction in TS2564 missing errors
- [ ] Add flow-sensitive union narrowing
  - Track type guards in conditional branches
  - Update variable types based on control flow
  - Test with instanceof/typeof checks

## Completed
(none yet)
