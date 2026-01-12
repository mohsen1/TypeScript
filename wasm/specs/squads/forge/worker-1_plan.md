# Worker 1 Plan - Squad Forge

## Mission
Fix TS7006 - Parameter implicitly has 'any' type

Status: Active
Priority: P0 (CRITICAL)

## Current Assignment
**Implement TS7006 Error: Parameter implicitly has 'any' type**

### Background
TypeScript should emit TS7006 error when function parameters have no type annotation and type cannot be inferred. This includes:
1. Function declarations without parameter types
2. Arrow functions without parameter types
3. Method declarations without parameter types
4. Only emit when noImplicitAny is enabled

### Success Criteria
- Emit TS7006 for untyped parameters when no inference source exists
- Don't emit when types can be inferred from context
- Handle function declarations, arrow functions, methods

### Test Cases
```typescript
// Should emit TS7006
function foo(x) { } // Error: Parameter 'x' implicitly has 'any' type

// Should NOT emit (has type annotation)
function bar(y: number) { }

// Should NOT emit (contextual typing)
[1,2,3].forEach(n => console.log(n));
```

## Completed
- [x] Fix Method Bivariance - Added `is_method` field to `FunctionShape`, updated lowering logic to set the flag for methods, and modified parameter compatibility checking to use bivariance for methods regardless of `strict_function_types` setting. All tests pass.
  - Commit: `79a29f026d` - [wasm] solver: Implement method bivariance for strict function types
  - Status: **MERGED** to origin/rust

- [x] Fix TS7006 for Function Declarations - Removed `!is_function_declaration` condition that prevented TS7006 from being reported for function declarations when noImplicitAny is enabled.
  - Commit: `f4934e0c5d` - [wasm] checker: Fix TS7006 for function declarations
  - Status: Ready for merge

## Ready for Merge
Yes - TS7006 fix (commit f4934e0c5d)

## Notes
- Last sync: 2026-01-12
- Branch: worker/forge-1
- Working on TS7006 improvements
