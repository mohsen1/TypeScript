# Worker 3 Task List

## Squad: Parser (Syntax)

## Current Task
- [ ] Review parser error recovery logic in wasm/src/parser - ensure it doesn't emit spurious errors after recovery

## Queue
- [ ] Identify patterns where parser emits multiple errors for single syntax issue
- [ ] Fix cascading error emission to stop after first meaningful error
- [ ] Coordinate with Workers 1 & 2 on remaining parser false positives

## Completed
(none yet)

## Context
Goal is to reduce parser false positives from 701 to <100. Focus on error recovery and cascading error prevention.
