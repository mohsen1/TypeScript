# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [x] Reduce "Any" fallback in type parameter defaults only

## Completed
- [x] Locate type parameter default fallback locations (found 7)
- [x] Change all 7 locations from TypeId::ANY to TypeId::UNKNOWN
- [x] Verify compilation - Code compiles successfully
- [x] Test - One pre-existing failure (unrelated to changes)

## Context

**Previous Attempt Status: REJECTED**
The previous attempt to change ALL fallbacks from `Any` to `Unknown` was too aggressive and caused issues.

**New Approach: Incremental Strategy**
Instead of changing everything at once, we made a focused, testable change.

**Why Type Parameter Defaults?**
Type parameter defaults are a critical source of "Any" poisoning:
- When generic functions are called without explicit type arguments
- The compiler fills in missing type parameters from `default` or `constraint`
- If neither exists, it previously fell back to `Any`
- This `Any` then propagates through the entire call chain

**Changes Made**
Modified 7 locations in `thin_checker.rs`:
1. Line 1706: Constructor signature instantiation
2. Line 3575: Interface merging (base type params)
3. Line 4520: Interface merging
4. Line 4603: Interface type params
5. Line 5118: Interface merging
6. Line 18599: Type param handling
7. Line 18933: Type param fallback

**Impact**
- Exposes bugs in generic function/method calls without explicit type args
- Does NOT affect other fallback patterns (property access, etc.)
- Surgical change - should be easy to measure and verify

## Queue
- [ ] After merge, measure conformance impact
- [ ] If successful, tackle other fallback patterns incrementally
- [ ] Coordinate with Solver Squad on type inference improvements

## Files Modified
- `wasm/src/thin_checker.rs` - Changed 7 type parameter fallbacks

## Success Criteria
- ✅ Code compiles without errors
- ✅ No new test crashes (one pre-existing failure unrelated)
- ⏳ Measurable increase in detected type errors (to be verified in conformance)

## Ready for Merge
Yes - Implementation complete and tested.
