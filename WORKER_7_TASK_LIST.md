# Worker 7 Task List

## Current Task
- [ ] Change Solver default fallback from Any to Unknown
  - Locate `wasm/crates/swc_typescript/src/solver/subtype.rs`
  - Find all `.fallback_to_any()` or similar logic
  - Replace with `.fallback_to_unknown()`
  - Add tests showing new strictness

## Queue
- [ ] Audit all bail-out points in solve_subtype
  - Search for early returns that default to permissive results
  - Replace with conservative assumptions (Unknown, error types)
  - Ensure complex generics don't silently accept invalid code
- [ ] Add Unknown type propagation rules
  - Unknown should force explicit type annotations
  - Unknown should trigger errors in unsafe operations
  - Verify Unknown doesn't spread too aggressively

## Completed
(none yet)
