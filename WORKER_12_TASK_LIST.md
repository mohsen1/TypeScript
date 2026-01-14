# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [x] Reduce "Any" fallback in contextual type patterns

## Completed
- [x] Locate contextual type fallback locations (found 3)
- [x] Change all 3 locations from TypeId::ANY to TypeId::UNKNOWN
- [x] Verify compilation - Code compiles successfully
- [x] Test - One pre-existing failure (unrelated to changes)

## Context

**Previous Work Completed**
Worker 12 has been incrementally reducing "Any" fallback usage:
1. ✅ Type parameter defaults (7 locations) - implemented, not merged to rust
2. ✅ Property access fallbacks (5 locations) - just merged to rust

**New Focus: Contextual Type Fallbacks**
Contextual types are used to provide type hints during type inference:
- When a value is used in a context that expects a specific type
- The checker provides contextual information to guide inference
- When the context type is unknown, it currently falls back to `Any`

**Targeted Changes**
Change contextual type fallbacks in `thin_checker.rs`:

1. Line ~9667: `helper.get_this_type().unwrap_or(TypeId::ANY)` → `unwrap_or(TypeId::UNKNOWN)`
2. Line ~9673: `contextual_type.unwrap_or(TypeId::ANY)` → `unwrap_or(TypeId::UNKNOWN)`
3. Line ~15473: `self.current_return_type().unwrap_or(TypeId::ANY)` → `unwrap_or(TypeId::UNKNOWN)`

These are in:
- Call expression this-type inference
- Contextual type helpers
- Return type contextual typing

**Why This Matters**
When contextual typing can't determine the expected type:
- **Current**: Returns `Any` (loses type safety, accepts invalid code)
- **New**: Returns `Unknown` (maintains strictness, emits errors for mismatches)

This will expose bugs in:
- Callback contexts without type annotations
- Return statements without inferred return types
- This-type inference in methods

## Queue
- [ ] After contextual type fix, measure conformance impact
- [ ] Tackle this-type fallbacks
- [ ] Tackle array element type fallbacks
- [ ] Coordinate with Solver Squad on type inference improvements

## Implementation Steps

1. **Make targeted changes**
   - Change contextual type fallbacks from `TypeId::ANY` to `TypeId::UNKNOWN`
   - Focus on: this-type, contextual type, and return type fallbacks
   - Found ~3 locations in thin_checker.rs

2. **Test incrementally**
   - Run `./wasm/test.sh` after changes
   - Verify compilation succeeds
   - Check for new test failures

3. **Document findings**
   - Note which error codes increase
   - Identify patterns in exposed bugs

## Files to Modify
- `wasm/src/thin_checker.rs` - Contextual type fallbacks (~3 locations)

## Success Criteria
- Code compiles without errors
- No critical test crashes (pre-existing test_closure_capture_with_array_filter failure is OK)
- Measurable increase in detected type errors

## Ready for Merge
Yes - Implementation complete and tested.

## Changes Summary
- **Files modified**: 1 (`wasm/src/thin_checker.rs`)
- **Lines changed**: 3 locations
- **Change**: Contextual type fallbacks from `TypeId::ANY` → `TypeId::UNKNOWN`
- **Commit**: `31a2ae625 Complete: Reduce 'Any' fallback in contextual type patterns`
- **Pushed to**: origin/worker-12
