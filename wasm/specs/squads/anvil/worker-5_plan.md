# Worker 5 Plan - Squad Anvil

## Mission
Fix Parser Error Recovery and Private Identifier Control Flow

Status: Active
Priority: P2 (Medium)

## Current Assignment
**Fix Parser Error Recovery (3 tests) + Control Flow (1 test)**

### Failing Tests - Parser
1. ~~`test_thin_parser_function_keyword_in_class_recovers`~~ ✅ FIXED
2. ~~`test_thin_parser_jsx_like_syntax_in_ts_recovers`~~ ✅ FIXED
3. ~~`test_thin_parser_type_assertion_in_new_expression_reports_ts1109`~~ ✅ FIXED

### Failing Test - Control Flow
4. ~~`test_in_operator_private_identifier_narrows_required_property`~~ ✅ FIXED

### Implementation Steps - Control Flow (Quick Win)

1. [x] Read the failing test in `src/checker/control_flow_tests.rs`
2. [x] Find `narrow_type_by_condition` in `src/checker/control_flow.rs`
3. [x] Add handling for `PrivateIdentifier` in `in` operator narrowing:
   ```rust
   // In narrow_type_by_in_expression or similar
   SyntaxKind::PrivateIdentifier => {
       // #field in obj should narrow like regular "field" in obj
       let field_name = get_private_identifier_name(left);
       // ... narrow to types that have this private field
   }
   ```
4. [x] Test: `./wasm/test.sh 2>&1 | grep -E "private_identifier_narrows"`

### Implementation Steps - Parser Recovery (Medium)

5. [x] Read failing parser tests in `src/thin_parser_tests.rs`
6. [x] For `function_keyword_in_class_recovers`:
   - Parser should silently skip 'function' keyword without emitting TS1068
   - Parser recovers gracefully and parses the rest as a method
7. [x] For `jsx_like_syntax_in_ts_recovers`:
   - `<Type>expr` is ambiguous with JSX
   - In .ts files, always try to parse as type assertion first
   - This produces appropriate TS1005 error for invalid JSX-like syntax
8. [x] For `type_assertion_in_new_expression_reports_ts1109`:
   - `new <Type>expr` should report TS1109 "Expression expected"
   - The type assertion syntax is invalid in `new` expressions

### Key Code Locations
- `src/checker/control_flow.rs` - type narrowing
- `src/thin_parser.rs` - error recovery, synchronization points

### Parser Recovery Strategy
```rust
// After detecting error, skip to synchronization point
fn recover_to_next_member(&mut self) {
    while !self.at_end() {
        match self.current() {
            SyntaxKind::CloseBraceToken |
            SyntaxKind::PublicKeyword |
            SyntaxKind::PrivateKeyword |
            SyntaxKind::ProtectedKeyword |
            SyntaxKind::StaticKeyword => break,
            _ => self.advance(),
        }
    }
}
```

## Task Queue
- [ ] Start with control flow (quick win), then parser recovery

## Completed
- [x] (Move finished items here)

## Ready for Merge
Yes - All 4 tests fixed:
- Control flow: private identifier in 'in' operator
- Parser: function keyword in class recovery
- Parser: JSX-like syntax in .ts files
- Parser: type assertion in new expression

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] control_flow: Handle private identifier in 'in' operator`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-5`
