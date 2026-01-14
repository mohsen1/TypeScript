# Worker 4 - Task 1: TS1005 Emission Audit

**Date:** 2026-01-14
**Status:** ✅ COMPLETE
**Squad:** Parser (False Positive Reduction)

---

## Executive Summary

This document provides a comprehensive analysis of TS1005 ("expected X") error emission in the TypeScript WASM compiler. TS1005 is one of the primary contributors to the 701 parser false positives that need to be reduced to <100.

---

## 1. TS1005 Error Definition

**Error Code:** `TOKEN_EXPECTED` (1005)
**Message Format:** "'{token}' expected"
**Diagnostic Module:** `wasm/src/checker/types/diagnostics.rs:200`

Common examples:
- `';' expected`
- `'}' expected`
- `'=' expected`
- `'>' expected`
- `'=>' expected`

---

## 2. TS1005 Emission Points

### Primary Emission Function
**Location:** `wasm/src/thin_parser.rs:432`
```rust
fn error_token_expected(&mut self, token: &str) {
    // Only emit error if we haven't already emitted one at this position
    // This prevents cascading errors when parse_semicolon() and similar functions call this
    if self.token_pos() != self.last_error_pos {
        self.parse_error_at_current_token(
            &format!("'{}' expected", token),
            diagnostic_codes::TOKEN_EXPECTED,
        );
    }
}
```

### Key Emission Locations

| Line | Context | Token | Notes |
|------|---------|--------|-------|
| 313 | parse_expected() | Various | Generic token expectation |
| 495 | parse_semicolon() | `;` | Semicolon with ASI checks |
| 733 | Type parameter list | `>` | Generic type parameter |
| 3996 | Type alias | `=` | Missing equals in type alias |
| 5871 | Arrow function | `=>` | Arrow token expected |
| 7140 | Class body | `}` | Closing brace |
| 7182 | Template literal | `` ` `` | Template literal end |

### parse_semicolon() Pattern (Most Common)
**Location:** `wasm/src/thin_parser.rs:491`
```rust
fn parse_semicolon(&mut self) {
    if self.is_token(SyntaxKind::SemicolonToken) {
        self.next_token();
    } else if !self.can_parse_semicolon() {
        self.error_token_expected(";");
    }
}
```

**ASI Check:** `can_parse_semicolon()` at line 501
- Returns true if semicolon present OR
- Returns true if ASI applies (close brace, EOF, or preceding line break)
- Only emits TS1005 if ASI doesn't apply

**Usage Count:** ~40+ calls throughout the parser

---

## 3. Existing Cascading Error Prevention

### Mechanism 1: Position-Based Suppression
**Location:** `wasm/src/thin_parser.rs:88-89, 352-353`
```rust
last_error_pos: u32,  // Track position of last error

// In error_token_expected:
if self.token_pos() != self.last_error_pos {
    // Only emit if not same position as last error
}
```

**Purpose:** Prevents multiple errors at the same position (e.g., "';' expected" followed by "')' expected")

### Mechanism 2: Proximity-Based Suppression (TS1109)
**Location:** `wasm/src/thin_parser.rs:376-390`
```rust
// In error_expression_expected():
// Suppress TS1109 if within 50 characters of last error
if self.last_error_pos > 0
    && current_pos > self.last_error_pos
    && current_pos < self.last_error_pos.saturating_add(50)
{
    return;  // Suppress cascading TS1109
}
```

**Purpose:** Prevents "Expression expected" errors after TS1005 when parser recovers to next token

### Mechanism 3: Resynchronization
**Location:** `wasm/src/thin_parser.rs:554-559`
```rust
fn resync_after_error(&mut self) {
    // If we're already at a statement start or EOF, no need to resync
    if self.is_statement_start() || self.is_token(SyntaxKind::EndOfFileToken) {
        return;
    }
    // Skip to next statement boundary
}
```

### Mechanism 4: Import/Export Brace Mismatch Detection
**Location:** `wasm/src/thin_parser.rs:4552-4557, 4833-4837`
```rust
// If we encounter 'from' keyword in specifier list, break loop
// This handles: import { a from "module"  (missing closing brace)
if self.is_token(SyntaxKind::FromKeyword) {
    break;
}
```

---

## 4. Common False Positive Patterns

### Pattern 1: ASI Edge Cases
**Scenario:** Semicolon inserted by ASI but parser still emits error
**Example:**
```typescript
return
    x + y;
