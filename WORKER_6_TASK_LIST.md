# WORKER-6 TASK LIST

## Squad: Semantics (Module Symbol Resolution)
## EM: EM-2
## Branch: worker-6

---

## Primary Task: Module Symbol Resolution - Re-exports and TS2792

**Priority:** 🔴 CRITICAL (Priority 1 for EM-2)
**Team:** Worker 6 (re-exports, TS2792) + Worker 7 (namespace, defaults)

### Problem
- Cross-file module symbol resolution is broken (800+ errors)
- Re-exports (`export * from 'x'`, `export { a } from 'x'`) don't resolve correctly
- TS2792 `import()` type assertions don't work
- This blocks real-world TypeScript projects with multiple files

### Action Items (Two-Pronged Approach with Worker 7)

**Subtask 6.1: Implement Re-export Resolution**
- Track `export * from 'module'` declarations during binding
- Build export tables that include re-exported symbols
- Resolve re-exported symbols when referenced from other files
- Handle circular re-exports correctly

**Subtask 6.2: Implement Named Re-exports**
- Track `export { name } from 'module'` declarations
- Map exported names to source module symbols
- Support aliased re-exports (`export { a as b } from 'x'`)
- Merge named re-exports with regular exports

**Subtask 6.3: Implement TS2792 `import()` Type Resolution**
- Handle `import('./module')` type expressions
- Resolve module-relative type imports
- Support dynamic type queries in type positions
- Test with common TypeScript patterns

**Subtask 6.4: Test with Conformance Suite**
- Run conformance tests for module resolution
- Verify TS7005, TS7008, TS2792 error counts decrease
- Coordinate with Worker 7 to prevent merge conflicts

### Files to Work On
- `wasm/src/thin_binder.rs` (export tracking, re-export binding)
- `wasm/src/binder/mod.rs` (shared module resolution logic)
- `wasm/src/thin_checker.rs` (import() type resolution)
- Test files for re-export scenarios

### Success Criteria
- Re-exports resolve correctly across files
- Named re-exports work with and without aliases
- TS2792 `import()` type assertions work
- Reduced error counts: TS7005, TS7008, TS2792
- Conformance test improvements

### Commit Requirement
- Daily commits with format: `[wasm] binder: module - <subtask description>`
- Example: `[wasm] binder: module - implement export * re-export resolution`

### Coordination
- EM-2 facilitates daily sync between Workers 6-7
- Shared workspace: `wasm/src/binder/mod.rs`, `wasm/src/thin_binder.rs`
- Code reviews required before merging to em-team-2

---

## Instructions
1. Work on the worker-6 branch
2. Start with Subtask 6.1: Re-export Resolution
3. Commit daily with format: `[wasm] binder: module - <subtask>`
4. Push to `worker-6` branch when ready for review
5. EM-2 will merge to em-team-2, then to rust
6. Coordinate with Worker 7 to avoid merge conflicts

---

## Subtask Progress Tracking

### Subtask 6.1: Re-export Resolution
- [ ] Track `export * from 'module'` declarations
- [ ] Build export tables with re-exported symbols
- [ ] Resolve re-exported symbols
- [ ] Handle circular re-exports
- [ ] Test re-export resolution

### Subtask 6.2: Named Re-exports
- [ ] Track `export { name } from 'module'` declarations
- [ ] Map exported names to source symbols
- [ ] Support aliased re-exports
- [ ] Merge with regular exports
- [ ] Test named re-exports

### Subtask 6.3: TS2792 `import()` Resolution
- [ ] Handle `import('./module')` type expressions
- [ ] Resolve module-relative type imports
- [ ] Support dynamic type queries
- [ ] Test import() type resolution

### Subtask 6.4: Conformance Testing
- [ ] Run conformance tests
- [ ] Verify error count reduction
- [ ] Document results

---

## Task Completion Report

### Previous Work Completed
**Task 1:** TS2304 Fix - Lib Symbol Loading (✅ Completed)
**Commit:** 85d474de80 - "Fix: Ensure lib symbols are available during binding"
**Date:** 2026-01-15

### Changes Made
- Fixed lib symbols being merged into current_scope after it's created
- Ensured console, Array, Promise, and other lib.d.ts symbols available during binding
- Added 8 lines to wasm/src/thin_binder.rs to preserve lib symbols

### Results
- All TS2304-related tests passing (19/19)
- Lib symbols now correctly available during binding and type checking
- Merged into rust branch successfully

---

**Task 2:** Global Interface Merging Fix (✅ Completed)
**Commit:** 96644afc1 - "Fix: Global interface merging for declare global blocks"
**Date:** 2026-01-15

### Changes Made
- Modified `resolve_named_type_reference` in `wasm/src/thin_checker.rs` to check for global augmentation symbols
- When a type reference is in `global_augmentations`, use `resolve_lib_type_by_name` to properly merge lib.d.ts declarations with user's `declare global` augmentations
- Added 8 lines to handle the augmentation merge path

### Results
- All global augmentation tests passing (3/3)
- `interface Window` from lib.dom.d.ts now properly merges with user's `declare global { interface Window { ... } }`
- Augmented properties are available and type-checked correctly
- No TS2304 or TS2339 errors for properly augmented global interfaces
- Merged into rust branch successfully

---

## Task Status Update
✅ **TS2304 - COMPLETE:** Worker 6 successfully fixed lib.d.ts symbol loading
✅ **Global Interface Merging - COMPLETE:** Fixed global interface merging across multiple files
🔄 **Module Symbol Resolution (Re-exports + TS2792) - ACTIVE:** Implementing cross-file symbol resolution
