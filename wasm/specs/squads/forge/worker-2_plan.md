# Worker 2 Plan - Squad Forge

## Mission
Implement TS2564 property initialization checking (class field analysis - edge cases).

Status: Active
Priority: 1

## Current Assignment
Handle edge cases for property initialization: control flow in constructors, inheritance, and special patterns.

**Error Code:** TS2564 - "Property 'X' has no initializer and is not definitely assigned in the constructor"

**Impact:** 135 conformance tests affected (shared with Worker 1)

### Background
Worker 1 handles the basic case. Worker 2 handles edge cases:
- Properties assigned in `if` branches (need all paths)
- Properties assigned via method calls in constructor
- Inherited properties from base class
- Properties with `declare` modifier (should skip)
- Abstract properties (should skip)

### Steps
1. **Add edge case test cases**:
   ```typescript
   // Should error: not assigned in all paths
   class Foo {
     name: string;
     constructor(flag: boolean) {
       if (flag) { this.name = "yes"; }
       // missing else branch
     }
   }

   // Should NOT error: assigned in all paths
   class Bar {
     name: string;
     constructor(flag: boolean) {
       if (flag) { this.name = "yes"; }
       else { this.name = "no"; }
     }
   }

   // Should NOT error: declare modifier
   declare class External {
     name: string;
   }

   // Should NOT error: abstract property
   abstract class Base {
     abstract name: string;
   }
   ```

2. **Extend control flow analysis**:
   - Use existing `checker/control_flow.rs` infrastructure
   - Track property assignments across all code paths
   - Handle try/catch/finally blocks
   - Handle early returns

3. **Handle inheritance**:
   - If base class constructor assigns a property, derived class shouldn't re-require it
   - Check for `super()` call location

4. **Run conformance tests** and report numbers.

### Key Files
- `wasm/src/thin_checker.rs` - main checker
- `wasm/src/checker/control_flow.rs` - flow analysis
- `wasm/src/ast.rs` - AST node types for modifiers

### Success Criteria
- Control flow analysis for constructor property assignment
- No false positives for edge cases
- Works correctly with inheritance

## Task Queue
(empty - single focused task)

## Completed
(previous work cleared - fresh start for Operation Conformance)

## Ready for Merge
No

## Notes
- Coordinate with Worker 1 (basic TS2564 implementation)
- Build on Worker 1's work, don't duplicate
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: TS2564 control flow and edge cases`
- Push to: `origin/worker/forge-2`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
