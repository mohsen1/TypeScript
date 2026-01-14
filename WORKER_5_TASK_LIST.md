# Worker 5 Task List

## Squad: Parser/Scanner - TS1005 Focus

## Current Task
- [ ] Audit TS1005 "expected X" emission patterns in `wasm/src/thin_parser.rs`
- [ ] Identify where "expected X" is over-triggering on valid syntax
- [ ] Reference Worker 1's TypeScript patterns in `src/compiler/parser.ts` and adapt to Rust

## Queue
- [ ] Fix TS1005 false positives (439 occurrences)
- [ ] Consolidate error emission to avoid duplicates
- [ ] Test parser changes on conformance suite to measure reduction
- [ ] Coordinate with Worker 6 to avoid duplicate work

## Completed
- [x] Initial merge attempt - No commits yet

## Context
TS1005 is the #1 source of parser false positives (439 occurrences). These pollute all measurements and inflate "Extra Errors" by 14%.

### Key Files
- `wasm/src/thin_parser.rs` - main parser implementation
- `wasm/src/scanner.rs` - lexical scanner

### Reference
- `src/compiler/parser.ts` - Worker 1's TS1005 fixes (TypeScript)
- `TS1005_REDUCTION_RESULTS.md` - Pattern analysis

### Goal
Reduce TS1005 errors from 439 to <50 through iterative fixes.
