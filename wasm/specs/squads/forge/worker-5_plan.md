# Worker 5 Plan - Squad Forge

## Mission
Fix TS2454: Variable Used Before Assignment

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Implement TS2454 Error: Variable used before assignment (43 occurrences)**

### Background
TypeScript should emit TS2454 error when a variable is used before it's definitely assigned. This requires definite assignment analysis to track which code paths assign values to variables.

### Success Criteria
- Emit TS2454 for variables used before definite assignment
- Handle control flow branches correctly
- Don't emit false positives for variables that are assigned in all paths
- Handle definite assignment assertions (!) correctly

### Implementation Steps
1. [ ] Sync from origin/rust: `git fetch origin && git merge origin/rust --no-edit`
2. [ ] Find definite assignment analysis code in `src/checker/control_flow.rs`
3. [ ] Search for existing TS2454 emission in `src/thin_checker.rs`
4. [ ] Implement or fix TS2454 emission when variable is used before assignment
5. [ ] Test with control flow scenarios (if/else, loops, try/catch)
6. [ ] Run conformance to verify TS2454 is emitted correctly

### Key Code Locations
- `src/checker/control_flow.rs` - definite assignment flow analysis
- `src/thin_checker.rs` - variable usage checking, TS2454 emission
- `src/checker/types/diagnostics.rs` - TS2454 error code

### Test Cases
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

// Should NOT emit (definite assignment assertion)
let z!;
console.log(z); // OK - has definite assignment assertion
```

## Completed
- [x] Fix New Expression Inference - Merged to squad/forge
- [x] Fix TS2322 Type Parameter Resolution - Merged to squad/forge
- [x] TS2564 Property Initialization - Already implemented, documented completion

## Ready for Merge
No

## Notes
- Commit format: `[wasm] checker: Implement TS2454 definite assignment errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-5`
