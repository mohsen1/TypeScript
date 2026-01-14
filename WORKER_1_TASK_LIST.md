# Worker 1 Task List

## Squad: Parser (Syntax) - TS1005 Focus

## Current Task
- [ ] Test and verify all TS1005 fixes impact - run conformance to measure false positive reduction

## Queue
- [ ] Continue fixing remaining TS1005 false positive patterns (target: <100 total)
- [ ] Fix object literal comma handling edge cases
- [ ] Fix array literal missing element handling
- [ ] Fix type parameter parsing edge cases
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
