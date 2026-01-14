# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [x] Reduce "Any" fallback in accessor patterns (Phase 6)

## Completed
- [x] Locate accessor fallback locations (found 2)
- [x] Change both accessor fallbacks from TypeId::ANY to TypeId::UNKNOWN
- [x] Verify compilation - Code compiles successfully
- [x] Test - Two pre-existing failures (unrelated to changes)

## Context

**Previous Work Completed**
Worker 12 has systematically reduced "Any" fallback usage across 6 phases:
1. ✅ Type parameter defaults (7 locations)
2. ✅ Property access patterns (5 locations)
3. ✅ Contextual type patterns (3 locations)
4. ✅ This-type patterns (1 location)
5. ✅ Array element type patterns (3 locations)
6. ✅ Accessor patterns (2 locations)

**Total: 21 locations changed from Any→Unknown (Phases 1-5 merged to rust, Phase 6 complete)**

**New Focus: Accessor Fallbacks**
Accessors are used for property getters/setters:
- When reading or writing object properties
- The checker resolves the accessor's type
- When the accessor type is unknown, it currently falls back to `Any`

**Targeted Changes**
Change accessor fallbacks in `thin_checker.rs`:

1. Line ~4357: `accessor.getter.or(accessor.setter).unwrap_or(TypeId::ANY)`
2. Line ~4979: `accessor.getter.or(accessor.setter).unwrap_or(TypeId::ANY)`

These are in:
- Property accessor type resolution for interface merging
- Combined accessor type resolution (getter or setter)

**Why This Matters**
When property accessors have no explicit type:
- **Current**: Returns `Any` (loses type safety for the property)
- **New**: Returns `Unknown` (maintains strictness, emits errors)

This will expose bugs in:
- Properties with only getter or setter (no accessor type)
- Interface property merging without type annotations
- Incomplete accessor definitions

## Queue
- [ ] After accessor fix, measure conformance impact
- [ ] Tackle argument type fallbacks
- [ ] Tackle remaining type_stack patterns
- [ ] Coordinate with Solver Squad on type inference improvements

## Implementation Steps

1. **Make targeted changes**
   - Change accessor fallbacks from `TypeId::ANY` to `TypeId::UNKNOWN`
   - Focus on: getter/setter accessor resolution
   - Found ~2 locations in thin_checker.rs

2. **Test incrementally**
   - Run `./wasm/test.sh` after changes
   - Verify compilation succeeds
   - Check for new test failures

3. **Document findings**
   - Note which error codes increase
   - Identify patterns in exposed bugs

## Files to Modify
- `wasm/src/thin_checker.rs` - Accessor fallbacks (~2 locations)

## Success Criteria
- Code compiles without errors
- No critical test crashes (pre-existing test_closure_capture_with_array_filter failure is OK)
- Measurable increase in detected type errors related to properties

## Ready for Merge
Yes - Phase 6 complete and tested.

## Changes Summary
- **Files modified**: 1 (`wasm/src/thin_checker.rs`)
- **Lines changed**: 2 locations
- **Change**: Accessor fallbacks from `TypeId::ANY` → `TypeId::UNKNOWN`
- **Commit**: `ac15353b6 Complete: Phase 6 - Accessor fallback from Any to Unknown`
- **Pushed to**: origin/worker-12
