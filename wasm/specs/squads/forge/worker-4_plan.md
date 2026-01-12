# Worker 4 Plan - Squad Forge

## Mission
Fix TS2339: Property Does Not Exist

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Fix TS2339 False Positives: Property does not exist (35 extra errors)**

### Background
W1 implemented a TS2339 fix that works in cargo but fails in WASM. The fix addresses atom comparison issues in the solver's property access resolution. Need to investigate and fix the WASM-specific behavior.

### Success Criteria
- Debug why the TS2339 fix works in cargo but not in WASM
- Fix the discrepancy between cargo run and WASM package execution
- Ensure property access works correctly for private static members
- Test with various property access patterns
- Run conformance to verify error reduction

### Implementation Steps
1. [ ] Sync from origin/rust: `git fetch origin && git merge origin/rust --no-edit`
2. [ ] Review W1's implementation in `src/thin_checker.rs`
3. [ ] Investigate WASM-specific behavior differences
4. [ ] Check for atom interning issues in WASM context
5. [ ] Fix the discrepancy to make it work in both cargo and WASM
6. [ ] Test with various property access patterns
7. [ ] Run conformance to verify error reduction

### Key Code Locations
- `src/thin_checker.rs` - W1's TS2339 fix implementation
- `src/solver/operations.rs` - property access resolution
- `wasm/src/thin_checker.rs` - WASM-specific code

### Test Cases
```typescript
// TS2339 should NOT emit for valid property accesses
class Foo {
  static #field: number;
  static getField() { return Foo.#field; } // OK - private static member
}

// Should still emit TS2339 for invalid properties
const obj = { x: 1 };
console.log(obj.y); // Error: Property 'y' does not exist
```

## Completed
- [x] TS7010 - Implicit any return type (merged to squad/forge)
- [x] TS7006 - Parameter 'any' type (destructuring and setter fixes, merged)
- [x] TS2322 - Constructor return statement fix (merged to squad/forge)

## Notes
- W1's commit: afc120f6e8 - "Implement TS2339 fix for private static members"
- Investigate WASM vs cargo discrepancy
- Commit format: `[wasm] checker: Fix TS2339 WASM discrepancy in property access`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
