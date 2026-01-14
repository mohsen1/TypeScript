# Worker 2 Task List

## Squad: Parser (Syntax)

## Conformance Test Results (2026-01-14)

### TS1109 Reduction
- **Before:** 262 errors
- **After:** 198 errors
- **Reduction:** 64 errors (-24%) ✅

### Overall Metrics Impact
| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Exact Match | 30.1% | 30.5% | +0.4% ✅ |
| Missing Errors | 60.0% | 59.7% | -0.3% ✅ |
| Extra Errors | 30.9% | 29.8% | -1.1% ✅ |
| Parser False Positives | 701 | 637 | -64 ✅ |

**Combined with Worker 1:** Total parser FP would be 510 (TS1005: 312 + TS1109: 198)

### Validation
✅ No regressions in other error codes
✅ Build passes
⚠️ TS1109 still above target (need <100, currently 198)
⚠️ Modest impact compared to Worker 1 (24% vs 29%)

### Patterns Fixed (64 cases total)
1. Variable declarations: `var t!: T;` (~28 cases)
2. Object shorthand properties: `{ a! }` (~18 cases)
3. Computed property type parameters (~12 cases)
4. Array destructuring with annotations (~6 cases)

### Remaining TS1109 Patterns
1. **new.target context validation** (~52 cases) - HIGH IMPACT
2. Cascading from TS1005 (~35 cases) - Worker 3's fix will help
3. Template strings in type positions (~14 cases)
4. Private names in `in` expressions (~9 cases)
5. Destructuring edge cases (~8 cases)
6. Miscellaneous edge cases (~82 cases)

**Estimated with Worker 3's fix:** ~158-168 remaining (vs 262 baseline)

## Current Task (Assigned by EM-1)
- [ ] **Fix new.target context validation** (~52 cases) - HIGH IMPACT
- Context: `new.target` should only be valid in function bodies and constructors
- Add TS17013 error when used outside valid contexts
- Implement context tracking for function/constructor bodies

## Queue (On Hold - Awaiting Baseline)
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
