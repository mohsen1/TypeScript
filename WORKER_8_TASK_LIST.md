# Worker 8 Task List

## Squad: Binder (CRITICAL)

## Current Task
- [ ] Implement proper scope chain traversal for nested declarations in wasm/src/binder

## Queue
- [ ] Audit how nested scopes (functions inside functions, class methods) resolve outer variables
- [ ] Fix scope chain to correctly walk up to parent scopes
- [ ] Add tests for nested scope resolution

## Completed
(none yet)

## Context
Scope resolution bugs are causing TS2304 errors. The binder needs to correctly traverse the scope chain from inner to outer scopes.
