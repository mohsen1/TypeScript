# Worker 7 Task List

## Squad: Binder (CRITICAL)

## Current Task
- [ ] Fix identified TS2304 extra error patterns from analysis

## Queue
- [ ] Fix symbol table scope chain traversal for nested scopes
- [ ] Fix import binding for re-exported symbols
- [ ] Debug why basic globals like console and Array still fail to resolve in some cases
- [ ] Target reduction from 343 to <150

## Completed
- [x] Investigate TS2304 extra errors (343 occurrences) - found patterns in false positives
- [x] Categorize TS2304 extra errors by type (globals, imports, locals, etc.)
- [x] Create analysis script and documentation (docs/ts2304_extra_analysis.md)

## Context
TS2304 is both missing (116) AND extra (343). The extra errors indicate the binder is incorrectly failing to find symbols that should be in scope.

---

## Analysis Findings

### Key TS2304 Extra Error Patterns Identified

1. **Import/Export Binding Issues** (~40% of extra errors)
   - Re-exported symbols not resolvable
   - Namespace import handling incomplete
   - Type-only imports not creating bindings

2. **Scope Chain Traversal** (~30% of extra errors)
   - Nested function/class scopes not walking up correctly
   - Block-scoped declarations not visible in child scopes
   - Closure capture failures

3. **Global Symbol Resolution** (~20% of extra errors)
   - lib.d.ts symbols not always reachable
   - Global augmentation not merging correctly
   - Multiple file global scope issues

4. **Module Resolution** (~10% of extra errors)
   - Ambient module declarations not working
   - @types package resolution failures

### Files Created
- `docs/ts2304_extra_analysis.md`: Detailed analysis of 343 extra errors
- `wasm/differential-test/analyze-extra-ts2304.mjs`: Analysis script

### Next Steps
Fix the patterns identified above, starting with the highest impact categories.
