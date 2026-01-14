# Worker 8 Task List

## Squad: Parser/Scanner - Error Recovery Focus

## Current Task
- [ ] Improve parser error recovery and resynchronization in `wasm/src/thin_parser.rs`
- [ ] Implement better resynchronization after syntax errors
- [ ] Keep parsing to complete the AST even with errors

## Queue
- [ ] Reduce cascading errors through better recovery
- [ ] Test parser changes on conformance suite
- [ ] Coordinate with Workers 5-7 on error emission patterns
- [ ] Ensure parser doesn't bail early on syntax deviations

## Completed
- [x] **Task 1 & 2:** Statement-level error recovery (commit: `aac4ace7c`)
- [x] **Task 3:** Error recovery verification tests (commit: `934ac45c2`)
- [x] **Merge to em-team-2:** commit `3aee5a746`
- [x] **Test Results:** 5/5 tests passed (100%)
- [x] **Conformance:** 33.1% exact match (unchanged - error recovery helps within files)
- [x] **Latest Merge Check:** worker-8 branch is out of sync (behind em-team-2)
  - em-team-2 has Worker 7's TS1128 deduplication
  - worker-8 branch doesn't have these fixes
  - No merge needed - em-team-2 already has all Worker 8 work

---

- [x] **Task 1: Statement Recovery Enhancement** - Implemented is_statement_start() and resync_after_error()
- [x] **Task 2: Block-Level Recovery** - Enhanced parse_source_file_statements() and parse_statements() with resync
- [x] Merged to em-team-2 (commit: `aac4ace7c`)
- [x] **Conformance Test Results (1000 tests):**
  - Exact Match: 33.1% (unchanged - error recovery helps within files, not across test boundaries)
  - Throughput: 15.6 tests/sec
  - **Note:** Error recovery prevents cascading errors within complex files, improving LSP experience and AST completeness
  - Real benefit is fewer incomplete ASTs and better error recovery in multi-statement blocks

- [x] Merge attempt #2 - No commits to merge yet

## Context
Error recovery is critical to prevent one syntax error from poisoning the entire file. When the parser bails early, the incomplete AST leads to missing symbols and cascading errors.

### Key Files
- `wasm/src/thin_parser.rs` - main parser implementation

### Goal
Improve parser resilience so it can recover from syntax errors and continue parsing, reducing incomplete ASTs and cascading errors.
