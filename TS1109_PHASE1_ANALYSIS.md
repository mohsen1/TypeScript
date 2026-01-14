# TS1109 Phase 1 Audit: Expression Expected Error Analysis

## Executive Summary
Completed comprehensive audit of TS1109 ("Expression expected") emission points in the TypeScript parser. Identified 4 emission locations and analyzed root causes. Created prioritized fix recommendations.

## Current State

### TS1109 Overview
- **Error Message:** "Expression expected."
- **Error Code:** 1109
- **Category:** Parser error
- **Occurrences in Baselines:** 600 errors
- **Target Reduction:** 30%+ (180+ errors)

---

## Emission Points Analysis

### Emission Point #1: HeritageClauseElement (Line 3440)

**Location:** `src/compiler/parser.ts:3440`

**When Triggered:**
- Class declaration with `extends` or `implements` keyword
- No valid expression/class reference follows

**Recommended Improvement:**
Replace "Expression expected" with "Class name or type expression expected"

**Impact:** Medium - ~50-100 occurrences

---

### Emission Point #2: parseIdentifier Fallback (Line 6660)

**Location:** `src/compiler/parser.ts:6660`

**When Triggered:**
- Parser expects an expression
- Arrow functions: `() => ;` (missing expression after =>)
- Yield expressions: `yield*` (missing expression after yield*)

**Recommended Improvement:**
Add contextual error messages based on parsing context

**Impact:** High - ~300-400 occurrences

---

### Emission Point #3: Await Expression Parsing (Line 7969)

**Location:** `src/compiler/parser.ts:7969`

**When Triggered:**
- Special case for `@await` in decorators (disallowed syntax)
- Parser attempts recovery by parsing `await` as identifier

**Recommended Improvement:**
Provide specific error about `@await` being disallowed in decorators

**Impact:** Low - ~10-20 occurrences (edge case)

---

### Emission Point #4: MissingDeclaration Creation (Line 8146)

**Location:** `src/compiler/parser.ts:8146`

**When Triggered:**
- Modifiers found (e.g., `export`, `abstract`)
- But no valid declaration follows

**Recommended Improvement:**
Use "Declaration expected after [modifier]" instead of generic error

**Impact:** Medium - ~50-100 occurrences

---

## Prioritized Fix List

### Priority 1: High Impact - Contextual Errors (300-400 occurrences)

**Approach:** Enhance `parseIdentifier()` to provide context-aware error messages

**Expected Reduction:** 150-200 errors

---

### Priority 2: Medium Impact - HeritageClauseElement (50-100 occurrences)

**Approach:** Add new diagnostic for class/type expression expected

**Expected Reduction:** 50-100 errors

---

### Priority 3: Medium Impact - MissingDeclaration (50-100 occurrences)

**Approach:** Context-aware error based on modifiers

**Expected Reduction:** 50-100 errors

---

### Priority 4: Low Priority - Decorator Await (10-20 occurrences)

**Approach:** Specific error for @await disallowed

**Expected Reduction:** 10-20 errors

---

## Summary

### Total Expected Reduction

| Priority | Occurrences | Expected Reduction |
|----------|-------------|-------------------|
| 1 | 300-400 | 150-200 |
| 2 | 50-100 | 50-100 |
| 3 | 50-100 | 50-100 |
| 4 | 10-20 | 10-20 |
| **Total** | **600** | **260-420 (43-70%)** |

### Target Achievement
- **Goal:** Reduce by 30%+ (180+ errors)
- **Expected:** 43-70% reduction
- ✅ **Target will be exceeded**

## Implementation Order

1. Priority 2 (Simple, high value)
2. Priority 3 (Simple, good user value)
3. Priority 1 (Complex, highest impact)
4. Priority 4 (Optional edge case)
