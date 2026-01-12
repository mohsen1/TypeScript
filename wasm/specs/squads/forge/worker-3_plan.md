# Worker 3 Plan - Squad Forge

## Mission
Fix TS2339: Property Does Not Exist

Status: Active
Priority: P0 (CRITICAL)

## Current Assignment
**Implement TS2339 Error: Property does not exist on type**

### Background
TypeScript should emit TS2339 error when accessing a property that doesn't exist on a type. This includes:
1. Property access on objects
2. Method access on objects
3. Index signatures
4. Optional chaining (?.)
5. Nested property access

### Success Criteria
- Emit TS2339 for non-existent properties
- Don't emit for existing properties
- Handle optional chaining correctly
- Handle index signatures
- Suggest typos if similar property exists

### Implementation Steps
1. [ ] Find property access checking in `src/thin_checker.rs`
2. [ ] Implement check: if property not found on type, emit TS2339
3. [ ] Test with various property access patterns
4. [ ] Ensure no false positives for valid properties

### Key Code Locations
- `src/thin_checker.rs` - property access checking
- `src/solver/operations.rs` - property lookup operations
- `src/checker/types/diagnostics.rs` - TS2339 error code

### Test Cases to Implement
```typescript
// Should emit TS2339
const obj = { x: 1 };
console.log(obj.y); // Error: Property 'y' does not exist on type '{ x: number }'

// Should NOT emit (property exists)
console.log(obj.x); // OK

// Should handle optional chaining
console.log(obj?.z); // Error: Property 'z' does not exist
```

## Completed
- [x] Namespace merging enum/function work (committed)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Implement TS2339 property access errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-3`
