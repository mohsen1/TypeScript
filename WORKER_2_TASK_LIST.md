# Worker 2 Task List

## Squad: Parser (Syntax)

## Current Task (Assigned by EM-1)
- [ ] **IMMEDIATE:** Run conformance tests to measure TS1109 baseline after definite assignment assertion fix
- [ ] Document exact TS1109 count before and after your fixes
- [ ] Report metrics to EM-1 for validation before proceeding

## Queue (On Hold - Awaiting Baseline)
- [ ] Fix new.target context validation (7 cases identified)
- [ ] Fix template strings in type positions (2 cases)
- [ ] Fix private names in `in` expressions (2 cases)
- [ ] Fix destructuring edge cases (3 cases)
- [ ] Add position deduplication for TS1005 → TS1109 cascades
- [ ] Run full conformance test suite and document final results

## Completed
- [x] Audit TS1109 emission points - created TS1109_ANALYSIS.md (4 emission points identified, Line 6312 is main source)
- [x] Create list of specific test cases where TS1109 fires incorrectly
- [x] Fix definite assignment assertions in variable declarations (`var x!: string;`)
- [x] Fix definite assignment assertions in object shorthand properties (`{ a! }`)

## Context

### Previous Analysis
TS1109 has 262 false positives. These parser errors mask real progress. Focus on expression parsing edge cases.

### Emission Points
1. **Line 6312** - `parse_primary_expression` (MAIN SOURCE)
2. Line 1316 - Variable declaration initializer
3. Line 7499 - New expression type assertion
4. Line 9608 - JSX attribute value

### Key Patterns Identified (from 34 test cases analyzed)
- **Incomplete Expression (31 cases, 91%)**:
  - Type parameters in computed properties (4 cases) - FIXED by definite assignment assertions
  - `new.target` context issues (7 cases)
  - Private name syntax (2 cases)
  - Template strings in type positions (2 cases)
  - Destructuring edge cases (3 cases)
  - And more...
- **Cascaded from TS1005 (3 cases, 9%)**:
  - Error recovery cascades

### Recent Fixes
The definite assignment assertion fix resolved:
- Variable declarations: `var t!: T;`
- Object shorthand: `{ a! }`
- This also fixed the computed property cases that were using type parameters

## Next Steps

### Immediate
1. **Baseline measurement**: Run conformance tests to see how many TS1109 cases remain after the definite assignment fix
2. **Prioritize remaining patterns** by impact and complexity

### High Impact Targets (22+ cases)
1. `new.target` validation (7 cases) - Parser needs better context tracking
2. Position deduplication for TS1005 cascades (3+ cases)
3. Template strings in type positions (2 cases)
4. Private names in `in` expressions (2 cases)
5. Destructuring edge cases (3 cases)
6. Remaining edge cases (5+ cases)

## Success Criteria
- Reduce TS1109 false positives from 262 to <100
- Fix all identified patterns with 3+ occurrences
- Document final results
- No regressions in other error codes
