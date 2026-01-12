# Worker 7 Task List

## Current Task
- [ ] **SOLVER-1: Audit solve_subtype fallback behavior**
  - Find all locations where solver falls back to `Any`
  - Identify safe vs. unsafe fallbacks
  - Document in code comments where each fallback occurs
  - Create tracking issue for each fallback point

## Queue
- [ ] **SOLVER-2: Change default fallback from Any to Unknown**
  - Update `solve_subtype` in `src/solver/subtype.rs`
  - Replace `Type::Any` with `Type::Unknown` for safe failures
  - Ensure this forces errors instead of silent acceptance
  - Add unit tests for Unknown fallback behavior

## Completed
(none yet)
