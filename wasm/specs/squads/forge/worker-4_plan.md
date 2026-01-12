# Worker 4 Plan - Squad Forge

## Mission
Fix TS7006: Parameter Implicitly Has 'Any' Type

Status: Active
Priority: P1 (HIGH)

## Current Assignment
**Implement TS7006 Error: Parameter implicitly has 'any' type (42 occurrences)**

### Background
TypeScript should emit TS7006 when a function parameter has an implicit 'any' type and the `noImplicitAny` compiler option is enabled.

### Success Criteria
- Emit TS7006 for function/method parameters without type annotations when noImplicitAny is true
- Handle all function types: function declarations, expressions, arrow functions, methods
- Don't emit when parameter has explicit type annotation
- Don't emit when parameter has default value
- Don't emit when noImplicitAny is false

### Implementation Steps
1. [ ] Sync from origin/rust: `git fetch origin && git merge origin/rust --no-edit`
2. [ ] Search for parameter type checking code in `src/thin_checker.rs`
3. [ ] Find where implicit any is detected for parameters
4. [ ] Implement TS7006 emission when noImplicitAny is true and parameter has no type annotation
5. [ ] Test with various function patterns
6. [ ] Run conformance to verify TS7006 is emitted correctly

### Key Code Locations
- `src/thin_checker.rs` - function parameter type checking
- `src/checker/types/diagnostics.rs` - TS7006 error code
- `src/cli/args.rs` - noImplicitAny flag

### Test Cases
```typescript
// @noImplicitAny: true

// Should emit TS7006
function foo(x) { } // Error: Parameter 'x' implicitly has 'any' type

const bar = (y) => { }; // Error: Parameter 'y' implicitly has 'any' type

class Baz {
  method(z) { } // Error: Parameter 'z' implicitly has 'any' type
}

// Should NOT emit
function qux(a: number) { } // OK - explicit type
function quux(b = 5) { } // OK - has default value
```

## Completed
- [x] TS2792 - Module resolution (reassigned)
- [x] TS7010 - Implicit any return type - Merged to squad/forge ✅

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Implement TS7006 implicit any parameter errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
