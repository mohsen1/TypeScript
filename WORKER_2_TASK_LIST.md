# Worker 2 Task List

## Squad: Parser Squad (TS1109 Focus)

## Current Task
- [ ] Audit TS1109 ("expression expected") emission points in `wasm/src/thin_parser.rs` - identify all emission locations

## Queue
- [ ] Analyze conformance test failures with false positive TS1109 - identify edge case patterns
- [ ] Fix TS1109 false positives on valid syntax (target: reduce 262 occurrences)
- [ ] Review expression parsing in `parse_expression` and `parse_unary_expression` for incorrect triggering
- [ ] Add regression tests for fixed edge cases

## Completed
(Previous phase work archived)

## Context
- **Goal:** Reduce parser false positives from 701 to <100
- **Key file:** `wasm/src/thin_parser.rs`
- **Error code:** TS1109 currently has 262 false positives
- **Impact:** These parser errors pollute conformance measurements
