# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [ ] Reduce "Any" fallback in array element type patterns

## Context

**Previous Work Completed**
Worker 12 has been incrementally reducing "Any" fallback usage:
1. ✅ Type parameter defaults (7 locations)
2. ✅ Property access fallbacks (5 locations)
3. ✅ Contextual type fallbacks (3 locations)
4. ✅ This-type fallback (1 location)

All changes merged to rust. Total: 16 locations changed.

**New Focus: Array Element Type Fallbacks**
Array types like `Array<T>` or `T[]` need to know their element type:
- When creating or inferring array types
- The checker extracts the element type for type checking
- When the element type is unknown, it currently falls back to `Any`

**Targeted Changes**
Change array element type fallbacks in `thin_checker.rs`:

1. Line ~965: Array type element type inference
2. Line ~2583: Array type element type in type literals
3. Line ~2657: Additional array element type handling

Pattern: `.unwrap_or(TypeId::ANY)` after array type construction

**Why This Matters**
When array element type cannot be determined:
- **Current**: Returns `Any` (array becomes `any[]`, loses all type safety)
- **New**: Returns `Unknown` (array becomes `unknown[]`, maintains strictness)

This will expose bugs in:
- Arrays without explicit type parameters
- Array literals with inconsistent element types
- Generic array operations where element type is lost

Note: Arrays are a fundamental data structure. Ensuring element type is
correctly resolved is critical for collection type safety.

## Queue
- [ ] After array element fix, measure conformance impact
- [ ] Tackle remaining accessor fallbacks
- [ ] Tackle argument type fallbacks
- [ ] Coordinate with Solver Squad on type inference improvements

## Implementation Steps

1. **Make targeted changes**
   - Change array element type fallbacks from `TypeId::ANY` to `TypeId::UNKNOWN`
   - Focus on: array literal type construction
   - Found ~3 locations in thin_checker.rs

2. **Test incrementally**
   - Run `./wasm/test.sh` after changes
   - Verify compilation succeeds
   - Check for new test failures

3. **Document findings**
   - Note which error codes increase
   - Identify patterns in exposed bugs (especially array-related)

## Files to Modify
- `wasm/src/thin_checker.rs` - Array element type fallbacks (~3 locations)

## Success Criteria
- Code compiles without errors
- No critical test crashes (pre-existing test_closure_capture_with_array_filter failure is OK)
- Measurable increase in detected type errors related to arrays

## Ready for Merge
No (task in progress)
