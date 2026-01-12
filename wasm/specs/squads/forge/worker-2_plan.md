# Worker 2 Plan - Squad Forge

## Mission
Fix Namespace Merging - Part 1: Binder changes

Status: Active - Tests failing, needs fixes
Priority: P1 (High)

## Current Assignment
**Fix Namespace Merging Binder Implementation**

### Background
When a class/enum/function and namespace share the same name, they should merge into a single symbol. Current implementation has test failures.

### Test Failures (Needs Debugging)
The following tests are failing after namespace+class merge implementation:
1. ❌ `test_checker_namespace_merges_across_decls_value_access` - TypeId mismatch (4 vs 9)
2. ❌ `test_checker_namespace_merges_with_class_element_access` - TypeId mismatch (111 vs 9)

### What Was Implemented
- ✅ Updated `can_merge_flags()` to allow MODULE + CLASS merging
- ✅ Updated `bind_class_declaration()` to populate `symbol.exports` with static members
- ✅ Updated `bind_module_declaration()` to merge exported members
- ✅ Updated `bind_enum_declaration()` to add enum members to exports

### Debugging Needed
The exports are being populated but the type resolution is returning wrong TypeIds. Possible issues:
1. Symbol exports not being correctly combined during merge
2. Checker not looking up the merged symbol's exports properly
3. TypeIds not matching between declaration and access

### Next Steps
1. [ ] Debug why TypeId mismatch occurs in failing tests
2. [ ] Check if `get_type_of_symbol` correctly resolves merged symbols
3. [ ] Verify symbol.exports is properly maintained across merges
4. [ ] Test with: `./wasm/test.sh namespace_merges 2>&1 | grep -E "FAIL|PASS"`

### Key Code Locations
- `src/binder.rs` - `can_merge_flags()`, `bind_class_declaration()`, `bind_module_declaration()`
- `src/thin_binder.rs` - enum export population
- `src/thin_checker.rs` - symbol resolution and type access

## Task Queue
- [ ] Fix failing tests (priority)
- [ ] Coordinate with Worker 4 on namespace merging

## Completed
- [x] Implemented namespace+class merging in binder
- [x] Implemented enum export population
- [x] Added exports to module declarations

## Ready for Merge
No - Tests failing, needs fixes

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] binder: Fix namespace+class merge type resolution`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-2`
- **IMPORTANT**: Fix test failures before requesting merge
