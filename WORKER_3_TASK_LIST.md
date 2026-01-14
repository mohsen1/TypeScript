# Worker 3 Task List

## Squad: Parser (Syntax)

## Current Task (Assigned by EM-1)
- [ ] **IMMEDIATE:** Run conformance tests to measure TS1005/TS1109 reduction from cascading error fix
- [ ] Document exact error counts before and after `last_error_pos` implementation
- [ ] Report metrics to EM-1 for validation before proceeding

## Queue (On Hold - Awaiting Baseline)
- [ ] Coordinate with Workers 1 & 2 on remaining parser false positives
- [ ] Address any remaining cascading error patterns not covered by the fix
- [ ] Review and fix parser error emission in edge cases (e.g., ASI failures, type parameters)

## Completed
- [x] Identify patterns where parser emits multiple errors for single syntax issue
- [x] Fix cascading error emission to stop after first meaningful error
- [x] Review parser error recovery logic in wasm/src/parser - ensure it doesn't emit spurious errors after recovery
- [x] Implement position-based error tracking (last_error_pos field)
- [x] Update parse_expected() and parse_expected_greater_than() to check last_error_pos

## Analysis Findings

### Error Recovery Mechanisms in thin_parser.rs

1. **parse_expected() - Line 298**
   - Reports TS1005 ("'X' expected") when token not found
   - Does NOT consume the unexpected token after error
   - This can cause cascading errors when multiple parse_expected calls are made sequentially

2. **Cascading Error Patterns Identified**

   **Pattern 1: Multiple Sequential parse_expected() Calls**
   - Example: `parse_external_module_reference()` (lines 1124-1132)
   - If first fails, all subsequent parse_expected may also fail, emitting multiple TS1005

   **Pattern 2: Block Parsing (lines 1175-1200)**
   - If OpenBraceToken missing, error emitted but parsing continues
   - Then CloseBraceToken also fails, resulting in two TS1005 errors

   **Pattern 3: For Statement (lines 4807-4871)**
   - Has 5 parse_expected calls: ForKeyword, OpenParen, two SemicolonTokens, CloseParen
   - Missing one token can cascade to multiple errors

   **Pattern 4: Object Literal (lines 7029-7057)**
   - When comma missing, loop breaks instead of attempting recovery
   - Can lead to ",' expected", "':' expected", "'}' expected" for one issue

   **Pattern 5: If Statement (lines 4728-4756)**
   - Has 3 parse_expected calls: IfKeyword, OpenParenToken, CloseParenToken
   - Missing IfKeyword or OpenParenToken will cascade to CloseParenToken error

   **Pattern 6: While Statement (lines 4781-4804)**
   - Has 3 parse_expected calls: WhileKeyword, OpenParenToken, CloseParenToken
   - Missing WhileKeyword or OpenParenToken will cascade to CloseParenToken error

   **Pattern 7: Do-While Statement (lines 5062-5087)**
   - Has 4 parse_expected calls: DoKeyword, WhileKeyword, OpenParenToken, CloseParenToken
   - Missing WhileKeyword or OpenParenToken will cascade to CloseParenToken error

   **Pattern 8: Switch Statement (lines 5090-5183)**
   - Has 5 parse_expected calls: SwitchKeyword, OpenParenToken, CloseParenToken, OpenBraceToken, CloseBraceToken
   - Missing any of the first 4 will cascade to subsequent errors

   **Pattern 9: Try-Catch Statement (lines 5186-5246)**
   - Has parse_expected(TryKeyword) followed by parse_expected(CloseParenToken) in catch clause
   - If OpenParenToken missing, CloseParenToken error will follow

   **Pattern 10: With Statement (lines 5248-5273)**
   - Has 3 parse_expected calls: WithKeyword, OpenParenToken, CloseParenToken
   - Missing OpenParenToken will cascade to CloseParenToken error

   **Pattern 11: Type Parameters (lines 5622-5639)**
   - Has parse_expected(LessThanToken) and parse_expected_greater_than()
   - Missing < will cause loop to fail to find proper end, leading to missing > error

   **Pattern 12: Array Literal (lines 6977-7026)**
   - Has parse_expected(OpenBracketToken) and parse_expected(CloseBracketToken)
   - Missing OpenBracketToken will cause CloseBracketToken to also fail

   **Pattern 13: Object Get Accessor (lines 7193-7240)**
   - Has parse_expected(OpenParenToken) and parse_expected(CloseParenToken)
   - Missing OpenParenToken will cascade to CloseParenToken error

   **Pattern 14: Arrow Function Expression (lines 5525-5620)**
   - Has parse_expected(OpenParenToken), parse_expected(CloseParenToken), parse_expected(EqualsGreaterThanToken)
   - Missing OpenParenToken will cascade to CloseParenToken and EqualsGreaterThanToken errors

   **Pattern 15: Import Declaration (parse_import_declaration)**
   - Multiple parse_expected calls for OpenBraceToken, CloseBraceToken, FromKeyword, StringLiteral
   - Missing one token cascades to multiple TS1005 errors

   **Pattern 16: Heritage Clauses (lines 2103-2201)**
   - In extends clause: missing comma triggers "Classes can only extend a single class" but parsing continues
   - Can cascade to multiple errors for single missing delimiter

3. **Good Error Recovery Patterns Found**
   - JSX element name parsing (lines 9468-9491): Returns missing node without cascading
   - JSX attribute parsing (lines 9574-9595): Skips invalid token and returns dummy attribute
   - Class member recovery (lines 2703+): Handles stray statements gracefully

4. **No Error Suppression Mechanism**
   - ThinParserState has no field to track "in recovery mode"
   - All errors are emitted regardless of prior errors at same location
   - Could benefit from error deduplication or recovery-mode flag

### Recommendations for Fix

1. **Add recovery mode flag** to suppress cascading errors
   - Add `last_error_pos: u32` field to ThinParserState
   - In parse_expected(), skip emitting error if token_pos() == last_error_pos

2. **Modify parse_expected to check error position** - skip if error was just emitted at same position

3. **Add smarter comma recovery** in object/array literals

4. **Consider syncing to statement boundaries** after errors

5. **Create parse_expected_seq() helper** for sequential tokens
   - Takes array of expected tokens
   - Stops after first error to prevent cascading
   - Use for: if/while/switch statements, type parameters, etc.

6. **Add "soft" parse_expected variant** that only errors if not in recovery mode
   - `parse_expected_in_recovery()` - suppresses error if already emitted at current position
   - Replace critical parse_expected calls with this variant

7. **Prioritize statement-level fixes first**
   - Control flow statements (if, while, for, switch) are most common
   - Fixing these will have biggest impact on false positive count

## Implementation Status

### Fix Implementation (cascading error suppression)
- [x] Add `last_error_pos: u32` field to ThinParserState
- [x] Update `new()` and `reset()` to initialize `last_error_pos`
- [x] Update `parse_error_at()` to track `last_error_pos`
- [x] Update `parse_expected()` to check `last_error_pos` before emitting errors
- [x] Update `parse_expected_greater_than()` to check `last_error_pos`
- [x] Build verified - code compiles with only minor warnings

## Context
Goal is to reduce parser false positives from 701 to <100. Focus on error recovery and cascading error prevention.
