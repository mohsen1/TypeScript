# Worker 2 Task List

Maintained by EM-2

## Completed Tasks

### Task 1: Global Scope Symbol Resolution ✅
- Added comprehensive global symbol resolution tests
- Validated TS2304 fixes from previous work
- Status: Completed and merged to rust

---

## Active Task

### Task 2: Fix TS2792 Module Import Errors (161 Missing)

**Priority:** 🔴 CRITICAL (161 missing occurrences)

**Problem:**
- TS2792 errors are not being emitted when module imports fail or are incorrect
- Examples:
  - Importing from non-existent modules
  - Circular import dependencies
  - Incorrect module syntax
  - Missing export in imported module

**Action Items:**

1. **Investigate Current Module Resolution**
   - Search for `TS2792` in codebase to see if check exists
   - Check `wasm/src/checker/` for module resolution code
   - Look for `check_module_import()` or similar functions

2. **Implement TS2792 Check**
   - For each import statement:
     - Resolve module path
     - Check if module exists and is accessible
     - Verify module exports the requested symbol
     - Emit TS2792 if:
       - Module not found
       - Module doesn't export requested symbol
       - Circular dependency detected
   - Key files to check:
     - `wasm/src/checker/modules.rs` (if exists)
     - `wasm/src/thin_binder.rs` - Import binding
     - `wasm/src/thin_checker.rs` - Module checking

3. **Handle Edge Cases**
   - Re-exports (export { x } from 'module')
   - Type-only imports
   - Dynamic imports
   - Relative vs absolute module paths
   - @types package resolution
   - node_modules resolution

4. **Testing**
   - Create test cases for missing modules
   - Create test cases for missing exports
   - Create test cases for circular dependencies
   - Verify TS2792 is emitted in all failure cases
   - Verify no false positives on valid imports

**Success Criteria:**
- Reduce Missing TS2792 from 161 to <20
- Check works for all import types (default, named, namespace, type-only)
- No false positives on valid module imports

**Test Cases:**
```typescript
// Should emit TS2792
import { MissingExport } from './module';
import { x } from 'nonexistent';
import { y } from './circular-a'; // circular-a imports from circular-b

// Should NOT emit TS2792
import { ValidExport } from './existing-module';
```

**Files to Work On:**
- `wasm/src/checker/modules.rs` - Main module checking (create if needed)
- `wasm/src/thin_binder.rs` - Import binding
- `wasm/src/checker/declarations.rs` - Import declarations

**Related Work:**
- Builds on Task 1 (global scope resolution)
- Connected to TS2304 (cannot find name) checks
- Module resolution depends on lib.d.ts loading (Worker 7's work)

**Target Branch:** rust

**Testing:**
- Run `./wasm/differential-test/run-conformance.sh --all` after changes
- Focus on tests in `declARATION/import` directories
- Verify TS2792 appears in test output where expected

---

## Notes
- Stay in worktree: /tmp/orchestrator-workspace/worktrees/worker-2
- Push to worker-2 branch when complete
- Do not touch other teams' directories
