# Worker 1 Plan - Squad Forge

## Mission
Fix TS2339: Property Does Not Exist

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Fix TS2339 False Positives: Property does not exist (35 extra errors)**

### Background
TypeScript is emitting TS2339 "Property does not exist" errors too aggressively (35 extra errors). This is likely a type narrowing issue where valid property accesses are incorrectly flagged.

### Success Criteria
- Don't emit TS2339 for valid property accesses on narrowed types
- Only emit TS2339 when property truly doesn't exist on type
- Fix type narrowing in property access expressions

### Implementation Steps
1. [ ] Sync from origin/rust: `git fetch origin && git merge origin/rust --no-edit`
2. [ ] Search for TS2339 emission code in `src/thin_checker.rs`
3. [ ] Investigate type narrowing logic for property access
4. [ ] Fix cases where valid properties are incorrectly flagged
5. [ ] Test with various property access patterns
6. [ ] Run conformance to verify reduction in extra errors

### Key Code Locations
- `src/thin_checker.rs` - property access checking, TS2339 emission
- `src/checker/control_flow.rs` - type narrowing
- `src/checker/types/diagnostics.rs` - TS2339 error code

### Test Cases
```typescript
// Should NOT emit TS2339 (type narrowing)
interface Foo { x: number; }
interface Bar { y: string; }

function foo(obj: Foo | Bar) {
  if ('x' in obj) {
    console.log(obj.x); // OK - narrowed to Foo
  }
}

// Should emit TS2339
const obj = { x: 1 };
console.log(obj.y); // Error: Property 'y' does not exist
```

## Completed
- [x] Fix Method Bivariance - Merged to squad/forge
- [x] TS7010/TS7011 - Implemented and pushed to origin/worker/forge-1

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Fix TS2339 false positives in property access`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-1`
