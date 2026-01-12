# Worker 5 Plan - Squad Anvil

## Mission
Fix Parser Error Recovery

Status: Active
Priority: P2 (Medium)

## Current Assignment
**Fix Parser Error Recovery (3 tests)**

### Failing Tests
1. `test_thin_parser_function_keyword_in_class_recovers`
2. `test_thin_parser_jsx_like_syntax_in_ts_recovers`
3. `test_thin_parser_type_assertion_in_new_expression_reports_ts1109`

### Implementation Steps - Parser Recovery (Medium)

1. [ ] Read failing parser tests in `src/thin_parser_tests.rs`
2. [ ] For `function_keyword_in_class_recovers`:
   - Parser should produce error but continue parsing
   - Skip to next valid class member (find `}` or next modifier)
3. [ ] For `jsx_like_syntax_in_ts_recovers`:
   - `<Type>expr` is ambiguous with JSX
   - In .ts files, treat as type assertion
   - Should not crash, should produce error
4. [ ] For `type_assertion_in_new_expression_reports_ts1109`:
   - `new <Type>expr` should report TS1109 "Expression expected"
   - The type assertion syntax is invalid in `new` expressions

### Key Code Locations
- `src/thin_parser.rs` - error recovery, synchronization points
- `src/thin_parser_tests.rs` - failing tests

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
- [ ] After parser recovery: help with CLI driver issues or emitter edge cases as needed

## Completed
- [x] Control Flow: Private identifier in `in` operator narrowing (MERGED to squad/anvil)
  - Fixed in worker/anvil-5 branch, merged to squad/anvil
  - Test passes: test_in_operator_private_identifier_narrows_required_property

## Ready for Merge
Yes - Merged to squad/anvil, pushed to origin/squad/anvil

## Notes
- Follow `wasm/specs/WASM_ARCHITECTURE.md`
- Use Docker for Rust tests: `./wasm/test.sh`
- Commit format: `[wasm] control_flow: Handle private identifier in 'in' operator`
- Sync before each task: `git fetch origin && git merge origin/rust --no-edit`
- Push to: `origin/worker/anvil-5`
