# Worker 7 Task List

## Squad: Parser/Scanner - TS1109 Focus

## Current Task
- [ ] Continue auditing TS1109 "expression expected" false positives (262 occurrences)
- [ ] Test cascading error fix on conformance suite to measure impact

## Queue
- [ ] Fix remaining TS1109 false positive patterns
- [ ] Improve error recovery/resynchronization in `wasm/src/thin_parser.rs`
- [ ] Reduce parser false positives from 701 to <100 total

## Completed
- [x] **TS1109 Cascading Error Fix** - Prevent duplicate errors at same position
- [x] Merged to em-team-2 (commit: `bf68ce47a`)

### Changes Made
- `wasm/src/thin_parser.rs`: Added check to prevent TS1109 emission when another error was already reported at the same position
- This reduces cascading errors that pollute measurements

## Context
TS1109 has 262 false positive occurrences. Worker 7 has implemented the first fix to prevent cascading errors.

### Key Files
- `wasm/src/thin_parser.rs` - main parser implementation

### Reference
- `TS1109_ANALYSIS.md` - TS1109 analysis

### Goal
Reduce TS1109 errors from 262 to <50 through iterative fixes.
