# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [x] Reduce "Any" fallback in property access patterns (COMPLETE)

## Completed
- [x] Reduce "Any" fallback in type parameter defaults (7 locations)
- [x] Reduce "Any" fallback in property access patterns (5 locations)
- [x] Verify compilation - Code compiles successfully
- [x] Test - One pre-existing failure (unrelated to changes)

## Context

**Incremental Strategy**
Worker 12 has been systematically reducing "Any" fallback usage to expose hidden bugs.

### Phase 1: Type Parameter Defaults (MERGED)
Changed 7 locations in `thin_checker.rs` from `TypeId::ANY` to `TypeId::UNKNOWN`:
1. Constructor signature instantiation
2. Interface merging (base type params)
3. Interface type params
4. Type param handling/fallback

**Impact**: Exposes bugs in generic function/method calls without explicit type args.

### Phase 2: Property Access Patterns (JUST COMPLETED)
Changed 5 locations in `thin_checker.rs`:
- Lines 8470, 8554, 8758, 8981, 9361
- `property_type.unwrap_or(TypeId::ANY)` → `.unwrap_or(TypeId::UNKNOWN)`

**Impact**: Exposes bugs where:
- Properties are accessed without type checking
- Optional chaining isn't used where it should be
- Type assertions are missing

## Queue
- [ ] After merge, measure conformance impact
- [ ] Tackle contextual type fallbacks
- [ ] Tackle this-type fallbacks
- [ ] Coordinate with Solver Squad on type inference improvements

## Files Modified
- `wasm/src/thin_checker.rs` - 12 total locations changed (7 type params + 5 property access)

## Success Criteria
- ✅ Code compiles without errors
- ✅ No new test crashes (one pre-existing failure unrelated)
- ⏳ Measurable increase in detected type errors (to be verified in conformance)

## Merge Status
- **Date**: 2026-01-14
- **Phase 2 Commit**: `6ba66b274 Complete: Reduce 'Any' fallback in property access patterns`
- **Status**: Ready for merge to em-team-3
