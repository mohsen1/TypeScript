# Worker 10 Task List

## Squad: Solver (Semantics)

## Current Task
- [ ] Switch solver default fallback from Any to Unknown in wasm/src/solver - expose hidden bugs

## Queue
- [ ] Find all places where solver returns Any as fallback
- [ ] Change fallback to Unknown or Error type
- [ ] Document new errors exposed by this change

## Completed
(none yet)

## Context
The biggest enemy is Error Poisoning. When resolution fails, the Solver says "it's Any" which silences all downstream errors. We need to be strict.
