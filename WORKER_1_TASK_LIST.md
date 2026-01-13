# Worker 1 Task List

## Squad: Parser Squad (TS1005 Focus)

## Current Task
- [ ] Audit TS1005 ("expected X") emission points in `wasm/src/thin_parser.rs` - identify all locations emitting this error

## Queue
- [ ] Analyze conformance test failures with false positive TS1005 - find patterns causing over-triggering
- [ ] Fix TS1005 false positives on valid TypeScript syntax (target: reduce 439 to <50)
- [ ] Review `expect_token()` and `parse_expected()` methods for incorrect triggering
- [ ] Add regression tests for edge cases where TS1005 should NOT be emitted

## Completed
(Previous phase work archived)

## Context
- **Goal:** Reduce parser false positives from 701 to <100
- **Key file:** `wasm/src/thin_parser.rs`
- **Error code:** TS1005 currently has 439 false positives
- **Impact:** Parser errors mask real progress and inflate "Extra Errors" by 14%
