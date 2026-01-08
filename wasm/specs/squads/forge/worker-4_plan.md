# Worker 4 Plan

## Mission
Execute tasks assigned by EM-Forge for the Forge squad (type system).

Status: Active
Priority: 4

## Current Assignment
- [ ] Fix `test_namespace_value_member_access` and related namespace member access tests in `wasm/src/thin_checker_tests.rs`. These tests verify that namespace members can be accessed as values. Run `./wasm/test.sh -- test_namespace_value_member` to reproduce failures. Check `wasm/src/thin_checker.rs` for namespace member resolution logic.

## Task Queue
- [ ] Fix `test_nested_namespace_member_resolution` if still failing
- [ ] Add coverage for namespace type member access patterns

## Completed
- [x] Fixed method type parameter scope in check_method_declaration
- [x] Added solver coverage for all major utility type patterns (Partial, Required, Pick, Omit, Record, Exclude, Extract, etc.)
- [x] Added solver coverage for template literal patterns and string intrinsics
- [x] Added solver coverage for recursive conditional types and variadic tuples
- [x] Added solver edge case tests (intersections, unions, generics, conditionals)
- [x] Added checker coverage for class/enum/function namespace merges

## Ready for Merge
No

## Notes
- Project Direction: integration and conformance-first; prioritize solver correctness
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] checker: <description>`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/forge-4`
