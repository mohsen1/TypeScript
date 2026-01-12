# Worker 2 Plan - Squad Forge

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
1. [ ] Sync from origin/rust: `git fetch origin && git merge origin/rust --no-edit`
2. [ ] Search for async function type checking in `src/thin_checker.rs`
3. [ ] Find where function return types are validated
4. [ ] Implement TS2705 emission: if function is async and return type is not Promise, emit error
5. [ ] Test with various async function patterns
6. [ ] Run conformance to verify TS2705 is emitted correctly

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
- [x] Namespace merging enum/function work
- [x] TS2322 investigation (already implemented)

## Notes
- Commit format: `[wasm] checker: Implement TS2705 async function return type errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
