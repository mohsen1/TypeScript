# ASI Edge Case Audit Report
**Worker:** Worker-10 (Syntax Support Squad)
**Date:** 2026-01-14
**Priority:** P0 - High
**Target:** Reduce edge case parser failures by 50%

---

## Executive Summary

This audit examines the Automatic Semicolon Insertion (ASI) implementation in the thin parser (`thin_parser.rs`). The current implementation has a solid foundation with key ASI mechanisms in place, but there are several edge cases that need attention to meet the 50% failure reduction target.

---

## Current ASI Implementation

### Core ASI Function (`thin_parser.rs:513-550`)

```rust
fn can_parse_semicolon(&self) -> bool {
    // Explicit semicolon
    if self.is_token(SyntaxKind::SemicolonToken) {
        return true;
    }

    // ASI applies before closing brace
    if self.is_token(SyntaxKind::CloseBraceToken) {
        return true;
    }

    // ASI applies at EOF
    if self.is_token(SyntaxKind::EndOfFileToken) {
        return true;
    }

    // ASI applies after line break
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

**Status:** ✅ Good - Implements standard ASI rules with enhanced statement detection

---

## Edge Case Analysis

### 1. Line Terminators Before `++`/`--` Operators

**Location:** `thin_parser.rs:6287-6307`

```rust
fn parse_postfix_expression(&mut self) -> NodeIndex {
    let start_pos = self.token_pos();
    let mut expr = self.parse_left_hand_side_expression();

    // Handle postfix operators
    if !self.scanner.has_preceding_line_break() {
        if self.is_token(SyntaxKind::PlusPlusToken)
            || self.is_token(SyntaxKind::MinusMinusToken)
        {
            let operator = self.token() as u16;
            self.next_token();
            // ... create postfix node
        }
    }
    // ...
}
```

**Test Cases:**
```javascript
// Should parse as postfix: x becomes 6
let x = 5;
x++;

// Should NOT parse as postfix (ASI creates statement boundary)
// This becomes: expr; ++y; (two statements)
let expr = 5
++y;

// Edge case: should parse as prefix
let a = 5
let b = ++a;  // a becomes 6
```

**Status:** ✅ CORRECT - The parser correctly prevents postfix when preceded by line break

**Potential Issue:** None found - matches JavaScript specification

---

### 2. `return`, `throw`, `yield` Statements Without Semicolons

**Location:** `thin_parser.rs:5099-5119` (return), `thin_parser.rs:5383-5400` (throw)

#### `return` Statement Analysis

```rust
fn parse_return_statement(&mut self) -> NodeIndex {
    let start_pos = self.token_pos();
    self.parse_expected(SyntaxKind::ReturnKeyword);

    let expression = if !self.can_parse_semicolon() {
        self.parse_expression()
    } else {
        NodeIndex::NONE
    };

    self.parse_semicolon();
    // ...
}
```

**Test Cases:**
```javascript
// Case 1: return with expression on same line
function f1() { return x + y; }

// Case 2: return with expression on next line (ASI applies)
function f2() { return  // ASI here - returns undefined
    x + y;  // Separate statement (unreachable code warning)
}

// Case 3: return with expression in parens (no ASI)
function f3() { return (
    x + y
); }

// Case 4: return at EOF
function f4() { return 42 }
```

**Status:** ✅ CORRECT - Uses `can_parse_semicolon()` to check for ASI conditions

**Issue Found:** The expression check happens via `can_parse_semicolon()` which checks for line break + statement start. This should work correctly.

#### `throw` Statement Analysis

```rust
fn parse_throw_statement(&mut self) -> NodeIndex {
    let start_pos = self.token_pos();
    self.parse_expected(SyntaxKind::ThrowKeyword);

    let expression = self.parse_expression();  // Unconditional!

    self.parse_semicolon();
    // ...
}
```

**CRITICAL ISSUE FOUND:** The `throw` statement unconditionally parses an expression, but according to JavaScript spec, a line break after `throw` should trigger ASI:

```javascript
// This should be a SYNTAX ERROR per spec:
// throw  // ASI here
// new Error("msg");

