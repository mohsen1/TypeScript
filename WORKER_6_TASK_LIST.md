# WORKER-6 TASK LIST

## Squad: Binder Squad
## EM: EM-2
## Branch: worker-6

---

## Primary Task: Fix Global Interface Merging (Multiple Files Contributing to Global Scope)

**Priority:** 🔴 CRITICAL (Priority 1 for EM-2)

### Problem
- Global interfaces from multiple files don't merge correctly
- When `interface Window` is defined in multiple files (lib.dom.d.ts + user code), declarations don't combine
- This causes missing properties and methods on global types
- The `global_augmentations` tracking exists but merging logic may be incomplete

### Action Items
1. **Verify Global Augmentation Tracking**
   - Check that `declare global` blocks are properly detected
   - Verify `global_augmentations` map is populated during binding

2. **Fix Interface Merging Across Files**
   - Ensure interface declarations in `declare global` blocks merge with lib declarations
   - Verify the merger creates intersection types combining both declarations
   - Test that properties from both lib and user code are available

3. **Fix Type Resolution for Augmented Interfaces**
   - Ensure `resolve_lib_type_by_name` includes augmentation declarations
   - Verify type lowering creates proper intersection types
   - Test that augmented interfaces work in type checking

### Files to Work On
- `wasm/src/thin_binder.rs` (global_augmentations tracking)
- `wasm/src/thin_checker.rs` (resolve_lib_type_by_name)
- Test files for global augmentation scenarios

### Success Criteria
- `interface Window` from lib.dom.d.ts merges with user's `declare global { interface Window { ... } }`
- Augmented properties are available and type-checked correctly
- No TS2304 or TS2339 errors for properly augmented global interfaces
- Existing lib symbol loading remains working

### Testing
- Create test case with `declare global` augmenting lib interface
- Verify properties from both sources are available
- Run conformance tests for global augmentations

---

## Instructions
1. Work on the worker-6 branch
2. Focus on global interface merging across files
3. Push to `worker-6` branch when ready for review
4. EM-2 will merge and validate

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

## Task Status Update
✅ **TS2304 - COMPLETE:** Worker 6 successfully fixed lib.d.ts symbol loading
🔄 **Global Interface Merging - NEW TASK:** Fixing global interface merging across multiple files
