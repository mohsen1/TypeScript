# Worker 1 Plan - Squad Forge

## Mission
Fix Method Bivariance in the solver - QUICK WIN task

Status: Active
Priority: P0 (Highest)

## Current Assignment
**Fix Method Bivariance (3 failing tests)**

### Background
In TypeScript, method parameters are bivariant (both covariant and contravariant) while function parameters are contravariant (when `strictFunctionTypes` is enabled). The solver currently treats all function types the same.

### Failing Tests
1. `test_method_bivariance_event_handler_pattern`
2. `test_method_bivariance_narrower_argument`
3. `test_method_bivariance_wider_argument`

### Implementation Steps
1. [ ] Read the failing tests in `src/thin_checker_tests.rs` to understand expected behavior
2. [ ] Find `solve_subtype` in `src/solver/logic.rs`
3. [ ] Add check: if comparing method signatures (not standalone functions), skip strict contravariance check
4. [ ] The key is detecting whether we're checking a method vs a standalone function
5. [ ] Run tests: `./wasm/test.sh 2>&1 | grep -E "method_bivariance"`

### Key Code Locations
- `src/solver/logic.rs` - `solve_subtype()` function
- `src/solver/mod.rs` - `SolverConfig` with `strict_function_types` flag
- Reference: TypeScript's `strictFunctionTypes` compiler option

### Expected Fix Pattern
```rust
// In solve_subtype when comparing function parameters:
if is_method_signature && self.config.strict_function_types {
    // Skip contravariance check for methods - use bivariance
    return self.solve_subtype(param_a, param_b) || self.solve_subtype(param_b, param_a);
}
```

## Task Queue
- [ ] After bivariance: help with element access literal keys if time permits

## Completed
- [x] (Move finished items here with brief notes and tests run)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver: Fix method bivariance for strict function types`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-1`
