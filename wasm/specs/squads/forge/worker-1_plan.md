# Worker 1 Plan - Squad Forge

## Mission
Implement TS2564 property initialization checking (constructor tracking).

Status: Active
Priority: 1

## Current Assignment
Implement property initialization checking for class properties in constructors.

**Error Code:** TS2564 - "Property 'X' has no initializer and is not definitely assigned in the constructor"

**Impact:** 135 conformance tests affected

### Background
TypeScript's `strictPropertyInitialization` requires that class properties either:
1. Have an initializer (`name: string = "default"`)
2. Are definitely assigned in the constructor
3. Have a `!` assertion (`name!: string`)
4. Are optional (`name?: string`)

Currently WASM checker does NOT emit TS2564. The code structure exists but the check is not wired up.

### Steps
1. **Find existing infrastructure** in `thin_checker.rs`:
   - Search for `check_class_declaration` function
   - Look for property initialization tracking (may be partial)
   - Find where TS2564 should be emitted

2. **Add test cases first** in `wasm/src/checker/tests/` or inline:
   ```typescript
   // Should error: TS2564
   class Foo {
     name: string;  // no initializer, not assigned in constructor
   }

   // Should NOT error
   class Bar {
     name: string;
     constructor() { this.name = "bar"; }
   }

   // Should NOT error
   class Baz {
     name: string = "default";
   }

   // Should NOT error
   class Qux {
     name?: string;
   }
   ```

3. **Implement tracking**:
   - In `check_class_declaration`, collect all non-optional properties without initializers
   - Walk constructor body to find `this.X = ...` assignments
   - For unassigned properties, emit TS2564

4. **Run conformance tests**:
   ```bash
   cd wasm/differential-test && node conformance-runner.mjs --max=500
   ```
   Report before/after numbers.

### Key Files
- `wasm/src/thin_checker.rs` - main checker, `check_class_declaration`
- `wasm/src/checker/control_flow.rs` - may have flow analysis helpers

### Success Criteria
- TS2564 emitted for uninitialized properties
- No false positives for properties assigned in constructor
- Conformance "missing errors" count reduced

## Task Queue
(empty - single focused task)

## Completed
(previous work cleared - fresh start for Operation Conformance)

## Ready for Merge
No

## Notes
- Coordinate with Worker 2 (also on TS2564 - they handle class field analysis)
- Run `./wasm/test.sh` before pushing
- Commit format: `[wasm] checker: implement TS2564 property initialization checking`
- Push to: `origin/worker/forge-1`
- **NEVER edit**: `STRUCTURE.md`, `GOALS.md`, other workers' plan files, or anything in `orchestrator/`
