# Worker 11 Task List

## Squad: Parser/Scanner - TS1005 Focus (Part 3)

**Note**: Reassigned from Binder squad to Parser squad per EM-3 reconfiguration.

## Current Task
- [ ] Analyze and fix remaining parser error types with position deduplication
- [ ] Focus on error types beyond TS1005/TS1109 that contribute to false positive count

## Queue
- [ ] Implement position deduplication for TS1012 (Unexpected Token) in loop contexts
- [ ] Audit and fix TS1044 (Modifiers Not Allowed Here) cascading errors
- [ ] Review TS2568 (Property Assignment Expected) for false positives
- [ ] Cross-squad validation: Verify EM-1/EM-2 TS1005 fixes in WASM parser
- [ ] Run conformance tests and measure overall parser error reduction

## Completed
- [x] Branch created from em-team-3
- [x] Reviewed TS1005_REDUCTION_RESULTS.md for patterns already fixed
- [x] **TS2304 global scope binding fix** (Merged: eba0e94b6)
  - Implemented chained lookup in `ThinBinderState::resolve_identifier`
  - Added lib_binders check for resolving console, Array, Object, Promise, etc.
  - All lib_loader tests pass
- [x] **TS1005 WASM Parser Analysis - All patterns 11-15** (Completed)
  - Pattern 11 (Type parameters): Token splitting (>>, >>>) + position deduplication
  - Pattern 12 (Return types): Speculative parsing prevents diagnostic leakage
  - Pattern 13 (Statements): ASI correctly implemented
  - Pattern 14 (Class properties): Optional semicolons with parse_optional
  - Pattern 15 (Decorators): Non-greedy decorator parsing
  - **Finding: NO TS1005 false positives in WASM parser for patterns 11-15**
  - All 225 parser tests pass
  - No code changes needed
  - Document: `TS1005_PATTERNS_11-15_WASM_AUDIT.md`
- [x] **TS1109 Class Member Parsing Review** (Completed)
  - Position deduplication already implemented in error_expression_expected (line 377)
  - All TS1109 errors go through helper with position deduplication
  - Prevents cascading TS1109 errors when TS1005 already fired at same position
  - All 225 parser tests pass
  - No code changes needed
  - Document: `TS1109_CLASS_MEMBER_AUDIT.md`
- [x] **Coordination with Workers 9 & 10** (Completed - No duplicate work)
  - Worker 9: TS1005 patterns 6-10 (WASM analysis complete) - NO false positives
  - Worker 10: TS1109 eliminated 100% (262 → 0)
  - Worker 11: TS1005 patterns 11-15 + TS1109 class members (WASM audit complete) - NO false positives
  - Key Finding: **WASM parser is production-ready across all patterns 6-15**
- [x] Synced with em-team-3

## Recent Merge Status
- **Date**: 2026-01-14
- **Result**: Merged worker-11 with all TS1005/TS1109 analysis complete
- **Key Findings**:
  - TS1005 patterns 11-15: NO false positives in WASM parser
  - TS1109 class member parsing: Position deduplication already implemented
  - All 225 parser tests pass
  - WASM parser is production-ready
- **Next Task**: Analyze remaining parser error types (TS1012, TS1044, TS2568, etc.)
- **Files**:
  - TS1005_PATTERNS_11-15_WASM_AUDIT.md
  - TS1109_CLASS_MEMBER_AUDIT.md

## Context
TS1005 ("expected X") was the #1 source of parser false positives.

### WASM Parser Status
**All patterns 6-15 verified - NO false positives:**
- Patterns 6-10: Worker 9 verified
- Patterns 11-15: Worker 11 verified
- TS1109: Position deduplication implemented

**WASM parser is production-ready.**

### Next Phase: Remaining Parser Error Types
Based on REMAINING_PARSER_ERROR_TYPES_ANALYSIS.md (Worker 7):
- TS1012 (Unexpected Token): Loop contexts need position deduplication
- TS1044 (Modifiers Not Allowed Here): May have cascading errors
- TS2568 (Property Assignment Expected): Specific validation errors
