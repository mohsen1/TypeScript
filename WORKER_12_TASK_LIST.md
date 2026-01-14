# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [ ] Reduce "Any" fallback in property access patterns

## Context

**Previous Work**
Worker 12 has been incrementally reducing "Any" fallback usage to expose hidden bugs. The type parameter defaults change (7 locations) was implemented but not yet merged to rust.

**New Focus: Property Access Fallbacks**
Property access is a critical source of "Any" poisoning:
- When accessing `obj.property` where the property type is unknown
- The checker returns `Any` instead of `Unknown`
- This `Any` then propagates through all subsequent operations on that property

**Targeted Changes**
Change property access fallbacks in `thin_checker.rs`:

1. Lines 8470, 8554, 8758, 8981, 9361: `property_type.unwrap_or(TypeId::ANY)` → `unwrap_or(TypeId::UNKNOWN)`

These are in:
- Property type resolution for spread operators
- Property access expressions
- Union type construction
- Object literal property types

**Why This Matters**
When you access `obj.foo` and `foo` doesn't exist or has no type:
- Current: Returns `Any` (silently accepts invalid code)
- New: Returns `Unknown` (will emit errors for unsafe operations)

This will expose bugs where:
- Properties are accessed without type checking
- Optional chaining isn't used where it should be
- Type assertions are missing

## Queue
- [ ] After property access fix, measure conformance impact
- [ ] Tackle contextual type fallbacks
- [ ] Tackle this-type fallbacks
- [ ] Coordinate with Solver Squad on type inference improvements

## Implementation Steps

1. **Make targeted changes**
   - Change property access fallbacks from `TypeId::ANY` to `TypeId::UNKNOWN`
   - Search pattern: `property_type.unwrap_or(TypeId::ANY)`
   - Found 5 locations in thin_checker.rs

2. **Test incrementally**
   - Run `./wasm/test.sh` after changes
   - Verify compilation succeeds
   - Check for new test failures

3. **Document findings**
   - Note which error codes increase
   - Identify patterns in exposed bugs

## Files to Modify
- `wasm/src/thin_checker.rs` - Property access fallbacks (5 locations)

## Success Criteria
- Code compiles without errors
- No critical test crashes (pre-existing test_closure_capture_with_array_filter failure is OK)
- Measurable increase in detected type errors

## Ready for Merge
No (task in progress)
