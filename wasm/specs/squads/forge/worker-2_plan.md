# Worker 2 Plan - Squad Forge

## Mission
Fix Namespace Merging - Part 1: Binder changes

Status: Active
Priority: P1 (High)

## Current Assignment
**Implement Namespace Merging in Binder (12 failing tests)**

### Background
When a class/enum/function and namespace share the same name, they should merge into a single symbol. Currently the binder creates separate symbols or fails to merge exports properly.

### Failing Tests (your subset - class merging)
1. `test_checker_namespace_merges_with_class_exports`
2. `test_checker_namespace_merges_with_class_exports_reverse_order`
3. `test_checker_namespace_merges_with_class_value_exports`
4. `test_checker_namespace_merges_with_class_value_exports_reverse_order`
5. `test_checker_namespace_merges_with_class_element_access`

### Implementation Steps
1. [ ] Read failing tests to understand expected behavior
2. [ ] Update `can_merge_flags()` in `src/binder.rs`:
   ```rust
   // Allow merging Module with Class, Function, or Enum
   if (existing & symbol_flags::MODULE != 0) &&
      (new & (symbol_flags::CLASS | symbol_flags::FUNCTION | symbol_flags::ENUM) != 0) {
       return true;
   }
   // And vice versa
   ```
3. [ ] Update `bind_class_declaration()` to populate `symbol.exports` with static members
4. [ ] Update `bind_module_declaration()` to merge exported members into `symbol.exports`
5. [ ] Test with: `./wasm/test.sh 2>&1 | grep -E "namespace_merges_with_class"`

### Key Code Locations
- `src/binder.rs` - `can_merge_flags()`, `bind_class_declaration()`, `bind_module_declaration()`
- `src/binder.rs` - `SymbolTable`, `symbol.exports`, `symbol.members`

### Symbol Flag Combination
When class + namespace merge: `CLASS | VALUE_MODULE | NAMESPACE_MODULE`

## Task Queue
- [ ] Coordinate with Worker 3 on enum/function merging

## Completed
- [x] (Move finished items here)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] binder: Implement class + namespace symbol merging`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
