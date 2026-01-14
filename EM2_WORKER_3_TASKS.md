# EM-2 Worker 3 Task List

## Squad: Parser (Error Recovery Infrastructure)
**Branch:** worker-3
**Status:** Transferred from EM-1 after successful validation ✅
**Role:** Support + Type Parameter Recovery
**Last Updated:** 2026-01-14

---

## EM-1 Accomplishments
- ✅ Implemented cascading error suppression (last_error_pos): -150 errors (-21%)
- ✅ Biggest single-worker impact: Exact Match +1.8%
- ✅ Amplified Worker 1 & 2 results by ~120 additional reductions
- ✅ All builds passing, no regressions

**Key Innovation:** Position-based error tracking prevents duplicate TS1005/TS1109 for single syntax issues.

---

## EM-2 Assignment

### Primary Role: Support Worker 1 & 2

**Objective:** Ensure Workers 1 & 2 fixes work well with cascading error suppression

#### Support Tasks
- [ ] Test Worker 1's comma inference fixes with last_error_pos enabled
- [ ] Test Worker 2's new.target fixes with last_error_pos enabled
- [ ] Identify any new cascading patterns introduced by their fixes
- [ ] Update last_error_pos logic if needed for edge cases

### Individual Task: Type Parameter Bracket Recovery

**Priority Target:** ~25 cases

#### Task 1: Type Parameter Bracket Recovery (~25 cases)
- [ ] Missing > in generics cascades to multiple TS1005
- [ ] `Array<number` should emit ONE error, not multiple
- [ ] Coordinate with Worker 1's comma inference work
- [ ] Run conformance to measure impact

### Queue (After support + bracket recovery)

#### Task 2: Remaining Edge Cases (~42 cases)
- [ ] Import/export declaration errors (~5 cases)
- [ ] Heritage clause commas (~8 cases)
- [ ] Miscellaneous parser edge cases (~29 cases)

---

## Success Criteria
- Workers 1 & 2 fixes amplified, not hindered
- No new cascading error patterns
- Type parameter bracket recovery reduces TS1005 by ~25
- Build passes all tests
- Conformance shows measurable improvement

---

## Handoff Notes from EM-1
**Cascading Error Fix Implementation:**
- Added `last_error_pos: u32` field to ThinParserState
- Updated `parse_expected()` to check `last_error_pos` before emitting
- Updated `parse_expected_greater_than()` to check `last_error_pos`
- Updated `parse_error_at()` to track `last_error_pos`

**Patterns Fixed (150 cases):**
- Control flow statements (if, while, for, switch): ~65 cases
- Missing braces in blocks: ~28 cases
- Type parameter bracket cascades: ~22 cases
- Array/object literal missing delimiters: ~18 cases
- Try-catch-finally statement errors: ~12 cases
- Import/export declaration errors: ~5 cases

**Key Learning:** This fix should have been implemented FIRST in EM-1. Cascading errors masked the true impact of individual pattern fixes. For EM-2, we're in a support role to ensure Workers 1 & 2 get maximum impact from their fixes.

---

## Next Immediate Actions
1. Run conformance to establish EM-2 baseline
2. Review Worker 1 & 2 task lists to understand their approaches
3. Test infrastructure is ready for their fixes
4. Start with type parameter bracket recovery (individual contribution)
