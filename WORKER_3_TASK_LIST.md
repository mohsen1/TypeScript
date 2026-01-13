# Worker 3 Task List

## Squad: Parser Squad (Error Recovery)

## Current Task
- [ ] Identify cases where error recovery causes cascading TS1005/TS1109 false positives

## Queue
- [ ] Audit scanner token classification for edge cases (JSX, template literals, regex) in `wasm/src/scanner_impl.rs`
- [ ] Fix scanner-level issues that propagate into parser errors
- [ ] Test parser robustness with malformed input (ensure no crashes)

## Completed
- [x] Review parser error recovery mechanisms in `wasm/src/thin_parser.rs` for cascading false positive patterns

## Context
- **Goal:** Support Parser Squad in reducing false positives to <100
- **Key files:** `wasm/src/thin_parser.rs`, `wasm/src/scanner.rs`, `wasm/src/scanner_impl.rs`
- **Note:** Error recovery is working well - focus on preventing false positive cascades
