# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [x] Reduce "Any" fallback in contextual type patterns (COMPLETE)

## Completed
- [x] Phase 1: Type parameter defaults (7 locations)
- [x] Phase 2: Property access patterns (5 locations)
- [x] Phase 3: Contextual type patterns (3 locations)
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

### Phase 2: Property Access Patterns (MERGED)
Changed 5 locations in `thin_checker.rs`:
- Lines 8470, 8554, 8758, 8981, 9361
- `property_type.unwrap_or(TypeId::ANY)` → `.unwrap_or(TypeId::UNKNOWN)`

**Impact**: Exposes bugs where properties are accessed without type checking.

### Phase 3: Contextual Type Patterns (JUST COMPLETED)
Changed 3 locations in `thin_checker.rs`:
1. Line ~9667: `helper.get_this_type().unwrap_or(TypeId::ANY)` → `unwrap_or(TypeId::UNKNOWN)`
2. Line ~9673: `contextual_type.unwrap_or(TypeId::ANY)` → `unwrap_or(TypeId::UNKNOWN)`
3. Line ~15473: `self.current_return_type().unwrap_or(TypeId::ANY)` → `unwrap_or(TypeId::UNKNOWN)`

**Impact**: Exposes bugs in callback contexts, return type inference, and this-type inference.

## Queue
- [ ] After merge, measure conformance impact
- [ ] Tackle this-type fallbacks
- [ ] Tackle array element type fallbacks
- [ ] Coordinate with Solver Squad on type inference improvements

## Files Modified
- `wasm/src/thin_checker.rs` - 15 total locations changed (7 type params + 5 property access + 3 contextual)

## Success Criteria
- ✅ Code compiles without errors
- ✅ No new test crashes (one pre-existing failure unrelated)
- ⏳ Measurable increase in detected type errors (to be verified in conformance)

## Merge Status
- **Date**: 2026-01-14
- **Phase 3 Commit**: `31a2ae625 Complete: Reduce 'Any' fallback in contextual type patterns`
- **Status**: Ready for merge to em-team-3
