# Worker 4 Task List

## Squad: Binder (CRITICAL)

## Current Task
- [ ] Investigate lib.d.ts loading and symbol merging into root SymbolTable in wasm/src/binder

## Queue
- [ ] Trace how lib.d.ts symbols (Array, Promise, console) are supposed to be injected
- [ ] Fix symbol table merging to correctly expose global types
- [ ] Add tests verifying lib.d.ts globals are resolvable

## Completed
(none yet)

## Context
TS2304 (Cannot find name) is the #1 source of error poisoning. When the Binder fails to resolve Promise, the Solver defaults to Any, suppressing all downstream errors.
