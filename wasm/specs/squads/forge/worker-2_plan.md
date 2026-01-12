# Worker 2 Plan - Squad Forge

## Mission
Fix TS7010: Implicit Any Return Type

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement TS7010 Error: Function implicitly has 'any' return type**

### Background
TypeScript should emit TS7010 error when a function's return type cannot be inferred and is implicitly 'any'. This happens when:
1. Function has no return type annotation
2. Return type cannot be inferred from return statements
3. The `noImplicitAny` compiler option is enabled

### Success Criteria
- Emit TS7010 when return type cannot be inferred
- Don't emit when return type can be inferred from return statements
- Handle all function types: function declarations, arrow functions, methods
- Account for void return (no return statements)
- Handle async functions (return Promise wrapper)

### Implementation Steps
1. [ ] Read existing return type inference code in `src/thin_checker.rs` and `src/solver/infer.rs`
2. [ ] Find where return types are inferred
3. [ ] Implement check: if return type is any and cannot be inferred, emit TS7010
4. [ ] Test with various function patterns
5. [ ] Ensure no false positives when return type can be inferred

### Key Code Locations
- `src/thin_checker.rs` - function checking, return type validation
- `src/solver/infer.rs` - return type inference
- `src/checker/types/diagnostics.rs` - TS7010 error code

### Test Cases to Implement
```typescript
// Should emit TS7010
function foo() { } // Error: Function implicitly has 'any' return type
const bar = () => { }; // Error: Function implicitly has 'any' return type

// Should NOT emit (return type inferred)
function baz(): number { return 5; } // OK
const qux = (): string => { return "hi"; }; // OK

// Should NOT emit (void return)
function nada() { console.log("void"); } // OK - inferred as void

// Should NOT emit (inferred from return)
function inferred() { return 42; } // OK - inferred as number
```

### Edge Cases
- Async functions should return Promise, not the unwrapped type
- Functions with only throws/return errors should infer void
- Generator functions should return Generator type

## Task Queue
- [ ] After TS7010: coordinate with W3 on type inference issues

## Completed
- [x] Namespace merging binder implementation (committed, tests still failing)
- W4 can continue debugging namespace merging

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Implement TS7010 implicit any return type errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
