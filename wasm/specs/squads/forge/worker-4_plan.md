# Worker 4 Plan - Squad Forge

## Mission
Fix TS2454: Variable Used Before Assignment

Status: Active
Priority: P0 (CRITICAL)

## Current Assignment
**Implement TS2454 Error: Variable used before assignment**

### Background
TypeScript should emit TS2454 error when a variable is used before it's definitely assigned. The definite assignment analysis needs to identify these cases and report errors.

### Success Criteria
- Emit TS2454 for variables used before definite assignment
- Handle all variable declaration contexts (let, const, var)
- Account for control flow branches and early returns
- Don't emit false positives (variables that are definitely assigned)

### Implementation Steps
1. [ ] Read existing definite assignment code in `src/thin_checker.rs`
2. [ ] Find where variables are checked for usage before assignment
3. [ ] Implement check: if variable used before any assignment, emit TS2454
4. [ ] Test with cases that should emit TS2454
5. [ ] Ensure no false positives for definitely-assigned variables

### Key Code Locations
- `src/thin_checker.rs` - definite assignment analysis, `should_check_definite_assignment`
- `src/checker/control_flow.rs` - flow analysis for definite assignment

### Test Cases to Implement
```typescript
// Should emit TS2454
let x;
console.log(x); // Error: 'x' used before assignment

// Should NOT emit (assigned in all paths)
let y;
if (condition) {
  y = 1;
} else {
  y = 2;
}
console.log(y); // OK

// Should emit TS2454 (not all paths assign)
let z;
if (condition) {
  z = 1;
}
console.log(z); // Error: 'z' might not be assigned
```

## Completed
- [x] Fix Element Access Literal Keys - Merged to squad/forge

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Implement TS2454 definite assignment errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
