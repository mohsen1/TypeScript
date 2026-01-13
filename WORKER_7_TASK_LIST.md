# Worker 7 Task List

## Current Task
- [ ] Audit all bail-out points in solve_subtype
  - Search for early returns that default to permissive results
  - Replace with conservative assumptions (Unknown, error types)
  - Ensure complex generics don't silently accept invalid code

## Queue
- [ ] Add Unknown type propagation rules
  - Unknown should force explicit type annotations
  - Unknown should trigger errors in unsafe operations
  - Verify Unknown doesn't spread too aggressively

## Completed
- [x] Change Solver default fallback from Any to Unknown
  - Located and changed all fallback logic from Any to Unknown in:
    * src/solver/subtype.rs (this parameter compatibility)
    * src/solver/infer.rs (this parameter compatibility)
    * src/solver/lower.rs (generic type parameter constraints)
    * src/solver/lower.rs (Array/ReadonlyArray element types)
    * src/solver/evaluate.rs (function this types)
  - Added comprehensive tests in src/solver/integration_tests.rs:
    * test_function_this_parameter_fallback_to_unknown
    * test_generic_parameter_without_constraint_fallback_to_unknown
    * test_array_without_type_argument_fallback_to_unknown
    * test_unknown_fallback_prevents_silent_acceptance
    * test_unknown_vs_any_behavior
  - Build succeeded with no errors
