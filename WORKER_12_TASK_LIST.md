# Worker 12 Task List

## Squad: Solver Strictness

## Current Task
- [ ] Reduce "Any" fallback in this-type patterns (Phase 4 - IN PROGRESS)

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
Changed 7 locations in `thin_checker.rs` from `TypeId::ANY` to `TypeId::UNKNOWN`.

### Phase 2: Property Access Patterns (MERGED)
Changed 5 locations in `thin_checker.rs`.

### Phase 3: Contextual Type Patterns (MERGED)
Changed 3 locations in `thin_checker.rs`.

### Phase 4: This-Type Patterns (IN PROGRESS)
Target: Line ~629 in thin_checker.rs
- `self.current_this_type().unwrap_or(TypeId::ANY)` → `unwrap_or(TypeId::UNKNOWN)`

**Impact**: Exposes bugs in class methods, arrow functions using `this`, and nested scopes.

## Queue
- [ ] After this-type fix, measure conformance impact
- [ ] Tackle array element type fallbacks
- [ ] Tackle remaining accessor fallbacks
- [ ] Coordinate with Solver Squad on type inference improvements

## Files Modified
- `wasm/src/thin_checker.rs` - 15 total locations changed (phases 1-3), phase 4 in progress

## Success Criteria
- ✅ Code compiles without errors
- ✅ No new test crashes (one pre-existing failure unrelated)
- ⏳ Measurable increase in detected type errors (to be verified in conformance)

## Merge Status
- **Date**: 2026-01-14
- **Phases 1-3**: Merged to em-team-3
- **Phase 4**: Task assigned, in progress
