# Worker 3 Plan - Squad Forge

## Mission
Fix TS2304: Cannot Find Name

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Fix TS2304 Extra Errors: Cannot find name (129 extra errors)**

### Background
TypeScript is emitting TS2304 "Cannot find name" errors too aggressively (129 extra errors). This is likely a scope resolution issue where valid identifiers are incorrectly flagged as undefined.

### Success Criteria
- Don't emit TS2304 for valid identifiers in scope
- Fix scope resolution for all variable types (let/const/var, functions, classes, interfaces)
- Handle global/builtin types correctly (Array, Object, etc.)
- Reduce extra TS2304 errors significantly

### Implementation Steps
1. [ ] Sync from origin/rust: `git fetch origin && git merge origin/rust --no-edit`
2. [ ] Search for TS2304 emission code in `src/thin_checker.rs` or binder
3. [ ] Investigate scope resolution logic
4. [ ] Fix cases where valid identifiers are incorrectly flagged as undefined
5. [ ] Test with various identifier resolution patterns
6. [ ] Run conformance to verify reduction in extra errors

### Key Code Locations
- `src/thin_checker.rs` - identifier resolution, TS2304 emission
- `src/binder.rs` - symbol table lookup, scope management
- `src/checker/types/diagnostics.rs` - TS2304 error code

### Test Cases
```typescript
// Should NOT emit TS2304 (defined)
let x = 42;
console.log(x); // OK

function foo() { return 1; }
console.log(foo()); // OK

class Bar { }
const b = new Bar(); // OK

// Should emit TS2304 (undefined)
console.log(undefinedVar); // Error: Cannot find name 'undefinedVar'
let y: NotDefined; // Error: Cannot find name 'NotDefined'
```

## Completed
- [x] Namespace merging enum/function work
- [x] TS2322 investigation (already implemented)

## Notes
- Commit format: `[wasm] checker: Fix TS2304 false positives in scope resolution`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-3`
