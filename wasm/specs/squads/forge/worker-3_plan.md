# Worker 3 Plan - Squad Forge

## Mission
Fix TS2705: Async Function Must Return Promise

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Implement TS2705 Error: Async function must return Promise (37 occurrences)**

### Background
TypeScript should emit TS2705 when an async function is declared with a return type that is not a Promise. Async functions always return Promises, so the return type must reflect that.

### Success Criteria
- Emit TS2705 when async function has non-Promise return type annotation
- Handle all async function types: function declarations, expressions, arrow functions, methods
- Don't emit when return type is Promise or generic Promise<T>
- Don't emit when return type is omitted (inferred as Promise)
- Handle async arrow functions correctly

### Implementation Steps
1. [x] Sync from origin/rust: `git fetch origin && git merge origin/rust --no-edit`
2. [x] Search for async function type checking in `src/thin_checker.rs`
3. [x] Find where function return types are validated
4. [x] Implement TS2705 emission: if function is async and return type is not Promise, emit error
5. [x] Test with various async function patterns
6. [x] Run conformance to verify TS2705 is emitted correctly

### Key Code Locations
- `src/thin_checker.rs` - async function type checking
- `src/checker/types/diagnostics.rs` - TS2705 error code
- `src/checker/types.rs` - Promise type definition

### Test Cases
```typescript
// Should emit TS2705
async function foo(): number { } // Error: Async function must return Promise
async function bar(): string { return "x"; } // Error: return type is string, not Promise<string>

const baz = async (): boolean => false; // Error: Async arrow function must return Promise

class Qux {
  async method(): void { } // Error: Async method must return Promise
}

// Should NOT emit
async function qux(): Promise<number> { } // OK - explicit Promise
async function quux() { } // OK - inferred as Promise<any>
async function corge(): Promise<void> { } // OK - Promise<void>
```

## Completed
- [x] TS2304 Cannot Find Name - Fixed infer type parameter false positives, committed
- [x] TS2705 Async Function Must Return Promise - Implemented TS2705 emission for async functions with non-Promise return types
  - Added ASYNC_FUNCTION_RETURNS_PROMISE error code (2705)
  - Added is_promise_type() helper function
  - Handles function declarations, arrow functions, function expressions, and methods
  - Test added: test_async_function_returns_promise

## Notes
- Commit format: `[wasm] checker: Implement TS2705 async function return type errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-3`