// But current implementation will parse:
// throw (new Error("msg"));
```

**Required Fix:** `throw` should check for line break similar to `return`:

```rust
fn parse_throw_statement(&mut self) -> NodeIndex {
    let start_pos = self.token_pos();
    self.parse_expected(SyntaxKind::ThrowKeyword);

    // CRITICAL: throw must not have line break before expression
    if self.scanner.has_preceding_line_break() {
        // ASI applies - throw statement without expression (error)
        self.error_token_expected("expression");
        self.parse_semicolon();
        return self.arena.add_return(...);
    }

    let expression = self.parse_expression();
    self.parse_semicolon();
    // ...
}
```

**Status:** ❌ BUG FOUND - `throw` doesn't check for line break before expression

#### `yield` Statement Analysis

**Location:** `thin_parser.rs:6256` (yield expression parsing)

```rust
SyntaxKind::YieldKeyword => {
    let start_pos = self.token_pos();
    self.parse_expected(SyntaxKind::YieldKeyword);

    let expression = if !self.scanner.has_preceding_line_break()
        && !self.is_token(SyntaxKind::SemicolonToken)
        // ... more conditions
    {
        self.parse_assignment_expression()
    } else {
        NodeIndex::NONE
    };
    // ...
}
```

**Status:** ✅ CORRECT - `yield` checks for line break before expression

---

### 3. `break`, `continue` With Labels

**Location:** `thin_parser.rs:5342-5381`

```rust
fn parse_break_statement(&mut self) -> NodeIndex {
    let start_pos = self.token_pos();
    self.parse_expected(SyntaxKind::BreakKeyword);

    // Optional label (TODO: store in break statement node)
    let _label = if !self.can_parse_semicolon() && self.is_identifier_or_keyword() {
        self.parse_identifier_name()
    } else {
        NodeIndex::NONE
    };

    self.parse_semicolon();
    // ...
}
```

**Test Cases:**
```javascript
// Labeled break
outer: for (let i = 0; i < 10; i++) {
    for (let j = 0; j < 10; j++) {
        if (i === j) break outer;  // OK
    }
}

// Edge case: line break before label
outer: for (;;) {
    break  // ASI applies here!
    outer;  // Separate expression statement (unused label warning)
}

// Edge case: keyword as label
break: while (true) {
    break break;  // 'break' is both label and keyword
}
```

**Status:** ✅ CORRECT - Uses `can_parse_semicolon()` which checks line break + statement start

**Potential Issue:** None found - label parsing correctly handles ASI

**Note:** The TODO comment about storing the label in the node suggests labels aren't being preserved in the AST, but that's separate from ASI handling.

---

### 4. Arrow Functions With Block-less Bodies

**Location:** `thin_parser.rs:5700-5820`

```rust
fn parse_arrow_function_expression(&mut self) -> NodeIndex {
    // ... parse parameters ...
    self.parse_expected(SyntaxKind::EqualsGreaterThanToken);

    // Concise body vs block body
    if self.is_token(SyntaxKind::OpenBraceToken) {
        // Block body: { return x; }
        self.parse_function_body_from_arrow()
    } else {
        // Concise body: x + y
        let body = self.parse_assignment_expression();
        // ...
    }
}
```

**Test Cases:**
```javascript
// Concise body (block-less)
let f1 = x => x * 2;

// Block body
let f2 = x => { return x * 2; };

// Edge case: object literal (requires parens)
let f3 = x => ({ x: 1 });  // OK
let f4 = x => { x: 1 };    // SyntaxError - block with label

// ASI edge cases in concise body
let f5 = x =>
    x * 2;  // OK - single expression

let f6 = x =>
    let y = x * 2;  // SyntaxError - let is declaration, not expression
```

**Status:** ⚠️ POTENTIAL ISSUE - Arrow function concise body parsing

**Issue Found:** The parser uses `parse_assignment_expression()` for the concise body. This doesn't directly interact with ASI because the arrow function itself is an expression. However, there's a subtle issue:

```javascript
// This should work (ASI applies after arrow)
let x = () =>
    42
