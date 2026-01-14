# Worker 8 Task List

## Squad: Parser/Scanner - Error Recovery Focus

## Current Task - Statement-Level Resynchronization

### Task 1: Statement Recovery Enhancement
**Problem:** When `parse_statement()` encounters an error, it may not properly resynchronize to the next statement, causing cascading errors throughout the block.

**Solution:** Implement statement-level resynchronization that skips to the next statement start after encountering a parse error.

**Implementation Locations:**
- `wasm/src/thin_parser.rs:716` - `parse_statement()`
- `wasm/src/thin_parser.rs:695` - `parse_statements()`

**Strategy:**
1. After emitting an error in `parse_statement()`, check if we can resynchronize
2. Look ahead for statement start tokens:
   - Keywords: `function`, `class`, `if`, `for`, `while`, `do`, `switch`, `try`, `var`, `let`, `const`, `return`, `break`, `continue`, `throw`, `import`, `export`, `type`, `interface`, `enum`, `namespace`
   - Identifiers (for expression statements)
   - `{` (block statements)
3. If current token can't start a statement, consume tokens until we find one that can
4. Add `fn can_start_statement(&self) -> bool` helper method

**Reference Pattern:**
```rust
fn parse_statement(&mut self) -> NodeIndex {
    let start = self.token_pos();

    match self.token() {
        SyntaxKind::OpenBraceToken => self.parse_block(),
        // ... other statement types ...
        _ => {
            // Error: unexpected token
            self.error_unexpected_token();

            // RESYNCHRONIZE: Skip to next statement
            self.resynchronize_to_next_statement();

            // Return empty statement node to continue parsing
            self.create_missing_node(SyntaxKind::ExpressionStatement, start)
        }
    }
}

fn resynchronize_to_next_statement(&mut self) {
    while !self.is_at_end() && !self.can_start_statement() {
        self.next_token();
    }
}

fn can_start_statement(&self) -> bool {
    match self.token() {
        // Keywords that start statements
        SyntaxKind::FunctionKeyword |
        SyntaxKind::ClassKeyword |
        SyntaxKind::IfKeyword |
        SyntaxKind::ForKeyword |
        SyntaxKind::WhileKeyword |
        SyntaxKind::DoKeyword |
        SyntaxKind::SwitchKeyword |
        SyntaxKind::TryKeyword |
        SyntaxKind::VarKeyword |
        SyntaxKind::LetKeyword |
        SyntaxKind::ConstKeyword |
        SyntaxKind::ReturnKeyword |
        SyntaxKind::BreakKeyword |
        SyntaxKind::ContinueKeyword |
        SyntaxKind::ThrowKeyword |
        SyntaxKind::ImportKeyword |
        SyntaxKind::ExportKeyword |
        SyntaxKind::TypeKeyword |
        SyntaxKind::InterfaceKeyword |
        SyntaxKind::EnumKeyword |
        SyntaxKind::NamespaceKeyword |
        SyntaxKind::OpenBraceToken |
        SyntaxKind::Identifier => true,
        _ => false,
    }
}
```

### Task 2: Block-Level Recovery
**Problem:** When parsing statements inside a block (`parse_block`), errors in one statement can cause the parser to fail to parse subsequent statements.

**Solution:** Ensure `parse_statements()` continues parsing after errors by catching failures and resynchronizing.

**Implementation Location:**
- `wasm/src/thin_parser.rs:695-706` - `parse_statements()`

**Current Code Pattern:**
```rust
fn parse_statements(&mut self) -> NodeList {
    let mut statements = NodeList::default();
    while self.token() != SyntaxKind::EndOfFileToken && self.token() != SyntaxKind::CloseBraceToken {
        statements.push(self.parse_statement());
    }
    statements
}
```

**Enhanced Version:**
```rust
fn parse_statements(&mut self) -> NodeList {
    let mut statements = NodeList::default();
    while self.token() != SyntaxKind::EndOfFileToken && self.token() != SyntaxKind::CloseBraceToken {
        // Safety: catch infinite loops
        if self.token_pos() == self.last_error_pos {
            // Already emitted error here and couldn't recover
            break;
        }
        statements.push(self.parse_statement());
    }
    statements
}
```

## Queue
- [ ] Test parser changes on conformance suite to measure impact
- [ ] Coordinate with Workers 5-7 on error emission patterns
- [ ] Measure reduction in cascading errors
- [ ] Document recovery patterns for future workers

## Completed
- [x] Merge attempt #2 - No commits to merge yet

## Context
Error recovery is critical to prevent one syntax error from poisoning the entire file. When the parser bails early, the incomplete AST leads to missing symbols and cascading errors.

### Key Insight
Worker 7's TS1109 fix showed that tracking `last_error_pos` and avoiding duplicate emissions is effective. Apply similar pattern to statement-level recovery.

### Key Files
- `wasm/src/thin_parser.rs` - main parser implementation
- `src/compiler/parser.ts:3400-3500` - TypeScript's `parseStatement()` with recovery

### Success Metrics
- Fewer "Extra Errors" in conformance tests
- More complete ASTs (fewer missing symbols due to parse failures)
- Parser continues after syntax errors instead of bailing

### Goal
Improve parser resilience so it can recover from syntax errors and continue parsing, reducing incomplete ASTs and cascading errors.
