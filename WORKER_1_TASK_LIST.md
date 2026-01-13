# Worker 1 Task List

## Squad: Parser (Syntax)

## Current Task
- [ ] Audit TS1005 ("expected X") emission in wasm/src/parser - identify where it over-triggers on valid syntax

## Queue
- [ ] Create a list of specific test cases where TS1005 fires incorrectly
- [ ] Fix the top 5 most common TS1005 false positive patterns
- [ ] Run conformance tests to verify TS1005 reductions

## Completed
(none yet)

## Context
TS1005 has 439 false positives. This is polluting all measurements. Focus on the parser error emission logic in wasm/src/parser.
