# Worker 2 Plan - Squad Forge

## Mission
Fix TS2355: Function Must Return Value

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Fix TS2355 Extra Errors: Function must return value (82 extra errors)**

### Background
TypeScript is emitting TS2355 "Function must return value" errors too aggressively (82 extra errors). This is likely a control flow analysis issue where code paths are incorrectly flagged as not returning a value.

### Success Criteria
- Don't emit TS2355 for functions that return in all code paths
- Fix control flow analysis to recognize all return paths
- Handle early returns, conditional returns, throw statements correctly
- Reduce extra TS2355 errors significantly

### Implementation Steps
1. [ ] Sync from origin/rust: `git fetch origin && git merge origin/rust --no-edit`
2. [ ] Search for TS2355 emission code in `src/thin_checker.rs`
3. [ ] Investigate control flow analysis for return statements
4. [ ] Fix cases where valid returns are incorrectly flagged as missing
5. [ ] Test with various return patterns (early returns, conditionals, throws)
6. [ ] Run conformance to verify reduction in extra errors

### Key Code Locations
- `src/thin_checker.rs` - function return checking, TS2355 emission
- `src/checker/control_flow.rs` - control flow analysis
- `src/checker/types/diagnostics.rs` - TS2355 error code

### Test Cases
```typescript
// Should NOT emit TS2355 (returns in all paths)
function foo(x: number): number {
  if (x > 0) {
    return x;
  }
  return 0; // OK - all paths return
}

function bar(flag: boolean): number {
  if (flag) {
    return 1;
  } else {
    return 2;
  }
} // OK - all conditional paths return

// Should emit TS2355
function baz(): number {
  if (Math.random() > 0.5) {
    return 1;
  }
  // Error: Not all code paths return a value
}
```

## Completed
- [x] TS2564 property no initializer investigation (found existing implementation)

## Notes
- Commit format: `[wasm] checker: Fix TS2355 false positives in return analysis`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
