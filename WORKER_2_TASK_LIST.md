# Worker 2 Task List

## Squad: Parser (Syntax)

## Current Task
- [ ] Audit TS1109 ("expression expected") false positives in wasm/src/parser - identify edge cases triggering incorrectly

## Queue
- [ ] Create a list of specific test cases where TS1109 fires incorrectly
- [ ] Fix the top 5 most common TS1109 false positive patterns
- [ ] Run conformance tests to verify TS1109 reductions

## Completed
(none yet)

## Context
TS1109 has 262 false positives. These parser errors mask real progress. Focus on expression parsing edge cases.
