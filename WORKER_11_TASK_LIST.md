# Worker 11 Task List

## Squad: Binder & Scope Resolution - TS2304 Focus

## Current Task
- [ ] Investigate why `console.log` fails to resolve in test cases
- [ ] Check if `lib.dom.d.ts` is being loaded correctly

## Queue
- [ ] Verify `lib.d.ts` symbols are merged into root `SymbolTable`
- [ ] Debug `file_locals` population from library context
- [ ] Fix module augmentation resolution (merging `interface Window` across files)
- [ ] Fix global scope pollution - symbols should not leak between files
- [ ] Reduce TS2304 extra errors to <50

## Completed
- [ ] Branch created from em-team-3
- [ ] Reviewed binder.ts structure and scope resolution logic

## Context
TS2304 ("Cannot find name") is the #1 source of error poisoning. When the binder fails to resolve `Promise`, `Array`, or `console`, the solver defaults to `Any`, suppressing all downstream errors.

### Current Status
| TS2304 Error Type | Count |
|-------------------|-------|
| Extra (false positives) | 343 |
| Missing (real errors not caught) | 116 |

### Investigation Steps

**Step 1 - Verify lib.d.ts loading:**
- Check `src/compiler/program.ts` for library initialization
- Verify `lib.dom.d.ts` is included in compilation
- Add debug logging if needed

**Step 2 - Check SymbolTable merging:**
- Global symbols should be in the root `SymbolTable`
- Check `src/compiler/binder.ts` for scope merging logic
- Look for `createSymbolTable()` and merge operations

**Step 3 - Debug console.log specifically:**
- Find a test case where `console.log` fails
- Trace through binder: why isn't `console` found?
- Is it a scoping issue? A module issue?

**Step 4 - Module augmentation:**
- `interface Window` should merge across files
- Check `mergeSymbol()` logic in binder
- Verify module declaration handling

### Key Files
- `src/compiler/binder.ts` (194K lines) - Main binding logic
- `src/compiler/program.ts` (268K lines) - Program setup, lib loading
- `src/compiler/checker.ts` - Type checking (uses bound symbols)

### Success Metric
- TS2304 extra errors: 343 → <50
- TS2304 missing errors: 116 → <20
- `console.log`, `Promise`, `Array` should resolve correctly in global scope

### Workflow
1. Create branch: `git checkout -b worker-11-binder-scope-fix`
2. Investigate and fix scope resolution issues
3. Run tests: `npm run test:conformance`
4. Push: `git push origin worker-11-binder-scope-fix`
5. Notify EM-3 for merge
