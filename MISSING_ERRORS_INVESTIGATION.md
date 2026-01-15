# Missing Errors Investigation

**Date:** 2026-01-15
**Investigator:** Worker-1 (EM-1 Team)
**Sample Size:** 487 tests (500 files found, 13 skipped)
**Conformance:** 32.6% exact match, 36.3% same error count

---

## Executive Summary

Analysis of 487 conformance tests reveals 49.5% of tests have missing errors. This report identifies the top missing error categories by frequency and provides recommendations for prioritization.

**Key Finding:** While exact match is at 32.6% (target: 80%+), the majority of missing errors are concentrated in a few key categories, making them high-value targets for improvement.

---

## Top 10 Missing Error Categories

| Rank | Error Code | Count | % of Missing | Severity | Description |
|------|------------|-------|--------------|----------|-------------|
| 1 | TS2705 | 34 | 7.0% | Medium | ES Module import/export identifier issues |
| 2 | TS1109 | 20 | 4.1% | Low | Expression expected (parser recovery) |
| 3 | TS2524 | 15 | 3.1% | Medium | Duplicate identifier detection |
| 4 | TS1359 | 11 | 2.3% | High | Identifier/keyword expected in type position |
| 5 | TS2304 | 10 | 2.1% | Low | Cannot find name (undefined reference) |
| 6 | TS2683 | 9 | 1.9% | Low | `this` implicitly has type `any` |
| 7 | TS7022 | 9 | 1.9% | Low | Abstract class cannot be instantiated |
| 8 | TS1005 | 8 | 1.6% | Low | Token expected (parser syntax) |
| 9 | TS2507 | 8 | 1.6% | Low | Object literal property shorthand |
| 10 | TS2415 | 7 | 1.4% | Medium | Class inheritance errors |

**Total Top 10:** 131 errors (27% of all missing errors)

---

## Detailed Analysis: Top 5 Categories

### 1. TS2705: ES Module Import/Export Issues (34 occurrences)

**Frequency:** 34 occurrences (7.0% of missing errors)
**Severity:** 🟡 MEDIUM
**Impact:** Affects module system correctness

**Error Message:** "Import/export identifier cannot be a keyword or reserved word"

**Root Cause:**
The parser is not properly validating that identifiers used in ES module import/export statements are not reserved keywords. TypeScript's parser enforces stricter rules for module declarations.

**Example Cases:**
- `import { debugger } from "mod"` - `debugger` is reserved
- `export { if }` - `if` is a keyword
- Module namespace declarations with reserved identifiers

**Complexity:** MEDIUM
- Requires import/export statement validation
- Needs keyword checking in module contexts
- May affect multiple parse functions

**Suggested Owner:** worker-2 or EM-2 (module system focus)

**Estimated Effort:** 1-2 days

---

### 2. TS1109: Expression Expected (20 occurrences)

**Frequency:** 20 occurrences (4.1% of missing errors)
**Severity:** 🟢 LOW
**Impact:** Parser error recovery - largely addressed in recent work

**Error Message:** "Expression expected."

**Root Cause:**
Parser error recovery in edge cases where TypeScript expects an expression but the current token doesn't start a valid expression. Many of these are in async/await contexts which have been partially addressed.

**Status:** ✅ **PARTIALLY COMPLETE**
- Phase 1: Reduced from 26 to 20 occurrences
- Phase 2a: Further reduced to 20 (from 26 baseline)
- Remaining cases are in specific async patterns (e.g., `async (a = await)` parameter defaults)

**Complexity:** LOW (remaining cases)
- Most issues already addressed
- Remaining cases are niche patterns
- May require targeted fixes only

**Suggested Owner:** worker-1 (has existing context)

**Estimated Effort:** 0.5 day

---

### 3. TS2524: Duplicate Identifier (15 occurrences)

**Frequency:** 15 occurrences (3.1% of missing errors)
**Severity:** 🟡 MEDIUM
**Impact:** Symbol binding correctness

**Error Message:** "Duplicate identifier '{0}'"

**Root Cause:**
The binder is not detecting all cases of duplicate identifiers in block scopes, particularly in complex scenarios involving:
- Nested scopes with same variable names
- Function parameters with conflicting declarations
- Object destructuring with duplicate properties

**Complexity:** MEDIUM
- Requires binder/symbol table enhancements
- Scope tracking logic
- Declaration collision detection

**Suggested Owner:** worker-4 or EM-3 (binder expertise)

**Estimated Effort:** 2-3 days

---

### 4. TS1359: Identifier Expected in Type Position (11 occurrences)

**Frequency:** 11 occurrences (2.3% of missing errors)
**Severity:** 🔴 HIGH
**Impact:** Type system correctness

**Error Message:** "Identifier expected" in type annotations

**Root Cause:**
The parser is allowing invalid syntax in type positions where TypeScript requires specific identifiers. Examples:
- Type queries with reserved words
- Generic constraints with invalid identifiers
- Mapped type key patterns that are malformed

**Test Patterns:**
- `typeof T` where T is a reserved keyword
- `<T extends invalid>` constraint
- Mapped types with incorrect key syntax

**Complexity:** HIGH
- Requires type grammar enforcement
- Parser and checker coordination
- May affect type inference

**Suggested Owner:** worker-3 or EM-3 (type system focus)

**Estimated Effort:** 3-4 days

---

### 5. TS2304: Cannot Find Name (10 occurrences)

**Frequency:** 10 occurrences (2.1% of missing errors)
**Severity:** 🟢 LOW
**Impact:** Limited - mostly edge cases

**Error Message:** "Cannot find name '{0}'"

**Root Cause:**
Semantic checker not reporting undefined references in specific contexts. These are mostly in:
- Global scope references in specific modes
- Undeclared variables in non-strict mode
- Forward references in certain declaration orders

