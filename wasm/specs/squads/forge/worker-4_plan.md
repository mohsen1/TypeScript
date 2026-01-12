# Worker 4 Plan - Squad Forge

## Mission
Fix TS7006: Parameter Implicitly Has 'any' Type

Status: Active
Priority: P0 (CRITICAL)

## Current Assignment
**Implement TS7006 Error: Parameter implicitly has 'any' type**

### Background
TypeScript should emit TS7006 error when a function parameter has no type annotation and its type cannot be inferred. This commonly happens when:
1. Function parameters lack type annotations
2. No contextual type is available for inference
3. The `noImplicitAny` compiler option is enabled

### Success Criteria
- Emit TS7006 for parameters without type annotations when type cannot be inferred
- Don't emit when type can be inferred from context
- Handle all function types: function declarations, arrow functions, method signatures
- Account for contextual type inference

### Implementation Steps
1. [ ] Read existing parameter type checking code in `src/thin_checker.rs`
2. [ ] Find where parameter types are checked
3. [ ] Implement check: if parameter has no type and no inference source, emit TS7006
4. [ ] Test with various function patterns
5. [ ] Ensure no false positives when types can be inferred

### Key Code Locations
- `src/thin_checker.rs` - parameter type checking
- `src/solver/infer.rs` - type inference
- `src/checker/types/diagnostics.rs` - TS7006 error code

### Test Cases to Implement
```typescript
// Should emit TS7006
function foo(x) { } // Error: Parameter 'x' implicitly has 'any' type
const bar = (y) => { }; // Error: Parameter 'y' implicitly has 'any' type

// Should NOT emit (type can be inferred)
function baz(x: number) { } // OK
const qux = (z: string) => { }; // OK

// Should NOT emit (contextual type)
[1, 2, 3].forEach(n => console.log(n)); // OK - 'n' inferred as number
```

## Completed
- [x] Fix Element Access Literal Keys - Merged to squad/forge

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: Implement TS7006 implicit any parameter errors`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