```
**Expected:** No error (ASI applies)
**Actual:** May emit TS1005 depending on line break detection

**Root Cause:** `has_preceding_line_break()` may not correctly detect line breaks in all cases

### Pattern 2: Trailing Commas
**Scenario:** Trailing comma in enum or object literal
**Example:**
```typescript
enum E { A = 1, B = 2, }
```
**Expected:** No error (trailing commas are valid)
**Actual:** May emit TS1005 if parser expects closing brace

### Pattern 3: Missing Closing Brace Cascading Errors
**Scenario:** Missing brace causes multiple "expected" errors
**Example:**
```typescript
function f() {
    return 42;
// Missing closing brace
console.log("next");
```
**Expected:** One "}' expected" error
**Actual:** May emit multiple TS1005 errors before recovering

### Pattern 4: Arrow Function Type Annotations
**Scenario:** Arrow function with missing parameter type
**Example:**
```typescript
let f = (a: ) => {};
```
**Expected:** TS1110 ("Type expected")
**Actual:** May emit generic TS1005 ("identifier expected")

### Pattern 5: Template Literal Expressions
**Scenario:** Complex template literal with missing parts
**Example:**
```typescript
let s = `Hello ${name world`;
```
**Expected:** Specific error about template syntax
**Actual:** May emit generic TS1005

---

## 5. Proposed Fix Strategies

### Strategy 1: Enhanced ASI Detection
**Priority:** HIGH
**Impact:** Reduces semicolon false positives

**Implementation:**
1. Improve `has_preceding_line_break()` detection
2. Add lookahead for statement-starting keywords
3. Check if next token can legally follow current statement

**Target:** Reduce TS1005 semicolon false positives by 50%

### Strategy 2: Context-Specific Error Messages
**Priority:** MEDIUM
**Impact:** Better user experience, easier to identify true vs false positives

**Implementation:**
1. Replace generic "identifier expected" with context-specific messages
2. Use `can_token_start_type()` to emit TS1110 instead of TS1005 in type positions
3. Add specialized errors for common patterns (arrow functions, enums, etc.)

**Code Reference:** `can_token_start_type()` already exists at line 213

### Strategy 3: Aggressive Cascading Error Suppression
**Priority:** HIGH
**Impact:** Reduces multiple errors from single syntax mistake

**Implementation:**
1. Increase proximity suppression radius from 50 to 100 characters
2. Add "error budget" system: max 3 TS1005 errors per 500 characters
3. Implement statement-level error suppression (only first TS1005 per statement)

**Target:** Reduce cascading TS1005 errors by 70%

### Strategy 4: Smart Recovery Points
**Priority:** MEDIUM
**Impact:** Better parser recovery after errors

**Implementation:**
1. Enhance `resync_after_error()` to detect more recovery points
2. Add lookahead for class/function/expression boundaries
3. Implement "error barrier" at statement boundaries

**Code Reference:** `resync_after_error()` at line 554

### Strategy 5: Type Predicate Detection
**Priority:** LOW
**Impact:** Prevents TS1005 in type predicates

**Implementation:**
1. Enhance type predicate detection (already partially implemented)
2. Add better lookahead for `is`/`asserts` keywords in type positions

**Code Reference:** Line 7967 - type predicate parsing already exists

---

## 6. Baseline Measurement Plan

### Method 1: Differential Test (Recommended)
**Script:** `wasm/differential-test/find-ts1005.mjs` (created)
**Command:**
```bash
cd wasm
./differential-test/run-conformance.sh --max=10000
node differential-test/find-ts1005.mjs
```

**Metrics:**
- Total tsc TS1005 errors
- Total WASM TS1005 errors
- False positives (WASM but not tsc)
- False negatives (tsc but not WASM)
- Top 10 worst files

### Method 2: Unit Test Baseline
**Test File:** `wasm/src/thin_parser_tests.rs`
**Existing Tests:** Multiple TS1005 regression tests exist
- Line 252: "Unexpected TS1005" assertions
- Line 3426: "TS1005/TS1068 False Positive Regression Tests"
- Line 3604: "Error Recovery Tests for TS1005/TS1109"

---

## 7. Acceptance Criteria Status

| Criterion | Status | Notes |
|-----------|--------|-------|
| Clear understanding of TS1005 emission logic | ✅ COMPLETE | All emission points identified |
| Documented list of false positive patterns | ✅ COMPLETE | 5 patterns documented |
| Baseline metrics established | ⏳ PENDING | Requires Docker/WASM build |
| Proposed fix strategies documented | ✅ COMPLETE | 5 strategies with priorities |

---

## 8. Next Steps

### Immediate (Task 2):
1. ✅ Complete TS1005 audit (this document)
2. ⏳ Run baseline measurement (requires Docker environment)
3. ⏳ Audit TS1109 emissions
4. ⏳ Implement top-priority fixes

### Task 3 (Error Recovery Implementation):
1. Implement Strategy 1: Enhanced ASI detection
2. Implement Strategy 3: Aggressive cascading suppression
3. Add unit tests for edge cases
4. Validate with conformance suite

### Task 4 (Validation):
1. Measure TS1005 reduction (target: 300+ false positives eliminated)
2. Measure TS1109 reduction (target: 200+ false positives eliminated)
3. Verify no regressions in other error types
4. Document final results

---

## 9. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking existing tests | Medium | High | Comprehensive unit test coverage |
| Introducing false negatives | Medium | High | Differential testing against tsc |
| Performance regression | Low | Medium | Benchmark before/after |
| ASI edge cases | High | Medium | Incremental rollout with testing |

---

## 10. References

### Source Files
- `wasm/src/thin_parser.rs` - Parser implementation
- `wasm/src/thin_parser_tests.rs` - Parser tests
- `wasm/src/checker/types/diagnostics.rs` - Error codes

### Key Functions
- `error_token_expected()` - Primary TS1005 emission (line 432)
- `parse_semicolon()` - Semicolon parsing with ASI (line 491)
- `can_parse_semicolon()` - ASI detection (line 501)
- `error_expression_expected()` - TS1109 with suppression (line 373)
- `resync_after_error()` - Error recovery (line 554)

### Test Scripts Created
- `wasm/differential-test/find-ts1005.mjs` - TS1005 differential test

---

**Document Status:** Ready for review
**Next Action:** Proceed to Task 2 (TS1109 Audit) or run baseline measurement
