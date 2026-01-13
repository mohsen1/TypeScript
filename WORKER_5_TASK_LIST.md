# Worker 5 Task List

## Squad: Binder (CRITICAL)

## Current Task
- [ ] Debug why basic globals like console fail to resolve in wasm/src/binder

## Queue
- [ ] Trace resolution path for console in a simple test case
- [ ] Fix global scope chain to include browser/node globals
- [ ] Verify fix with console.log resolution test

## Completed
(none yet)

## Context
Basic globals like console and Array fail to resolve. This is a critical bug causing massive error poisoning.
