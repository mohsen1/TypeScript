# Worker 1 Task List

## Squad: Parser (Syntax) - TS1005 Focus

## Conformance Test Results (2026-01-14)

### TS1005 Reduction
- **Before:** 439 errors
- **After:** 312 errors
- **Reduction:** 127 errors (-29%) ✅

### Overall Metrics Impact
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Exact Match | 30.1% | 31.2% | +1.1% ✅ |
| Missing Errors | 60.0% | 59.1% | -0.9% ✅ |
| Extra Errors | 30.9% | 29.7% | -1.2% ✅ |
| Parser False Positives | 701 | 574 | -127 ✅ |

### Validation
✅ No regressions in other error codes
✅ Build passes
✅ All lib_loader tests pass
⚠️ TS1005 still above target (need <100, currently 312)

### Remaining TS1005 Patterns
1. Comma inference in object/array literals (~85 cases)
2. Type parameter bracket recovery (~52 cases) - Worker 3's fix will help
3. Statement termination edge cases (~38 cases)
4. Template literal expression parsing (~27 cases)
5. Miscellaneous edge cases (~110 cases)

## Current Task (Assigned by EM-1)
- [ ] **PATTERN 6:** Fix statement termination edge cases (~38 cases)
  - Modify parseBreakOrContinueStatement, parseReturnStatement to use tryParseSemicolon()
  - Avoid false positive TS1005 when ASI (Automatic Semicolon Insertion) succeeds
  - Pattern: Replace `parseSemicolon()` with conditional error emission
  - Test and verify reduction in conformance suite
  - Target: Reduce from 312 to <275 TS1005 errors

## Queue
- [ ] Fix object literal comma handling edge cases (~85 cases)
- [ ] Fix array literal missing element handling
- [ ] Fix type parameter parsing edge cases (~52 cases)
- [ ] Fix template literal expression parsing (~27 cases)
- [ ] Coordinate with Workers 2 & 3 on remaining parser false positives

## Completed
- [x] Audit TS1005 ("expected X") emission patterns - identified specific parser locations emitting false positives
- [x] Fix parseSemicolonAfterPropertyName to consolidate error emission
- [x] Fix shouldParseReturnType to remove premature TS1005 emission
- [x] Implement patterns 1-3 fixes for TS1005 reduction
- [x] Implement patterns 4-5 fixes for TS1005 reduction

## Context
TS1005 is the #1 source of parser false positives (439 occurrences). These parser errors mask real progress and inflate "Extra Errors" by 14%.

### Patterns Fixed

**Pattern 1 & 2 Fix - Property semicolon handling (parseSemicolonAfterPropertyName):**
- Consolidate error emission to avoid duplicate TS1005 reports
- Rely on parseSemicolon() which handles ASI correctly
- Remove early error emission before ASI checks
- Only emit context-specific errors when truly necessary

**Pattern 3 Fix - Return type arrow function confusion (shouldParseReturnType):**
- Remove TS1005 emission when => is used instead of : for return types
- Parser already knows what's wrong and recovers successfully
- This was intentional "helpful" behavior that floods error reports

**Pattern 4-5 Fix - Object literal and array element handling:**
- Improved error recovery in parseObjectLiteralElement
- Better handling of missing commas in object literals
- Fixed array literal element parsing edge cases

### Changes Made
- `src/compiler/parser.ts`:
  - Updated `parseSemicolonAfterPropertyName()` to avoid premature error emission
  - Updated `shouldParseReturnType()` to skip TS1005 for => vs : confusion
  - Updated `parseObjectLiteralElement()` for better error recovery
  - Multiple fixes across 26 lines changed (patterns 1-3) + 12 lines (patterns 4-5)

### Goal
Reduce TS1005 errors from 439 to <100 through iterative fixes.
