# TS1109 ("Expression expected") Analysis

## Summary
Audit of TS1109 emission points in `wasm/src/thin_parser.rs`. Goal: Identify edge cases causing 262 false positives.

---

## Emission Points (4 Total)

### 1. Variable Declaration Initializer (Line 1316)
**Function:** `parse_variable_declaration`
**Condition:** After `=`, next token is `const`, `let`, or `var`
**Example:** `let x = const;`
**Risk:** LOW - Specific keyword check, valid error

### 2. Primary Expression Fallback (Line 6312) - **MAIN SOURCE**
**Function:** `parse_primary_expression`
**Condition:** Token is not valid primary expression starter AND not identifier/keyword
**Example:** Any unexpected token in expression position

**Call chain to this point:**
```
parse_assignment_expression -> parse_binary_expression(2)
  -> parse_unary_expression -> parse_postfix_expression
    -> parse_left_hand_side_expression -> parse_primary_expression
```

**Handled tokens (no TS1109):**
- Identifier, PrivateIdentifier
- Literals: NumericLiteral, BigIntLiteral, StringLiteral, NoSubstitutionTemplateLiteral, TemplateHead
- Keywords: true, false, null, undefined, this, super, new, function, class, async, import
- Brackets: (, [, {
- LessThanToken (JSX/type assertion)
- SlashToken (regex)
- Type keywords: any, string, number, boolean, symbol, bigint, object, never, unknown, require, module
- **Fallback: Any token >= Identifier (all keywords treated as identifiers)**

**TS1109 triggers for:**
- EndOfFile, CloseBrace, CloseParen, CloseBracket
- Semicolon, Comma, Colon, Dot
- Assignment operators
- Binary operators in wrong position

### 3. New Expression Type Assertion (Line 7499)
**Function:** `parse_new_expression`
**Condition:** `<` token immediately after `new`
**Example:** `new <T>Foo()` (invalid - type assertion not allowed here)
**Risk:** LOW - This is a valid TypeScript error

### 4. JSX Attribute Value (Line 9608)
**Function:** `parse_jsx_attribute`
**Condition:** After `=`, value is not string, brace `{`, or `<`
**Example:** `<div attr=123>` (invalid JSX)
**Risk:** MEDIUM - Edge cases possible

---

## False Positive Patterns

### Pattern A: Error Recovery Cascading
When TS1005 ("expected X") fires incorrectly, parser may skip to unexpected position, causing TS1109 cascade.

**Example scenario:**
```typescript
const x = {
  method(): void  // Missing semicolon or comma
  other() {}
}
// TS1005 fires, parser skips, TS1109 may fire at wrong location
```

### Pattern B: Context-Sensitive Tokens
Some tokens valid in certain contexts may reach `parse_primary_expression` through error recovery.

### Pattern C: Complex Generic/Type Syntax
Deeply nested generics or type assertions may confuse error recovery.

---

## Diagnosis Strategy

1. **High correlation with TS1005:** With 439 TS1005 false positives, many TS1109 may be cascades
2. **Focus on error recovery:** Most false positives likely from recovery paths
3. **Position tracking:** TS1109 at same position as TS1005 suggests cascade

---

## Recommendations

### Immediate (High Impact)
1. Fix TS1005 false positives first - may reduce TS1109 cascades
2. Add position deduplication - don't emit TS1109 if TS1005 just fired at same position

### Medium Term
3. Improve error recovery to skip to safer positions
4. Add guard checks before `parse_primary_expression` calls

### Investigation Needed
5. Run conformance tests with logging to identify specific failing tests
6. Categorize false positives by triggering pattern

---

## Files Changed
- `wasm/src/thin_parser.rs:361-367` - `error_expression_expected()` helper
- `wasm/src/thin_parser.rs:1316` - Variable declaration
- `wasm/src/thin_parser.rs:6312` - Primary expression (MAIN)
- `wasm/src/thin_parser.rs:7499` - New expression
- `wasm/src/thin_parser.rs:9608` - JSX attribute

---

## Next Steps for Queue Tasks

1. **Create test case list:** Run conformance, filter tests with TS1109 extra errors
2. **Fix top 5 patterns:** Focus on error recovery and position deduplication
3. **Verify:** Re-run conformance to measure reduction from 262
