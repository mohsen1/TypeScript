# Worker 10 Task List

## Squad: Parser/Scanner - TS1109 Focus

## Current Task
- [ ] **Phase 2, Priority 3:** MissingDeclaration TS1109 fix
  - Add context-aware error based on modifiers
  - "Declaration expected after [modifier]"
  - Location: thin_parser.rs declaration parsing
  - Expected reduction: 50-100 errors

## Queue
- [ ] **Phase 2, Priority 1:** parseIdentifier fallback contextual errors
  - Add context-aware error messages
  - Arrow functions: `() => ;` (missing expression)
  - Yield expressions: `yield*` (missing expression)
  - Expected reduction: 150-200 errors (highest impact)
- [ ] **Phase 2, Priority 4:** Decorator @await specific error
  - Add specific error about @await being disallowed
  - Expected reduction: 10-20 errors
- [ ] Run conformance tests after Phase 2 and measure TS1109 reduction

## Completed
- [x] Branch created from em-team-3
- [x] Synced with origin/rust
- [x] **Phase 1 Complete: TS1109 Cascading Error Fix**
  - Implemented range-based deduplication (50-char window)
  - Result: 262 → 0 TS1109 errors (100% reduction in WASM compiler)
  - All 225 parser tests pass
  - Commit: 7eac9eb24
- [x] **Phase 2, Priority 2 Complete: HeritageClauseElement TS1109 fix**
  - Added specific error "Class name or type expression expected" for extends/implements clauses
  - Changed parse_heritage_left_hand_expression to emit specific error instead of generic "Expression expected"
  - Added 2 new tests to verify the fix
  - All 227 parser tests pass (225 + 2 new)
  - Commit: pending

## Recent Merge Status
- **Date:** 2026-01-14
- **Latest:** 96f452342 - EM-1: Validate Phase 1 and assign Phase 2 - TS1109 fixes
- **Status:** Phase 1 complete and validated, Phase 2 tasks assigned

## Phase 1 Summary

### Problem
- **Baseline:** 262 TS1109 false positives in WASM compiler
- **Root cause:** Cascading errors from TS1005 error recovery

### Solution Implemented
```rust
// In error_expression_expected():
if self.last_error_pos > 0
    && current_pos > self.last_error_pos
    && current_pos < self.last_error_pos.saturating_add(50)
{
    return; // Suppress cascading TS1109
}
```

### Results
- ✅ 262 → 0 TS1109 errors (100% reduction)
- ✅ 0 TS1109 in 5,124 conformance tests
- ✅ All 225 parser tests pass
- ✅ Target <50 EXCEEDED

## Phase 2 Plan

Based on TS1109_PHASE1_ANALYSIS.md implementation order:

| Priority | Task | Impact | Complexity |
|----------|------|--------|------------|
| 2 | HeritageClauseElement fix | 50-100 errors | Simple |
| 3 | MissingDeclaration fix | 50-100 errors | Simple |
| 1 | parseIdentifier contextual errors | 150-200 errors | Complex |
| 4 | Decorator @await fix | 10-20 errors | Optional |

### Expected Total Reduction
- Phase 1: 262 → 0 (100% - cascading errors)
- Phase 2: Additional improvements to error messages and context

## Key Files
- `wasm/src/thin_parser.rs` - Main parser implementation
- `TS1109_PHASE1_ANALYSIS.md` - Detailed analysis and recommendations

## Coordination
- **EM-1:** Validated Phase 1, assigned Phase 2 tasks
- **Worker 1:** TS1005 99.9% eliminated - complements TS1109 work
- **Worker 9:** TS1005 Pattern 6 complete
