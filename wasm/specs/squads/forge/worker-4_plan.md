# Worker 4 Plan - Squad Forge

## Mission
Fix TS2322: Type Not Assignable

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Fix TS2322 Missing Errors: Type not assignable (19 occurrences - some missing)**

### Background
TypeScript is missing TS2322 "Type not assignable" errors in some cases. This is a type compatibility issue where the assignability checker isn't catching all invalid type assignments.

### Success Criteria
- Emit TS2322 for all invalid type assignments
- Handle type compatibility checking for:
  - Interface implementations
  - Class inheritance
  - Generic type constraints
  - Union/intersection types
- Don't emit false positives for valid assignments
- Fix missing TS2322 detections (19 occurrences)

### Implementation Steps
1. [ ] Sync from origin/rust: `git fetch origin && git merge origin/rust --no-edit`
2. [ ] Search for TS2322 emission code in `src/solver/operations.rs` or `src/thin_checker.rs`
3. [ ] Investigate assignability checking logic
4. [ ] Find cases where TS2322 should be emitted but isn't
5. [ ] Fix type compatibility checking
6. [ ] Test with various assignability patterns
7. [ ] Run conformance to verify TS2322 is emitted correctly

### Key Code Locations
- `src/solver/operations.rs` - type assignability checking
- `src/thin_checker.rs` - assignment expression checking
- `src/checker/types/diagnostics.rs` - TS2322 error code

### Test Cases
```typescript
// Should emit TS2322
let x: string = 42; // Error: Type 'number' is not assignable to type 'string'

interface Foo {
  x: number;
}

class Bar implements Foo {
  x: string; // Error: Type 'string' is not assignable to type 'number'
}

function baz<T extends number>(arg: T): T {
  return "hello" as T; // Error: Type 'string' is not assignable to type 'T'
}

// Should NOT emit
let y: number = 42; // OK

class Qux implements Foo {
  x: number; // OK
}
```

## Completed
- [x] TS7010 Implicit Any Return - Merged to squad/forge
- [x] TS7006 Parameter 'Any' Type - Basic implementation complete, committed

## Ready for Merge
No

## Notes
- Commit format: `[wasm] checker: Fix TS2322 missing type assignability errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
