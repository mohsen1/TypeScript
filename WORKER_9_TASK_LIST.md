# Worker 9 Task List

## Squad: Binder (CRITICAL)

## Current Task
- [ ] Fix type-only import and export bindings

## Queue
- [ ] Ensure type-only imports create proper symbol bindings
- [ ] Fix re-export chains for type declarations
- [ ] Handle `export type` and `import type` correctly in all scenarios

## Completed
- [x] Fix namespace import handling in bind_import_declaration
- [x] Audit how import type and export type are processed
- [x] Fix re-export chains for namespace imports

## Context
Type imports and exports may not be creating proper bindings, leading to TS2304 errors when referencing imported types. Namespace imports were not working correctly.

---

## Implementation Summary

### Namespace Import Fix

**Problem:** `import * as ns from 'module'` was not creating proper bindings for `ns.member`.

**Solution:** Fixed `bind_import_declaration` in `wasm/src/thin_binder.rs` to correctly handle namespace imports.

**Changes:**
- `wasm/src/thin_binder.rs`: Added namespace import handling (19 new lines)
  - Detect namespace import syntax (`import * as name`)
  - Create proper symbol bindings for namespace members
  - Ensure namespace symbols are resolvable in child scopes

**Impact:** Fixes TS2304 errors when using namespace imports.
