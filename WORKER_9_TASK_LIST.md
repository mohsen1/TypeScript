# Worker 9 Task List

## Squad: Parser/Scanner - TS1005 Focus (Part 2)

## Current Task
- [ ] Audit and fix TS1005 false positives in object literal parsing (parseObjectLiteralElement)
- [ ] Focus on comma handling and property name edge cases

## Queue
- [ ] Fix TS1005 in array literal parsing - missing element false positives
- [ ] Fix TS1005 in type parameter parsing - angle bracket edge cases
- [ ] Fix TS1005 in return type parsing - arrow function confusion
- [ ] Fix TS1005 in statement parsing - semicolon insertion edge cases
- [ ] Run conformance tests and measure TS1005 reduction
- [ ] Coordinate with Worker 10 to avoid duplicate work

## Completed
- [x] Branch created from em-team-3
- [x] Reviewed TS1005_REDUCTION_RESULTS.md for patterns already fixed
- [x] Synced with em-team-3 (no new commits to merge)
- [x] Reconfiguration check: Worker 9 remains on Parser squad (TS1005 focus)

## Recent Merge Status
- **Date**: 2026-01-14 (second verification)
- **Result**: Worker-9 already fully merged into em-team-3
- **Action Taken**:
  - Rebased em-team-3 on rust (successful)
  - Verified all worker-9 commits present in em-team-3
  - Build verification: PASSED
- **Next**: Worker 9 to continue TS1005 patterns 6-10
- **Team Update**: Worker 12 re-added to EM-3 (now 4 workers: 9-12)

## Context
TS1005 ("expected X") is the #1 source of parser false positives (439 occurrences). Workers 1-5 have already fixed patterns 1-5. Your focus is on remaining patterns.

### Patterns to Fix (Your Scope)

**Pattern 6 - Object literal comma handling:**
- `parseObjectLiteralElement()` may emit TS1005 on valid comma-separated properties
- Check error recovery when comma is missing but ASI applies
- Location: `src/compiler/parser.ts`

**Pattern 7 - Array literal element parsing:**
- Array literals with missing elements trigger false TS1005
- Check `parseArrayLiteralElement()` for over-aggressive error emission
- Location: `src/compiler/parser.ts`

**Pattern 8 - Type parameter parsing:**
- Generic type parameters with defaults may trigger TS1005 incorrectly
- Check `parseTypeParameter()` for edge cases
- Location: `src/compiler/parser.ts`

**Pattern 9 - Return type vs arrow confusion:**
- Similar to Pattern 3 but may have other instances
- `shouldParseReturnType()` already fixed, check for similar code paths
- Location: `src/compiler/parser.ts`

### Success Metric
Reduce TS1005 from 439 to <100 total.

### Workflow
1. Create branch: `git checkout -b worker-9-TS1005-patterns`
2. Fix identified patterns
3. Run tests: `npm run test:conformance`
4. Push: `git push origin worker-9-TS1005-patterns`
5. Notify EM-3 for merge
