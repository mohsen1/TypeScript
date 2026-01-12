# Worker 5 Plan - Squad Forge

## Mission
Fix TS2564: Property Has No Initializer

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement TS2564 Error: Property has no initializer (64 occurrences)**

### Background
TypeScript should emit TS2564 error when a class property is declared without an initializer and is not definitely assigned in the constructor. This is especially important for non-nullable properties.

### Success Criteria
- Emit TS2564 for properties without initializers that aren't assigned in constructor
- Handle definite assignment analysis for class properties
- Don't emit for properties with default values
- Don't emit for properties that are assigned in all constructor paths
- Don't emit for optional properties (with `?`)

### Implementation Steps
1. [ ] Read existing property initialization code in `src/thin_checker.rs`
2. [ ] Find where class properties are checked
3. [ ] Implement check: if property has no initializer and not assigned in constructor, emit TS2564
4. [ ] Use definite assignment analysis to check if property is assigned in all constructor paths
5. [ ] Handle parameter properties (they're auto-initialized)

### Key Code Locations
- `src/thin_checker.rs` - class property checking, constructor checking
- `src/checker/control_flow.rs` - definite assignment flow analysis
- `src/binder.rs` - property binding

### Test Cases to Implement
```typescript
// Should emit TS2564
class Foo {
  x: number; // Error: Property 'x' has no initializer and is not definitely assigned
}

// Should NOT emit (has initializer)
class Bar {
  x: number = 5; // OK
}

// Should NOT emit (assigned in constructor)
class Baz {
  x: number; // OK (assigned in constructor)
  constructor() {
    this.x = 5;
  }
}

// Should NOT emit (optional property)
class Qux {
  x?: number; // OK (optional)
}

// Should emit TS2564 (not all paths assign)
class Quux {
  x: number;
  constructor(flag: boolean) {
    if (flag) {
      this.x = 5;
    }
    // Error: 'x' not definitely assigned in all constructor paths
  }
}
```

## Task Queue
- [ ] After TS2564: coordinate with W1 on other definite assignment issues

## Completed
- [x] Fix New Expression Inference - Merged to squad/forge

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Implement TS2564 property initialization errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-5`
