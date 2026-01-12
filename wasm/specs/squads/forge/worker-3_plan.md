# Worker 3 Plan - Squad Forge

## Mission
Fix TS2322: Type Not Assignable

Status: Active
Priority: P0 (CRITICAL)

## Current Assignment
**Implement TS2322 Error: Type is not assignable to other type**

### Background
TypeScript should emit TS2322 error when a value of one type is assigned to a variable/parameter of a different, incompatible type. The type checker needs to verify assignability using subtyping rules.

### Success Criteria
- Emit TS2322 for type mismatches in assignments
- Handle all assignment contexts: variable declarations, parameter passing, return statements
- Account for type compatibility rules (subtype, supertype, unrelated)
- Don't emit false positives for compatible types
- Handle contextual typing (inferred from usage)

### Implementation Steps
1. [ ] Read existing assignability checking code in `src/solver/subtype.rs` and `src/thin_checker.rs`
2. [ ] Find where type assignability is checked
3. [ ] Implement or fix check: if source type not assignable to target type, emit TS2322
4. [ ] Test with various assignment patterns
5. [ ] Ensure no false positives for compatible types

### Key Code Locations
- `src/solver/subtype.rs` - subtype checking and assignability logic
- `src/thin_checker.rs` - assignment expression checking
- `src/checker/types/diagnostics.rs` - TS2322 error code

### Test Cases to Implement
```typescript
// Should emit TS2322
let x: number = "string"; // Error: Type 'string' is not assignable to type 'number'
function foo(y: string) { }
foo(42); // Error: Type 'number' is not assignable to parameter of type 'string'

// Should NOT emit (compatible types)
let a: number = 42; // OK
let b: number = a; // OK
let c: string | number = "hello"; // OK
```

## Completed
- [x] Namespace merging enum/function work (committed)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] solver/checker: Implement TS2322 type assignability errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-3`
