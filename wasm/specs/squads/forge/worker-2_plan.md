# Worker 2 Plan - Squad Forge

## Mission
Fix Namespace Merging - Part 1: Binder changes

Status: In Progress - Binder changes complete, investigating type checker issue
Priority: P1 (High)

## Current Assignment
**Implement Namespace Merging in Binder (5 failing tests)**

### Background
When a class/enum/function and namespace share the same name, they should merge into a single symbol. Currently the binder creates separate symbols or fails to merge exports properly.

### Failing Tests (your subset - class merging)
1. `test_checker_namespace_merges_with_class_exports`
2. `test_checker_namespace_merges_with_class_exports_reverse_order`
3. `test_checker_namespace_merges_with_class_value_exports`
4. `test_checker_namespace_merges_with_class_value_exports_reverse_order`
5. `test_checker_namespace_merges_with_class_element_access`

### Implementation Progress
1. [x] Read failing tests to understand expected behavior
2. [x] `can_merge_flags()` already supports MODULE + CLASS merging
3. [x] Implemented `populate_module_exports()` in `binder.rs`
4. [x] Implemented `populate_module_exports()` in `thin_binder.rs`
5. [x] Updated `bind_module_declaration()` to call `populate_module_exports()`
6. [ ] **Issue**: Type checker returns `ANY` (TypeId(4)) instead of `NUMBER` (TypeId(9))

### Remaining Investigation
The binder correctly:
- Merges CLASS + NAMESPACE_MODULE flags
- Populates `symbol.exports` with namespace exported members
- Calls `merge_namespace_exports_into_constructor()` in type checker

The issue appears to be in property access resolution:
- `Foo.value` should resolve to NUMBER type
- Type checker is returning ANY instead
- May need to investigate `property_access_type()` or `resolve_namespace_value_member()`

### Key Code Locations
- `wasm/src/binder.rs` - `populate_module_exports()`, `has_export_modifier()`
- `wasm/src/thin_binder.rs` - `populate_module_exports()`, `has_export_modifier_any()`
- `wasm/src/thin_checker.rs` - `merge_namespace_exports_into_constructor()`, `resolve_namespace_value_member()`
- `wasm/src/thin_checker.rs:4971` - Check for NAMESPACE_MODULE flag when computing type
- `wasm/src/thin_checker.rs:6904` - Property access via `resolve_namespace_value_member()`

### Symbol Flag Combination
When class + namespace merge: `CLASS | VALUE_MODULE | NAMESPACE_MODULE`

## Task Queue
- [ ] Fix type checker property access for merged symbols
- [ ] Coordinate with Worker 3 on enum/function merging

## Completed
- [x] Added `has_export_modifier()` helper to binder.rs
- [x] Added `populate_module_exports()` to binder.rs
- [x] Added `has_export_modifier_any()` helper to thin_binder.rs
- [x] Added `populate_module_exports()` to thin_binder.rs
- [x] Updated `bind_module_declaration()` in both binders to populate exports

## Ready for Merge
No - Tests still failing

## Notes
- Binder changes are complete and committed
- Type investigation needed to find why property access returns ANY instead of NUMBER
- Check if `merge_namespace_exports_into_constructor()` is working correctly
- Verify exports table is populated correctly during binding
