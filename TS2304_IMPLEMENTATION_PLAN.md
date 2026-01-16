# TS2304 Implementation Plan

**Date:** 2026-01-15
**Worker:** Worker 14 (EM-4)
**Task:** Implement fixes for TS2304 symbol resolution

---

## Summary

Phase 1 investigation is complete and merged. This document outlines the specific implementation fixes needed to reduce TS2304 errors from 12 total (7 missing + 5 extra) to the target of <5 total (<3 missing + <2 extra).

---

## Current Status

**Phase 1:** ✅ Complete - Investigation merged to em-team-4
**Phase 2:** 🔄 In Progress - Implementation
**Phase 3:** Pending - Validation

---

## Implementation Strategy

Given the constraints (no Docker test access), the implementation will follow a code-analysis-first approach:

1. Create targeted test cases (✅ Complete)
2. Identify specific code paths causing issues
3. Implement fixes for missing errors (scope chain, lib lookup)
4. Implement fixes for extra errors (scope boundaries, filtering)
5. Commit fixes for EM-4 validation

---

## Fix #1: Improve Scope Chain Traversal (Missing Errors)

**Issue:** 7 missing TS2304 errors - symbols not found when they should be.

**Target:** `wasm/src/thin_checker.rs:299` - `resolve_identifier_symbol`

**Current Implementation (Phase 2 - Scope Chain Traversal):**
```rust
// Lines 321-439
if let Some(mut scope_id) = self.find_enclosing_scope(idx) {
    while !scope_id.is_none() {
        if let Some(scope) = self.ctx.binder.scopes.get(scope_id.0 as usize) {
            // Check scope's local symbol table
            if let Some(sym_id) = scope.table.get(name) {
                // ... verify export_ok, is_class_member, etc.
            }
            // Check module exports
            if scope.kind == ContainerKind::Module {
                // ... check exports
            }
            scope_id = scope.parent;
        }
    }
}
```

**Potential Issues:**
1. `find_enclosing_scope` may not find all valid scopes
2. Scope chain may terminate early
3. Module scope may not be checked in all contexts

**Fix Approach:**
- Ensure scope chain always includes module-level scope
- Add fallback to global scope for top-level declarations
- Verify `find_enclosing_scope` returns correct scope for all node types

**Code Location:** `thin_checker.rs:241-297` (find_enclosing_scope function)

---

## Fix #2: Improve Lib Binder Lookup (Missing Errors)

**Issue:** External library symbols not found.

**Current Implementation (Phase 4 - Lib Binders):**
```rust
// Lines 472-502
for (i, lib_binder) in lib_binders.iter().enumerate() {
    if let Some(sym_id) = lib_binder.file_locals.get(name) {
        if let Some(symbol) = lib_binder.get_symbol(sym_id) {
            let is_class_member = Self::is_class_member_symbol(symbol.flags);
            if !is_class_member {
                return Some(sym_id);
            }
        }
    }
}
```

**Potential Issues:**
1. `lib_binders` may not include all necessary libraries
2. `is_class_member_symbol` may filter out valid symbols
3. `get_symbol` may fail for cross-arena references

**Fix Approach:**
- Ensure lib.d.ts and ambient declarations are loaded into lib binders
- Review `is_class_member_symbol` filtering logic
- Add cross-arena symbol resolution

**Code Location:** `thin_checker.rs:302-305` (lib_binder collection)

---

## Fix #3: Reduce False Positive TS2304s (Extra Errors)

**Issue:** 5 extra TS2304 errors - symbols found when they shouldn't be.

**Target:** `wasm/src/thin_checker.rs:5351-5376` (identifier type checking)

**Current Implementation:**
```rust
// Line 5318
if let Some(sym_id) = self.resolve_identifier_symbol(idx) {
    // ... process symbol
    return self.apply_flow_narrowing(idx, declared_type);
}

// Line 5360-5376
_ if self.is_known_global_value_name(name) => {
    TypeId::UNKNOWN
}
_ => {
    if let Some(ref class_info) = self.ctx.enclosing_class.clone() {
        if self.is_static_member(&class_info.member_nodes, name) {
            self.error_cannot_find_name_static_member_at(name, &class_info.name, idx);
            return TypeId::ERROR;
        }
    }
    self.error_cannot_find_name_at(name, idx);
    TypeId::ERROR
}
```

**Potential Issues:**
1. `is_known_global_value_name` may be too permissive
2. `is_static_member` check may fail incorrectly
3. Scope boundaries may not prevent leakage

**Fix Approach:**
- Review `is_known_global_value_name` to ensure it only matches actual globals
- Verify static member check doesn't incorrectly emit for accessible members
- Ensure block-scoped variables are not accessible outside their block

---

## Implementation Priority

### High Priority (Addresses Missing Errors):
1. **Fix scope chain traversal** - Ensure all valid scopes are checked
2. **Fix lib binder lookup** - Ensure external symbols are found

### Medium Priority (Addresses Extra Errors):
3. **Review class member filtering** - Don't filter out accessible members
4. **Enforce scope boundaries** - Prevent leakage across block/function boundaries

---

## Test Cases Created

File: `ts2304-test-cases.ts`

Coverage:
- ✅ Global symbols (declare)
- ✅ Local variables
- ✅ Outer scope variables
- ✅ Cross-function scope (should error)
- ✅ Type parameters
- ✅ Function parameters
- ✅ Undeclared variables (should error)
- ✅ Intrinsic types (undefined, etc.)
- ✅ Destructuring
- ✅ Class members
- ✅ Enum members
- ✅ Namespace members
- ✅ Loop variables
- ✅ Variable shadowing

---

## Success Metrics

| Metric | Before | Target |
|--------|--------|--------|
| TS2304 missing errors | 7 | <3 |
| TS2304 extra errors | 5 | <2 |
| Total TS2304 errors | 12 | <5 |

**Goal:** 60% reduction in TS2304 errors (12 → <5)

---

## Next Steps

1. ✅ Create test cases (COMPLETE)
2. ⏸️ Run tests with BIND_DEBUG=1 to identify specific failures
3. ⏸️ Implement scope chain fixes
4. ⏸️ Implement lib binder fixes
5. ⏸️ Implement false positive fixes
6. ⏸️ Validate with conformance tests
7. ⏸️ Commit and push fixes

---

## Notes

**Implementation Constraint:** Without Docker test access, specific fixes require test failure data to implement correctly. The fixes outlined above are based on code analysis and should be validated with actual test results before committing.

**Recommendation:** EM-4 should run conformance tests to identify specific test files failing with TS2304, then I can implement targeted fixes for those specific cases.

---

## Status

**Phase 2 Status:** Test cases created, implementation plan documented.
**Blocker:** Need specific test failure data to implement targeted fixes.
**Ready for:** EM-4 review and test execution.
