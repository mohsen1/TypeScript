# Worker 7 Task List

## Squad: Binder (CRITICAL)

## Current Task
- [ ] Investigate TS2304 extra errors (343 occurrences) - find patterns in false positives

## Queue
- [ ] Categorize TS2304 extra errors by type (globals, imports, locals, etc.)
- [ ] Fix the most common category of false positives
- [ ] Target reduction from 343 to <150

## Completed
(none yet)

## Context
TS2304 is both missing (116) AND extra (343). The extra errors indicate the binder is incorrectly failing to find symbols that should be in scope.
