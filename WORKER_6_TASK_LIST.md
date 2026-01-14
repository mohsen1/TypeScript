# Worker 6 Task List

## Squad: Parser/Scanner - TS1005 Focus

## Current Task
- [ ] Audit TS1005 emission patterns in `wasm/src/thin_parser.rs`
- [ ] Find all locations where "expected X" errors are emitted
- [ ] Compare with tsc behavior on same test cases

## Queue
- [ ] Fix TS1005 false positives (439 occurrences)
- [ ] Implement better error recovery in object literal parsing
- [ ] Test parser changes on conformance suite
- [ ] Coordinate with Worker 5 to avoid duplicate work

## Completed
- None

## Context
TS1005 has 439 false positive occurrences. Multiple workers needed to tackle this from different angles.

### Key Files
- `wasm/src/thin_parser.rs` - main parser implementation
- `wasm/src/scanner.rs` - lexical scanner

### Reference
- `src/compiler/parser.ts` - Worker 1's TS1005 fixes (TypeScript)
- `TS1005_REDUCTION_RESULTS.md` - Pattern analysis

### Goal
Reduce TS1005 errors from 439 to <50 through parallel work with Worker 5.
