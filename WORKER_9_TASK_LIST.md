# Worker 9 Task List

## Squad: Binder (CRITICAL)

## Current Task
- [ ] Fix type-only imports and exports handling in wasm/src/binder

## Queue
- [ ] Audit how import type and export type are processed
- [ ] Ensure type-only imports create proper symbol bindings
- [ ] Fix re-export chains for type declarations

## Completed
(none yet)

## Context
Type imports and exports may not be creating proper bindings, leading to TS2304 errors when referencing imported types.
