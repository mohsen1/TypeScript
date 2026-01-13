# Worker 3 Task List

## Squad: Parser (Syntax)

## Current Task
- [ ] Identify patterns where parser emits multiple errors for single syntax issue

## Queue
- [ ] Fix cascading error emission to stop after first meaningful error
- [ ] Coordinate with Workers 1 & 2 on remaining parser false positives

## Completed
- [x] Review parser error recovery logic in wasm/src/parser - ensure it doesn't emit spurious errors after recovery

## Analysis Findings

### Error Recovery Mechanisms in thin_parser.rs

1. **parse_expected() - Line 298**
   - Reports TS1005 ("'X' expected") when token not found
   - Does NOT consume the unexpected token after error
   - This can cause cascading errors when multiple parse_expected calls are made sequentially

2. **Cascading Error Patterns Identified**

   **Pattern 1: Multiple Sequential parse_expected() Calls**
   - Example: `parse_external_module_reference()` (lines 1124-1132)
     ```rust
     parse_expected(SyntaxKind::RequireKeyword);
     parse_expected(SyntaxKind::OpenParenToken);
     // ...
     parse_expected(SyntaxKind::CloseParenToken);
     ```
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

3. **Good Error Recovery Patterns Found**
   - JSX element name parsing (lines 9468-9491): Returns missing node without cascading
   - JSX attribute parsing (lines 9574-9595): Skips invalid token and returns dummy attribute
   - Class member recovery (lines 2703+): Handles stray statements gracefully

4. **No Error Suppression Mechanism**
   - ThinParserState has no field to track "in recovery mode"
   - All errors are emitted regardless of prior errors at same location
   - Could benefit from error deduplication or recovery-mode flag

### Recommendations for Fix

1. **Add recovery mode flag** to suppress cascading errors:
   ```rust
   error_position: Option<u32>,  // Track position of last error to avoid duplicates
   ```

2. **Modify parse_expected to check error position**:
   - If error was just emitted at same position, skip emitting another

3. **Add smarter comma recovery in object/array literals**:
   - Before breaking, check if next token starts a valid property
   - Emit one error and continue parsing

4. **Consider syncing to statement boundaries after errors**:
   - Skip to next `;`, `}`, or keyword to reduce cascading

## Context
Goal is to reduce parser false positives from 701 to <100. Focus on error recovery and cascading error prevention.
