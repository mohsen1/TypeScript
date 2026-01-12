# Worker 4 Plan - Squad Forge

## Mission
Help W2 Fix Namespace Merging (P1 Priority)

Status: Active
Priority: P1 (High)

## Current Assignment
**Collaborate with W2 to fix namespace merging test failures**

### Background
W2 has implemented namespace+class merging in the binder, but 2 tests are failing with TypeId mismatches. This is a high-priority collaborative task to get namespace merging working.

### Test Failures (from W2)
1. ❌ `test_checker_namespace_merges_across_decls_value_access` - TypeId mismatch (4 vs 9)
2. ❌ `test_checker_namespace_merges_with_class_element_access` - TypeId mismatch (111 vs 9)

### What W2 Implemented
- ✅ Updated `can_merge_flags()` to allow MODULE + CLASS merging
- ✅ Updated `bind_class_declaration()` to populate `symbol.exports` with static members
- ✅ Updated `bind_module_declaration()` to merge exported members
- ✅ Updated `bind_enum_declaration()` to add enum members to exports

### Debugging Approach
The exports are being populated but type resolution returns wrong TypeIds. Investigate:
1. **Symbol exports combination**: Are exports correctly merged when namespace merges with class?
2. **Checker lookup**: Does `get_type_of_symbol` correctly resolve merged symbols?
3. **TypeId consistency**: Do TypeIds match between declaration and access?

### Implementation Steps
1. [ ] Coordinate with W2 - don't duplicate work
2. [ ] Read the failing tests to understand expected vs actual behavior
3. [ ] Add debug logging to trace symbol export resolution
4. [ ] Check if the checker is looking up the right symbol after merge
5. [ ] Fix the type resolution issue
6. [ ] Test: `./wasm/test.sh namespace_merges 2>&1 | grep -E "FAIL|PASS"`

### Key Code Locations
- `src/binder.rs` - `can_merge_flags()`, `bind_class_declaration()`, `bind_module_declaration()`
- `src/thin_binder.rs` - enum export population
- `src/thin_checker.rs` - symbol resolution and type access

## Task Queue
- [ ] After namespace merging: other element access issues

## Completed
- [x] Fix Element Access Literal Keys - Merged to squad/forge

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker/binder: Fix namespace+class merge type resolution`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
- **IMPORTANT**: Coordinate with W2 - this is collaborative work
