# Worker 7 Task List

## Squad: Parser - TS1005/TS1109 False Positives

## Current Task
- [ ] Audit and fix TS1109 "expression expected" false positives (262 occurrences)
- [ ] Continue TS1005 fixes from where Worker 1 left off (patterns 6+)

## Queue
- [ ] Implement better error recovery/resynchronization in `src/thin_parser.rs`
- [ ] Reduce parser false positives from 701 to <100 total
- [ ] Test parser changes on conformance suite to measure false positive reduction
- [ ] Coordinate with Worker 1 to avoid duplicate work on TS1005

## Completed
- None

## Context
Parser errors are polluting all measurements with 701 false positives:
- TS1005 ("expected X"): 439 occurrences (Worker 1 has patterns 1-5 fixed)
- TS1109 ("expression expected"): 262 occurrences (needs investigation)

When parsing fails, the AST is incomplete, causing missing symbols → TS2304 → "Any" poisoning.

### Key Files
- `src/thin_parser.rs` - main parser implementation
- `src/parser/scanner.rs` - lexical scanner

### Goal
Reduce parser false positives from 701 to <100 to unblock accurate conformance measurement.
