# Worker 1 Plan - Squad Forge

## Mission
Fix TS2300: Duplicate Identifier

Status: Active
Priority: P0 (CRITICAL)

## Current Assignment
**Implement TS2300 Error: Duplicate identifier**

### Background
TypeScript should emit TS2300 error when the same identifier is declared multiple times in the same scope. This includes:
1. Duplicate variable declarations (let/const/var)
2. Duplicate function declarations
3. Duplicate parameter names
4. Duplicate class members
5. Duplicate enum members
6. Duplicate type aliases/interfaces

### Success Criteria
- Emit TS2300 for duplicate declarations in the same scope
- Don't emit for declarations in different scopes (shadowing)
- Handle all declaration types
- Report the location of both declarations

### Implementation Steps
1. [ ] Find symbol declaration tracking in binder
2. [ ] Implement duplicate detection when declaring symbols
3. [ ] Emit TS2300 when duplicate found in same scope
4. [ ] Test with various duplicate patterns
5. [ ] Ensure no false positives for valid shadowing

### Key Code Locations
- `src/binder.rs` - symbol declaration, scope management
- `src/thin_binder.rs` - symbol table
- `src/checker/types/diagnostics.rs` - TS2300 error code

### Test Cases to Implement
```typescript
// Should emit TS2300
let x = 1;
let x = 2; // Error: Duplicate identifier 'x'

function foo() { }
function foo() { } // Error: Duplicate identifier 'foo'

// Should NOT emit (different scopes)
let y = 1;
{
  let y = 2; // OK - different scope
}
```

## Completed
- [x] Fix Method Bivariance - Merged to squad/forge

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] binder: Implement TS2300 duplicate identifier detection`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-1`
