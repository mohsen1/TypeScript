# Worker-2 Task List

## Assignment: Global Scope Fix (TS2304)
**Priority:** 🔴 CRITICAL
**Owner:** worker-2
**Branch:** worker-2

## Task Description
Fix the "Global Scope" problem. TS2304 ("Cannot find name 'X'") appears 343 times in Extra errors and 116 times in Missing errors. This is the root cause of "Error Poisoning" - missing globals like `console`, `Promise`, `Array` cause downstream errors to be suppressed.

## Problem Analysis
From PROJECT_DIRECTION.md:
- **Extra TS2304 (343):** We aren't loading `lib.d.ts` correctly in the test runner
  - Global symbols like `console`, `Promise`, `Array` are undefined
- **Missing TS2304 (116):** When `Promise` is undefined, Solver treats it as `Any`
  - This suppresses TS2322 (Type Mismatch) errors downstream
- **Root cause:** lib injection and global merging issues in binder

## Action Items

### Phase 1: Investigation (Ask Gemini First!)
```bash
# MANDATORY - Run this before writing any code
./scripts/ask-gemini.mjs "I need to fix lib.d.ts injection and global merging to resolve TS2304 errors. What files should I modify and what's the approach?"
```

- [ ] Read `wasm/specs/WASM_ARCHITECTURE.md` binder section
- [ ] Study `wasm/src/binder/` lib injection logic
- [ ] Find where `lib.d.ts` should be loaded in test runner
- [ ] Understand global symbol merging (e.g., `interface Window`)
- [ ] Run conformance tests to get baseline report:
  ```bash
  ./wasm/differential-test/run-conformance.sh --all
  ```

### Phase 2: Implementation
- [ ] Fix lib.d.ts injection in test runner/integration:
  - Ensure `lib.d.ts` is correctly merged into root `SymbolTable` for every test
  - Verify lib files are loaded before type checking begins
- [ ] Fix global merging logic:
  - Ensure `interface Window` and similar globals merge correctly across files
  - Handle global augmentations properly
- [ ] Add tests for global symbol resolution

### Phase 3: Validation
- [ ] Run `./wasm/test.sh` (Docker-only!)
- [ ] Run conformance tests: `./wasm/differential-test/run-conformance.sh --all`
- [ ] Compare to baseline report
- [ ] Verify Extra TS2304 reduced from 343 to <10
- [ ] Verify downstream errors (TS2322, etc.) now appear correctly
- [ ] Check that `console`, `Promise`, `Array` are resolvable

## Success Metrics
- **Extra TS2304:** Reduce from 343 to <10
- **Missing TS2304:** Should approach expected count (not 0, but not 116)
- **Downstream errors:** TS2322 and other type errors should increase (good! = less poisoning)
- **No regressions:** Don't break existing working tests

## Deliverables
1. Code changes in `wasm/src/binder/` and/or `wasm/src/integration/`
2. Tests for global symbol resolution
3. Conformance test report showing improvement
4. Set `Ready for Merge: Yes` in your plan when complete

## Workflow
1. Sync: `git fetch origin && git merge origin/rust --no-edit`
2. **ASK GEMINI FIRST** (see Phase 1)
3. Write code following Gemini's guidance
4. Test: `./wasm/test.sh`
5. Commit: `[wasm] binder: fix lib.d.ts injection and global merging`
6. Push to worker-2 branch
7. Run conformance tests and analyze report
8. Mark `Ready for Merge: Yes` in your plan

## Status
- **Ready for Merge:** No
- **Last Updated:** 2026-01-14
