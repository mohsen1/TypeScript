# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [ ] Reduce "Any" fallback in this-type patterns

## Context

**Previous Work Completed**
Worker 12 has been incrementally reducing "Any" fallback usage:
1. ✅ Type parameter defaults (7 locations)
2. ✅ Property access fallbacks (5 locations)
3. ✅ Contextual type fallbacks (3 locations)

All changes merged to rust.

**New Focus: This-Type Fallbacks**
This-type is used in class methods to refer to the instance type:
- When a method uses `this` and the type needs to be inferred
- The checker resolves `this` to the class instance type
- When this-type cannot be determined, it currently falls back to `Any`

**Targeted Changes**
Change this-type fallback in `thin_checker.rs`:

1. Line ~629: `self.current_this_type().unwrap_or(TypeId::ANY)` → `unwrap_or(TypeId::UNKNOWN)`

This is in:
- Primary expression type checking (this keyword resolution)

**Why This Matters**
When `this` is used in a context where its type cannot be determined:
- **Current**: Returns `Any` (loses all type safety for `this`)
- **New**: Returns `Unknown` (maintains strictness, emits errors for unsafe operations)

This will expose bugs in:
- Methods without proper class binding
- Arrow functions using `this` incorrectly
- Nested function scopes losing `this` context

Note: This is a single-location change but with high impact. The `this` keyword
is fundamental to class methods, and ensuring its type is correctly resolved
is critical for type safety.

## Queue
- [ ] After this-type fix, measure conformance impact
- [ ] Tackle array element type fallbacks
- [ ] Tackle remaining accessor fallbacks
- [ ] Coordinate with Solver Squad on type inference improvements

## Implementation Steps

1. **Make targeted change**
   - Change this-type fallback from `TypeId::ANY` to `TypeId::UNKNOWN`
   - Location: Line ~629 in thin_checker.rs
   - Pattern: `self.current_this_type().unwrap_or(TypeId::ANY)`

2. **Test incrementally**
   - Run `./wasm/test.sh` after change
   - Verify compilation succeeds
   - Check for new test failures

3. **Document findings**
   - Note which error codes increase
   - Identify patterns in exposed bugs (especially class-related)

## Files to Modify
- `wasm/src/thin_checker.rs` - This-type fallback (1 location)

## Success Criteria
- Code compiles without errors
- No critical test crashes (pre-existing test_closure_capture_with_array_filter failure is OK)
- Measurable increase in detected type errors related to `this`

## Ready for Merge
No (task in progress)
