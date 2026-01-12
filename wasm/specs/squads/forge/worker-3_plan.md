# Worker 3 Plan - Squad Forge

## Mission
Fix Namespace Merging - Part 2: Enum and Function merging

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement Enum/Function + Namespace Merging (7 failing tests)**

### Background
Enums and functions can also merge with namespaces. This is complementary to Worker 2's class merging work.

### Failing Tests (your subset)
1. `test_checker_namespace_merges_with_enum_value_exports`
2. `test_checker_namespace_merges_with_enum_value_exports_reverse_order`
3. `test_checker_namespace_merges_with_function_value_exports`
4. `test_checker_namespace_merges_with_function_value_exports_reverse_order`
5. `test_checker_namespace_merges_across_decls_value_access`
6. `test_enum_namespace_merging`
7. `test_checker_typeof_namespace_alias_member`

### Implementation Steps
1. [ ] Read failing tests to understand expected behavior
2. [ ] Update `bind_enum_declaration()` in `src/binder.rs`:
   - Put enum members into `symbol.exports` (enums act like namespaces with constants)
3. [ ] Update `bind_function_declaration()` to support namespace merging:
   - When function has same name as namespace, merge exports
4. [ ] Ensure `can_merge_flags()` handles ENUM + MODULE and FUNCTION + MODULE
5. [ ] Test with: `./wasm/test.sh 2>&1 | grep -E "namespace_merges_with_(enum|function)|enum_namespace"`

### Key Code Locations
- `src/binder.rs` - `bind_enum_declaration()`, `bind_function_declaration()`
- `src/binder.rs` - `symbol_flags::ENUM`, `symbol_flags::FUNCTION`

### Enum Special Handling
Enums are special - their members should go to `symbol.exports`:
```rust
// In bind_enum_declaration
for member in &enum_decl.members {
    let member_id = self.declare_symbol(member_name, symbol_flags::ENUM_MEMBER, member_idx);
    if let Some(sym) = self.symbols.get_mut(enum_symbol_id) {
        sym.exports.get_or_insert_with(|| Box::new(SymbolTable::new()))
           .set(member_name.clone(), member_id);
    }
}
```

## Task Queue
- [ ] After enum/function: coordinate with Worker 2 on checker resolution

## Completed
- [x] (Move finished items here)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] binder: Implement enum/function + namespace merging`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-3`