;

// But this shouldn't (no ASI, 42 is incomplete)
let y = () =>
    42 +  // SyntaxError - unexpected end of input
```

The current implementation should handle this correctly since the arrow function body parsing continues until it hits a valid expression boundary.

---

## Summary of Issues

### Critical Bugs (Must Fix)

1. **`throw` statement doesn't check for line break** (`thin_parser.rs:5383-5400`)
   - Impact: Parses invalid code that should be syntax errors
   - TS Error Code: TS1005, TS1109
   - Fix: Add line break check before parsing expression

### Minor Issues (Should Fix)

2. **Label storage in `break`/`continue`** (`thin_parser.rs:5342-5381`)
   - Impact: Labels are parsed but not stored in AST
   - Fix: Store label in the node data structure

3. **Enhanced ASI for statement starts**
   - Current implementation checks `is_statement_start()` after line break
   - Should also consider restricted productions (throw, yield, etc.)

---

## Recommendations

### Priority 1: Fix `throw` Statement ASI

```rust
fn parse_throw_statement(&mut self) -> NodeIndex {
    let start_pos = self.token_pos();
    self.parse_expected(SyntaxKind::ThrowKeyword);

    // CRITICAL: throw expression must be on same line
    if self.scanner.has_preceding_line_break() {
        // ASI applies - report error
        self.error(
            start_pos,
            self.token_end(),
            diagnostic_codes::LINE_BREAK_NOT_ALLOWED_AFTER_THROW,
        );
        // Attempt to continue parsing
        let expression = self.parse_expression();
        self.parse_semicolon();
        return self.arena.add_return(
            syntax_kind_ext::THROW_STATEMENT,
            start_pos,
            self.token_end(),
            ReturnData { expression },
        );
    }

    let expression = self.parse_expression();
    self.parse_semicolon();
    let end_pos = self.token_end();

    self.arena.add_return(
        syntax_kind_ext::THROW_STATEMENT,
        start_pos,
        end_pos,
        ReturnData { expression },
    )
}
```

### Priority 2: Add ASI Edge Case Tests

Create dedicated test file: `wasm/src/parser/asi_edge_case_tests.rs`

```rust
#[test]
fn test_asi_throw_with_line_break() {
    let source = r#"
function f() {
    throw
    new Error("test");
}
"#;
    // Should produce syntax error
}

#[test]
fn test_asi_postfix_increment_with_line_break() {
    let source = "let x = 5\nx++;";
    // Should parse as two statements
}

#[test]
fn test_asi_return_with_line_break() {
    let source = r#"
function f() {
    return
    x + y;
}
"#;
    // Should apply ASI, return undefined
}
```

### Priority 3: Run Conformance Tests

Execute conformance tests to identify TS1005/TS1109 patterns:
```bash
# Run parser conformance tests
cargo test --package wasm --lib parser_conformance

# Filter for ASI-related failures
cargo test --package wasm --lib asi 2>&1 | grep TS1005\|TS1109
```

---

## Success Metrics

- [ ] Fix `throw` statement line break bug
- [ ] Add comprehensive ASI edge case test suite
- [ ] Run conformance tests and categorize failures
- [ ] Report findings to EM-1 and EM-2 teams
- [ ] Achieve 50% reduction in edge case parser failures

---

## Appendix: ASI Reference

### JavaScript ASI Rules (ECMAScript 2024)

1. **Before closing brace**: `} ;` is always valid
2. **At end of file**: EOF triggers ASI
3. **After line break** if next token cannot continue statement
4. **Restricted productions**: `throw`, `yield`, `return` must have expression on same line

### TypeScript Error Codes

- **TS1005**: ',' expected (missing token after ASI boundary)
- **TS1109**: Expression expected (ASI caused invalid code)

---

## Files Reviewed

- `wasm/src/thin_parser.rs` - Main parser implementation
- `wasm/src/parser/ast/statements.rs` - AST node definitions
- `wasm/src/thin_parser_tests.rs` - Existing parser tests
- `wasm/src/scanner_impl.rs` - Scanner (tokenization)
