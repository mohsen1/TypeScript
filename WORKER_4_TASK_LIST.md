# Worker 4 Task List

## Current Task
- [ ] **BINDER-1: Audit lib.d.ts loading mechanism**
  - Find where `lib.d.ts` is parsed in the codebase
  - Trace how symbols are added to global scope
  - Identify why `console`, `Promise`, `Array` are missing
  - Document the current flow in a comment or doc

## Queue
- [ ] **BINDER-2: Fix global symbol table merging**
  - Ensure `lib.d.ts` symbols are correctly merged into root `SymbolTable`
  - Fix any scope chain issues preventing global access
  - Add tests that verify `console.log` resolves without TS2304
  - Verify `Promise`, `Array`, `Object` are accessible

## Completed
(none yet)
