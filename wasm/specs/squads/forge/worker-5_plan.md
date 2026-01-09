# Worker 5 Plan - Squad Forge

## Mission
Implement TS7010/TS7006 function return type checking.

Status: Active
Priority: 1

## Current Assignment
Implement missing return statement detection and implicit any parameter checking.

**Error Codes:**
- TS7010 - "Function lacks ending return statement and return type does not include 'undefined'"
- TS7006 - "Parameter 'X' implicitly has an 'any' type"

**Impact:** 80 conformance tests affected

### Background
TypeScript validates function signatures:
```typescript
// TS7010: missing return
function foo(): number {
  // no return statement!
}

// TS7006: implicit any
function bar(x) {  // x has implicit any
  return x;
}
```

### Steps
1. **Add test cases first**:
   ```typescript
   // Should error: TS7010
   function noReturn(): number {
     console.log("oops");
     // no return
   }

   // Should error: TS7010 - not all paths return
   function maybeReturn(flag: boolean): number {
     if (flag) { return 1; }
     // else path doesn't return
   }

   // Should NOT error: all paths return
   function allReturn(flag: boolean): number {
     if (flag) { return 1; }
     return 2;
   }

   // Should NOT error: void return type
   function voidReturn(): void {
     console.log("ok");
   }

   // Should error: TS7006
   function implicitAny(x) {  // noImplicitAny
     return x;
   }

   // Should NOT error: explicit type
   function explicitType(x: number) {
     return x;
   }
   ```

2. **Implement TS7010 (missing return)**:
   - For functions with non-void return type
   - Walk all control flow paths
   - Ensure every path ends with a return statement
   - Exception: functions that always throw

3. **Implement TS7006 (implicit any)**:
   - For each function parameter
   - Check if type annotation present
   - Check if type can be inferred from context
   - If neither, emit TS7006 (when `noImplicitAny` is true)

4. **Handle edge cases**:
   - Arrow functions with expression bodies (implicit return)
   - Generators (yield paths)
   - Async functions (Promise wrapping)
   - Never-returning functions

5. **Run conformance tests** and report numbers.

### Key Files
- `wasm/src/thin_checker.rs` - function checking
- `wasm/src/checker/statements.rs` - statement/return checking
- `wasm/src/checker/control_flow.rs` - reachability analysis

### Success Criteria
- TS7010 emitted for missing returns
- TS7006 emitted for implicit any parameters
- Correct handling of control flow for return analysis

## Task Queue
(empty - single focused task)

## Completed
(previous work cleared - fresh start for Operation Conformance)

## Ready for Merge
No

## Notes
- Control flow analysis similar to TS2454 - coordinate with Worker 3
- Need to track return statements, not assignments
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: implement TS7010/TS7006 function return checking`
- Push to: `origin/worker/forge-5`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
