# Worker 4 Plan - Squad Forge

## Mission
Fix TS7010: Implicit Any Return Type

Status: Active
Priority: P0 (CRITICAL)

## Current Assignment
**Implement TS7010 Error: Function implicitly has 'any' return type**

### Background
TypeScript should emit TS7010 when a function's return type cannot be inferred and is implicitly 'any'.

### Success Criteria
- Emit TS7010 when return type cannot be inferred
- Don't emit when return type can be inferred from return statements
- Handle void return (no return statements)
- Handle async functions (return Promise wrapper)

### Implementation Steps
1. [ ] Find return type inference code in `src/thin_checker.rs`
2. [ ] Implement check: if return type is any and cannot be inferred, emit TS7010
3. [ ] Test with various function patterns
4. [ ] Ensure no false positives

### Test Cases
```typescript
// Should emit TS7010
function foo() { } // Error: Implicit 'any' return

// Should NOT emit
function bar() { return 42; } // OK - inferred number
function baz() { console.log('x'); } // OK - inferred void
```

## Completed
- [x] Fix Element Access Literal Keys - Merged to squad/forge
- [x] TS7006 - Production ready
- [x] TS2792 - Production ready

## Notes
- Commit format: `[wasm] checker: Implement TS7010 implicit any return type errors`
- Push to: `origin/worker/forge-4`
