# Worker 4 Plan - Squad Forge

## Mission
Fix Namespace Merging

Status: Active
Priority: P1 (High)

## Current Assignment
**Help W2 with Namespace Merging (12 failing tests)**

W2 has made changes to `bind_enum_declaration` to add enum members to exports, but tests are still failing.

### Failing Tests
1. `test_checker_namespace_merges_with_class_element_access`
2. `test_checker_namespace_merges_with_class_exports`
3. `test_checker_namespace_merges_with_class_exports_reverse_order`
4. And 9 more namespace_merges_with_* tests

### Next Steps
1. [ ] Review W2's pane output for specific test failures
2. [ ] Coordinate with W2 - don't duplicate work
3. [ ] Check if exports are properly combined when namespace merges with class
4. [ ] Verify class static members are added to namespace exports
5. [ ] Test: `./wasm/test.sh 2>&1 | grep namespace_merges_with_class`

### Key Code Locations
- `src/binder.rs` - `can_merge_flags()`, `bind_module_declaration()`, namespace exports
- `src/thin_checker.rs` - namespace member resolution

## Task Queue
- [ ] After namespace merging: element access literal keys (if W3 hasn't completed)

## Completed
- [x] (Move finished items here)

## Ready for Merge
No

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] binder: Fix namespace+class merge exports`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
- COORDINATE with W2 - this is a collaborative task
