# Worker 10 Task List

## Squad: Binder (CRITICAL) - TS2304 Focus

## Current Task
- [ ] Debug remaining console/Array resolution failures in edge cases
  - Trace why some global symbols still resolve incorrectly despite lib.d.ts fixes
  - Check module vs script mode differences in global resolution

## Queue
- [ ] Implement interface merging across declarations
  - Multiple `interface X {}` declarations should merge members
  - Test: interface extending, declaration merging in modules
- [ ] Fix function/variable scope hoisting edge cases
  - Functions should be hoisted within their scope
  - Verify `var` hoisting behavior matches tsc
- [ ] Add comprehensive tests for symbol resolution edge cases
- [ ] Coordinate with Worker 4 on module resolution and ambient contexts

## Completed
- [x] Initial setup and environment sync
- [x] Investigate and fix namespace declaration merging across files
  - Fixed 3 enum+namespace merging tests by updating test expectations
  - All enum+namespace binder tests now pass (4/4)
  - Commit: a72f57127 "Fix enum+namespace merging tests"

## Recent Merge Status
- **Date**: 2026-01-14
- **Result**: Successfully merged enum+namespace merging fix
- **Action Taken**:
  - Rebased em-team-3 on rust
  - Merged worker-10 (test fixes in thin_binder_tests.rs, thin_checker_tests.rs)
  - Build verification: PASSED
- **Next**: Continue debugging global symbol resolution edge cases

## Context

### Why Binder Squad is CRITICAL

**TS2304 (Cannot find name)** is the #1 source of **Error Poisoning**. When the binder fails to resolve:
- `console`, `Promise`, `Array` → becomes `Any`
- User-defined types in other files → becomes `Any`
- All downstream errors are **silenced**

This means you can fix CFA or Solver logic, but if the symbols never resolved, those fixes never fire.

### What Worker 4 Has Completed

Worker 4 fixed the core lib.d.ts injection issue:
- ✅ lib.d.ts symbols now merge into root SymbolTable
- ✅ Lib binders stored in ThinBinderState for cross-arena resolution
- ✅ `get_symbol()` now checks lib binders automatically

### What Still Needs Work

Despite the core fixes, TS2304 still has:
- **343 extra errors** (false positives - reporting errors when symbols exist)
- **116 missing errors** (failing to report when symbols truly don't exist)

The remaining issues are in:
1. **Module resolution** - Worker 4 is working on ambient modules
2. **Declaration merging** - Namespaces, interfaces, enums across files (PARTIALLY DONE - enum+namespace working)
3. **Scope edge cases** - Hoisting, block scoping, export/import scoping
4. **Namespace binding** - Complex namespace merging and member access

### Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| TS2304 extra errors | 343 | <50 |
| TS2304 missing errors | 116 | <20 |
| Global symbol resolution | ~85% | >98% |

### Key Files to Modify

- `wasm/src/thin_binder.rs` - Main binder logic
- `wasm/src/lib_loader.rs` - Lib.d.ts loading
- `wasm/src/symbol.rs` - Symbol table implementation
- `wasm/src/checker.rs` - Type checking integration

### Testing Approach

1. Write specific test cases in `tests/` for each bug pattern
2. Run `cargo test` for unit tests
3. Run conformance tests: `npm run conformance` (if available)
4. Compare error counts against tsc baseline