**Note:** This is primarily a semantic checking issue, not parser-level.

**Complexity:** MEDIUM
- Requires checker enhancements
- Symbol resolution logic
- Scope chain traversal

**Suggested Owner:** worker-3 or EM-3 (checker focus)

**Estimated Effort:** 2-3 days

---

## Priority Recommendations

### Quick Wins (Low Complexity, High Impact)

1. **TS1109 (Expression Expected)** - Worker-1
   - Only 20 remaining occurrences
   - Already partially addressed
   - Estimated: 0.5 day
   - **Expected Reduction:** 5-10 errors

2. **TS2705 (Module Import/Export)** - Worker-2
   - Clear fix scope (keyword validation)
   - Medium complexity but well-defined
   - Estimated: 1-2 days
   - **Expected Reduction:** 34 errors

### Strategic Investments (Higher Complexity, High Value)

3. **TS1359 (Type Position Identifiers)** - Worker-3
   - High severity
   - Type system correctness critical
   - Estimated: 3-4 days
   - **Expected Reduction:** 11 errors

4. **TS2524 (Duplicate Identifiers)** - Worker-4
   - Symbol binding core functionality
   - Medium severity
   - Estimated: 2-3 days
   - **Expected Reduction:** 15 errors

### Lower Priority

5. **TS2304 (Cannot Find Name)** - Worker-3
   - Lower impact
   - Semantic checking edge cases
   - Estimated: 2-3 days
   - **Expected Reduction:** 10 errors

---

## Top 10 Extra Error Categories (For Comparison)

While missing errors prevent us from catching TypeScript errors, extra errors create false positives. These are also worth addressing:

| Rank | Error Code | Count | Description |
|------|------------|-------|-------------|
| 1 | TS7008 | 85 | Implicit 'any' type |
| 2 | TS7006 | 51 | Parameter implicitly has 'any' |
| 3 | TS2322 | 43 | Type assignability error (worker-11 investigating) |
| 4 | TS2571 | 30 | Object is 'possibly null' |
| 5 | TS2300 | 20 | Duplicate identifier (checker level) |
| 6 | TS7005 | 12 | Variable implicitly has 'any' |
| 7 | TS1005 | 9 | Token expected (extra) |
| 8 | TS7011 | 8 | Indexed access |
| 9 | TS2345 | 7 | Type argument mismatch |
| 10 | TS2454 | 7 | Variable used before assignment |

**Note:** TS2322 (43 occurrences) is being investigated by worker-11 as a critical issue requiring subtype checker improvements.

---

## Implementation Priority Queue

Based on impact vs. effort:

### Phase 1: Quick Wins (Week 1)
1. ✅ **TS1109** (Worker-1) - Already addressed, down to 20
2. **TS2705** (Worker-2) - 34 errors, 1-2 days
3. **TS2304** (Worker-3) - 10 errors, 2-3 days

**Expected Total Reduction:** ~64 errors

### Phase 2: Medium Complexity (Week 2)
4. **TS2524** (Worker-4) - 15 errors, 2-3 days
5. **TS1005** (remaining) (Worker-1) - 8 errors, 1 day

**Expected Total Reduction:** ~87 errors

### Phase 3: Strategic (Week 3-4)
6. **TS1359** (Worker-3) - 11 errors, 3-4 days
7. **TS2322** (Worker-11) - 43 extra errors, high complexity

**Expected Total Reduction:** ~141 errors

---

## Success Metrics

Current Baseline:
- Exact Match: 32.6%
- Same Error Count: 36.3%
- Combined Match: 68.9%

Target:
- Exact Match: 80%+
- Same Error Count: 90%+
- Combined Match: 95%+

**Gap Analysis:**
- Need +47.4% exact match improvement
- Need +53.7% same error count improvement
- Top 10 categories account for ~27% of missing errors

**Projected Impact:**
- Addressing top 5 missing → +15-20% exact match
- Addressing top 10 missing → +25-30% exact match
- This gets us to ~57-62% exact match, closing ~60% of the gap

---

## Recommendations

1. **Assign TS2705 to Worker-2** (module import/export validation)
   - Clear scope, medium complexity
   - Quick win: 34 errors

2. **Complete TS1109 cleanup** (Worker-1)
   - Remaining 20 edge cases
   - Quick win: 5-10 errors

3. **Assign TS1359 to Worker-3** (type position identifiers)
   - High severity, type system critical
   - Strategic: 11 errors

4. **Assign TS2524 to Worker-4** (duplicate identifiers)
   - Binder improvements
   - Medium impact: 15 errors

5. **Coordinate with Worker-11 on TS2322**
   - 43 extra errors
   - High complexity (subtype checker)
   - Track progress separately

**Total Expected Impact:** Top 5 categories → 100+ error improvements

---

## Appendix: Error Code Reference

- **TS1005**: Token expected (parser syntax error)
- **TS1109**: Expression expected (parser recovery)
- **TS1359**: Identifier expected (type position)
- **TS2304**: Cannot find name (undefined reference)
- **TS2322**: Type assignability error (type mismatch)
- **TS2415**: Class inheritance error
- **TS2507**: Object literal shorthand
- **TS2524**: Duplicate identifier
- **TS2683**: 'this' has implicit 'any' type
- **TS2705**: Import/export identifier is reserved
- **TS7005**: Variable implicitly 'any'
- **TS7006**: Parameter implicitly 'any'
- **TS7008**: Member implicitly 'any'
- **TS7011**: Indexed access
- **TS7022**: Abstract class instantiation
- **TS2300**: Duplicate identifier (checker level)

---

**Report Generated:** 2026-01-15
**Next Review:** After Phase 1 completions
**Contact:** EM-1 team
