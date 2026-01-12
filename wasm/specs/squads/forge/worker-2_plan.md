# Worker 2 Plan - Squad Forge

## Mission
Fix TS2304: Cannot Find Name

Status: Active
Priority: P0 (CRITICAL)

## Current Assignment
**Implement TS2304 Error: Cannot find name**

### Background
TypeScript should emit TS2304 error when an identifier is used but not declared in scope. This includes:
1. Undefined variables
2. Undefined functions
3. Undefined classes/interfaces
4. Undefined types
5. Case sensitivity (foo vs Foo)

### Success Criteria
- Emit TS2304 for undefined identifiers
- Don't emit for global/builtin types (Array, Object, etc.)
- Handle case sensitivity correctly
- Provide helpful error messages

### Implementation Steps
1. [ ] Find identifier resolution in checker
2. [ ] Implement check: if identifier not found in scope, emit TS2304
3. [ ] Test with various undefined patterns
4. [ ] Ensure no false positives for valid references

### Key Code Locations
- `src/thin_checker.rs` - identifier resolution
- `src/binder.rs` - symbol table lookup
- `src/checker/types/diagnostics.rs` - TS2304 error code

### Test Cases to Implement
```typescript
// Should emit TS2304
console.log(undefinedVar); // Error: Cannot find name 'undefinedVar'
let x: NotDefined; // Error: Cannot find name 'NotDefined'

// Should NOT emit (defined)
let y = 42;
console.log(y); // OK
let z: string = "hello"; // OK
```

## Completed
- [x] Namespace merging binder implementation (committed)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Implement TS2304 undefined identifier errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
