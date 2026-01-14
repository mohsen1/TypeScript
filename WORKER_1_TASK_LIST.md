# Worker 1 Task List

## Squad: Parser (Syntax) - TS1005 Focus

## Current Task
- [ ] Test and verify TS1005 fix impact - run conformance to measure false positive reduction

## Queue
- [ ] Continue fixing remaining TS1005 false positive patterns (target: <100 total)
- [ ] Fix property semicolon handling edge cases in object literals
- [ ] Fix arrow function return type annotation edge cases
- [ ] Coordinate with Workers 2 & 3 on remaining parser false positives

## Completed
- [x] Audit TS1005 ("expected X") emission patterns - identified specific parser locations emitting false positives
- [x] Fix parseSemicolonAfterPropertyName to consolidate error emission
- [x] Fix shouldParseReturnType to remove premature TS1005 emission
- [x] Implement fixes to reduce TS1005 false positive emissions

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

### Changes Made
- `src/compiler/parser.ts`:
  - Updated `parseSemicolonAfterPropertyName()` to avoid premature error emission
  - Updated `shouldParseReturnType()` to skip TS1005 for => vs : confusion
  - 15 insertions, 11 deletions

### Goal
Reduce TS1005 errors from 439 to <100 through iterative fixes.
