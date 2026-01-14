# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [ ] Reduce "Any" fallback in type parameter defaults only

## Context

**Previous Attempt Status: REJECTED**
The previous attempt to change ALL fallbacks from `Any` to `Unknown` was too aggressive and caused issues.

**New Approach: Incremental Strategy**
Instead of changing everything at once, we'll make focused, testable changes one area at a time.

**Why Type Parameter Defaults?**
Type parameter defaults are a critical source of "Any" poisoning:
- When generic functions are called without explicit type arguments
- The compiler fills in missing type parameters from `default` or `constraint`
- If neither exists, it currently falls back to `Any`
- This `Any` then propagates through the entire call chain

**Targeted Change**
Only change the fallback in these specific locations in `thin_checker.rs`:
1. Line ~1706: `param.default.or(param.constraint).unwrap_or(TypeId::ANY)` → `unwrap_or(TypeId::UNKNOWN)`
2. Line ~3575: Same pattern (interface merging)
3. Line ~4520: Same pattern
4. Line ~4603: Same pattern
5. Line ~5118: Same pattern

This is a surgical change (5 locations) that should:
- Expose bugs in generic type handling
- Not affect other parts of the codebase
- Be easy to test and measure

## Queue
- [ ] After type parameter fix, measure conformance impact
- [ ] If successful, tackle other fallback patterns incrementally
- [ ] Coordinate with Solver Squad on type inference improvements

## Implementation Steps

1. **Make targeted changes**
   - Only modify type parameter default fallbacks in `thin_checker.rs`
   - Search for `param.default.or(param.constraint).unwrap_or(TypeId::ANY)`
   - Replace with `.unwrap_or(TypeId::UNKNOWN)`

2. **Test incrementally**
   - Run `./wasm/test.sh` after each change
   - Run small conformance sample: `./wasm/differential-test/run-conformance.sh --max=50`
   - Verify no crashes

3. **Measure impact**
   - Compare error counts before/after
   - Focus on TS2322 (type mismatch) and TS7006 (implicit any)
   - Document which tests are affected

## Files to Modify
- `wasm/src/thin_checker.rs` - Only type parameter default fallbacks (5 locations)

## Success Criteria
- Code compiles without errors
- No test crashes
- Measurable increase in detected type errors (TS2322, TS7006)

## Recent Merge Status
- **Date**: 2026-01-14
- **Result**: Successfully merged into em-team-3
- **Action Taken**:
  - Synced em-team-3 with rust (fast-forward)
  - Merged worker-12 with --no-ff
  - Build verification: PASSED
- **Next**: Continue incremental Any→Unknown changes in type parameter defaults

## Ready for Merge
Yes (task list update merged)
