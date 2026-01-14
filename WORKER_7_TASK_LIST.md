# Worker 7 Task List

## Squad: Parser/Scanner - Error Recovery Focus

## Current Task (Phase 6)
- [ ] Implement binary expression right-side error recovery
- [ ] Add resynchronization after failed right-side expression parsing
- [ ] Test and measure impact on false positive reduction

## Queue
- [ ] Investigate TS1005 patterns in WASM parser (coordinate with Workers 5/6)
- [ ] Implement sequence expression recovery (Priority 2)
- [ ] Parser error analysis for remaining optimization opportunities

## Completed ✅

### Phase 1: TS1109 Cascading Error Fix
**Status:** ✅ Complete (93% reduction!)
- TS1109: 262 → 17 occurrences (93% reduction)
- Goal of <50 achieved ✓

### Phase 2-3: Comprehensive Position Deduplication
**Status:** ✅ Complete
- Extended position deduplication to all major parser error types
- All 7 major error types now have position deduplication

### Phase 4: Expression-Level Error Recovery Analysis
**Status:** ✅ Complete
- Created `EXPRESSION_LEVEL_ERROR_RECOVERY_ANALYSIS.md`
- Identified Priority 1: Binary expression right-side recovery

### Phase 5: Remaining Error Type Analysis
**Status:** ✅ Complete (merged: `d915c88ff`)
- Analyzed remaining parser error types beyond TS1005/TS1109
- Conclusion: No additional position deduplication needed

### Merge Status
- [x] All phases 1-5 integrated to em-team-2
- [x] Documentation preserved: POSITION_DEDUPLICATION_STRATEGY.md, TS1005_ANALYSIS.md, EXPRESSION_LEVEL_ERROR_RECOVERY_ANALYSIS.md, REMAINING_PARSER_ERROR_TYPES_ANALYSIS.md
- [x] **EM-2 Latest Merge (2026-01-14):** Phase 6 assignment merged
  - Commit: 0fc85f52f - Binary expression recovery assignment
  - Rebased em-team-2 onto rust (10 commits, clean)
  - Status: IN PROGRESS - Working on Phase 6 implementation

---

## Context
Worker 7 has completed comprehensive position deduplication (Phases 1-3) and analysis of error recovery opportunities (Phases 4-5).

With EM-1 achieving 99.9% TS1005 elimination and position deduplication complete, Phase 6 focuses on implementing the Priority 1 improvement identified in the expression-level analysis: **binary expression right-side error recovery**.

### Phase 6 Implementation Plan

**Objective:** Add resynchronization to `parse_binary_expression()` when right-side expression parsing fails.

**Current Behavior:**
```rust
// When right-side parsing fails in: x = a + * b + c
// Parser emits error at *, then tries to parse rest
// May emit cascading errors for '+ c'
```

**Proposed Solution:**
Add resynchronization check after failed right-side parse:
```rust
let right = if self.can_follow_operator() {
    self.parse_binary_expression(next_min)
} else {
    // Recovery: Skip to next operator or end-of-expression marker
    self.resync_to_next_binary_operator();
    NodeIndex::NONE
};
```

**Expected Impact:**
- Prevents cascading errors in complex expressions with invalid syntax
- Particularly helpful for: `a + * b + c` type scenarios
- Reduces noise when binary expression parsing fails mid-expression

### Key Files
- `wasm/src/thin_parser.rs` - main parser implementation (target for Phase 6)
- `EXPRESSION_LEVEL_ERROR_RECOVERY_ANALYSIS.md` - detailed analysis from Phase 4

### Goal
Implement expression-level error recovery to further reduce parser false positives beyond the comprehensive position deduplication already achieved.

---

## Next Focus
Implement binary expression right-side recovery (Priority 1 from expression-level analysis) to prevent cascading errors in complex expressions.
