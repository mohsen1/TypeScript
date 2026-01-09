# Worker 3 Plan - Squad Forge

## Mission
Implement TS2454 definite assignment analysis for variables.

Status: Active
Priority: 1

## Current Assignment
Implement variable initialization tracking to detect use-before-assignment.

**Error Code:** TS2454 - "Variable 'X' is used before being assigned"

**Impact:** 104 conformance tests affected

### Background
TypeScript tracks whether variables are definitely assigned before use:
```typescript
let x: number;
console.log(x);  // TS2454: Variable 'x' is used before being assigned

let y: number;
y = 5;
console.log(y);  // OK
```

This requires control flow analysis to track variable state through branches.

### Steps
1. **Add test cases first**:
   ```typescript
   // Should error: TS2454
   function foo() {
     let x: number;
     return x;  // used before assigned
   }

   // Should error: TS2454 - not all paths assign
   function bar(flag: boolean) {
     let x: number;
     if (flag) { x = 1; }
     return x;  // might not be assigned
   }

   // Should NOT error: assigned in all paths
   function baz(flag: boolean) {
     let x: number;
     if (flag) { x = 1; } else { x = 2; }
     return x;  // definitely assigned
   }

   // Should NOT error: assigned before use
   function qux() {
     let x: number;
     x = 5;
     return x;
   }
   ```

2. **Implement variable tracking**:
   - Create a `DefiniteAssignmentChecker` or extend existing flow analysis
   - Track declared variables and their assignment state
   - At each variable reference, check if definitely assigned

3. **Handle control flow**:
   - If/else branches: both must assign for "definitely assigned"
   - Loops: conservative (assume loop might not execute)
   - Try/catch: handle exception paths
   - Switch: all cases must assign

4. **Emit TS2454** when variable is used but not definitely assigned.

5. **Run conformance tests** and report numbers.

### Key Files
- `wasm/src/thin_checker.rs` - main checker
- `wasm/src/checker/control_flow.rs` - flow analysis infrastructure
- `wasm/src/checker/statements.rs` - statement checking

### Success Criteria
- TS2454 emitted for use-before-assignment
- Correct handling of control flow branches
- No false positives for properly assigned variables

## Task Queue
(empty - single focused task)

## Completed
- [x] TS2454 error code and message added to diagnostics
- [x] DefiniteAssignmentAnalyzer implemented in control_flow.rs
- [x] Flow-based assignment tracking (ASSIGNMENT, BRANCH_LABEL, LOOP_LABEL, CONDITIONS)
- [x] Integration in thin_checker.rs for block-scoped variables without initializer
- [x] Test cases added in thin_checker_tests.rs
- [x] Fixed BindResult import in lib.rs

### Conformance Test Results (500 tests)
- Exact Match: 89 (18.3%)
- Same Error Count: 113 (23.2%)
- TS2454 false positives FIXED (no longer in top 10 extra errors)

## Ready for Merge
Yes

## Notes
- Similar infrastructure to TS2564 (property init) - share patterns with Workers 1-2
- Control flow analysis already exists in `control_flow.rs` - extend it
- `./wasm/test.sh` failed with existing repo errors (BindResult, TemplateLiteralSpan, object_with_index signature) unrelated to TS2454 changes.
- Commit format: `[wasm] checker: implement TS2454 definite assignment analysis`
- Push to: `origin/worker/forge-3`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
