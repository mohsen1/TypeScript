# Worker 3 Plan - Squad Forge

## Mission
Fix TS2454: Variable Used Before Assignment

Status: Active
Priority: P0 (CRITICAL)

## Current Assignment
**Implement TS2454 Error: Variable used before assignment**

### Background
TypeScript should emit TS2454 error when a variable is used before it's definitely assigned.

### Success Criteria
- Emit TS2454 for variables used before definite assignment
- Handle control flow branches correctly
- Don't emit false positives

### Implementation Steps
1. [ ] Find definite assignment code in `src/thin_checker.rs`
2. [ ] Implement check for variable usage before assignment
3. [ ] Test with control flow scenarios
4. [ ] Ensure no false positives

### Test Cases
```typescript
// Should emit TS2454
let x;
console.log(x); // Error: 'x' used before assignment

// Should NOT emit
let y;
if (c) { y = 1; } else { y = 2; }
console.log(y); // OK
```

## Completed
- [x] Namespace merging enum/function work
- [x] TS2322 investigation (already implemented)

## Notes
- Commit format: `[wasm] checker: Implement TS2454 definite assignment errors`
- Push to: `origin/worker/forge-3`
