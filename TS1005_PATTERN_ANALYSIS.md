# TS1005 Pattern Analysis

**Date:** 2026-01-14
**Worker:** Worker-1
**Task:** Reduce remaining TS1005 errors from 331 to <100

---

## Summary

Analyzed **284 baseline test files** containing TS1005 errors to identify common patterns. The analysis shows that most TS1005 errors in the baseline tests are **intentional error cases** (testing that the compiler correctly detects invalid syntax). The 331 "extra" errors in conformance tests are likely different - these represent cases where our compiler reports TS1005 but TypeScript does not.

---

## Top TS1005 Error Messages (Baseline Tests)

| Error Message | Count | Percentage |
|---------------|-------|------------|
| ';' expected. | 440 | 34.1% |
| ',' expected. | 394 | 30.5% |
| '{' expected. | 227 | 17.6% |
| ':' expected. | 93 | 7.2% |
| '}' expected. | 90 | 7.0% |
| '>' expected. | 61 | 4.7% |
| '(' expected. | 58 | 4.5% |
| ')' expected. | 51 | 3.9% |
| 'with' expected. | 48 | 3.7% |
| '=>' expected. | 41 | 3.2% |

**Total:** 1,293 TS1005 errors across 284 test files

---

## Pattern Categories

### 1. Arrow Function Syntax Errors (41 files)

**Pattern:** Missing `=>` or missing body in arrow functions

**Example:**
```typescript
// Missing =>
var a = () { };
var b = (x: number) { };

// Missing body braces
var c = () => var k = 10;
```

**Error:**
- `'{' expected` when missing `=>`
- `'{' expected` when body not enclosed in braces

**Assessment:** These are **intentional error tests** - not false positives.

---

### 2. Semicolon Insertion (ASI) Edge Cases (440 errors)

**Pattern:** ASI should insert semicolon but compiler doesn't

**Common ASI Rules:**
1. **Return statements:** ASI applies after `return` if newline follows
2. **Throw statements:** ASI applies after `throw` if newline follows
3. **Break/Continue:** ASI applies after `break`/`continue` if newline follows
4. **Restricted productions:** ASI prevented in certain contexts

**Current Implementation** (`thin_parser.rs:540-575`):
```rust
fn can_parse_semicolon(&self) -> bool {
    // 1. Explicit semicolon
    if self.is_token(SyntaxKind::SemicolonToken) {
        return true;
    }

    // 2. ASI applies before closing brace
    if self.is_token(SyntaxKind::CloseBraceToken) {
        return true;
    }

    // 3. ASI applies at EOF
    if self.is_token(SyntaxKind::EndOfFileToken) {
        return true;
    }

    // 4. ASI applies after line break
    if self.scanner.has_preceding_line_break() {
        // Enhanced ASI: Check if next token starts a statement
        if self.is_statement_start() {
            return true;
        }
        // Also allow ASI before common statement delimiters
        if self.is_token(SyntaxKind::CloseParenToken)
            || self.is_token(SyntaxKind::CloseBracketToken)
        {
            return true;
        }
    }

    false
}
```

**Potential Gaps:**
1. **Restricted productions:** Not explicitly handled
2. **Postfix expressions:** ASI blocked before `++`, `--` on same line
3. **Do-while loops:** ASI should always apply after `while` condition
4. **Expression statements:** May need refinement for edge cases

---

### 3. Type Annotation Errors (93 errors)

**Pattern:** Missing `:` in type annotations

**Example:**
```typescript
// Type annotation without colon
var v = (a) => { };  // OK
var v = (a): => { }; // ERROR - ':' not allowed here
```

**Assessment:** Usually intentional error tests.

---

### 4. Comma/Parameter Errors (394 errors)

**Pattern:** Missing commas in parameter lists, object literals, etc.

**Example:**
```typescript
function f(a b: string) {} // Missing comma
```

**Assessment:** Usually intentional error tests.

---

### 5. Destructuring Pattern Errors

**Pattern:** Invalid destructuring syntax

**Example:**
```typescript
// VariableDeclaration13_es6.ts
var let: any;
let[0] = 100;  // Ambiguous - 'let' is both variable and keyword
```

**Assessment:** Intentional error test for ambiguous syntax.

---

## Current Parser ASI Strengths

The parser already has **enhanced ASI** that:
1. ✅ Checks for line breaks before applying ASI
2. ✅ Verifies next token starts a statement
3. ✅ Handles statement delimiters (closing braces/parens/brackets)
4. ✅ Has comprehensive statement start detection

---

## Recommended Improvements

### Priority 1: Enhanced Expression Statement ASI

**Issue:** ASI may not apply in some expression statement contexts

**Fix:** Add more tokens to statement start detection:
```rust
// Add to is_statement_start():
SyntaxKind::NumericLiteral
| SyntaxKind::TrueKeyword
| SyntaxKind::FalseKeyword
| SyntaxKind::NullKeyword
| SyntaxKind::ThisKeyword
| SyntaxKind::SuperKeyword
| SyntaxKind::OpenParenToken  // parenthesized expressions
| SyntaxKind::OpenBracketToken  // array literals
| SyntaxKind::MinusToken  // unary minus
| SyntaxKind::PlusToken  // unary plus
| SyntaxKind::ExclamationToken  // ! operator
| SyntaxKind::TildeToken  // ~ operator
| SyntaxKind::AtToken  // already included
```

**Expected Impact:** Reduces false positive TS1005 errors for expression statements

---

### Priority 2: Restricted Production Detection

**Issue:** ASI should be blocked in certain "restricted" productions:
- After `++` or `--` (postfix operators)
- In `for` statement headers
- In `if`/`while` condition parentheses

**Fix:** Add context awareness to ASI:
```rust
// Track if we're in a restricted production
// If so, don't apply ASI even after line break
```

**Expected Impact:** Prevents incorrect semicolon insertion that would hide errors

---

### Priority 3: Do-While Loop ASI

**Issue:** Do-while loops require semicolon after `while` condition

**Fix:** Ensure ASI always applies after do-while:
```rust
// In do-while parsing, explicitly check for semicolon or apply ASI
```

**Expected Impact:** Handles edge case in do-while loops

---

### Priority 4: Conformance Test Analysis

**Action Required:**
1. Build WASM module
2. Run full conformance test suite
3. Extract ACTUAL extra TS1005 errors (not baseline tests)
4. Categorize by pattern
5. Fix top 5 patterns

**Files to analyze:**
- Actual conformance test failures
- Not the intentional error baseline tests

---

## Next Steps

1. **Build WASM** to enable conformance testing
2. **Run targeted analysis** on actual extra TS1005 errors
3. **Implement Priority 1** (Expression Statement ASI)
4. **Test and measure** improvement
5. **Iterate** on remaining patterns

---

## Notes

- The 284 baseline test files are **intentional error tests** - they verify the compiler correctly detects invalid syntax
- The **331 extra TS1005 errors** in conformance tests represent false positives we need to fix
- Current ASI implementation is solid but may need refinement for expression literals
- Focus should be on **conformance test errors**, not baseline intentional errors

---

**Author:** Worker-1 (EM-1 Team)
**Co-Authored-By:** Claude Sonnet 4.5 <noreply@anthropic.com>
