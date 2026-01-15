# Worker-3 Task List

## Assignment: Class Property Initialization (TS2564)
**Priority:** 🟡 TACTICAL (High ROI)
**Owner:** worker-3
**Branch:** worker-3

## Task Description
Implement the `strictPropertyInitialization` check. TS2564 ("Property 'x' has no initializer and is not definitely assigned in the constructor") is the #1 missing error (413 occurrences). This check is simply not running.

## Problem Analysis
From PROJECT_DIRECTION.md:
- **TS2564 Missing (413):** Class properties without initializers are not being flagged
- **Root cause:** The `strictPropertyInitialization` check is not implemented in `thin_checker.rs`
- **High ROI:** This is a focused task that will eliminate the top missing error category
- **Prerequisite:** Parser and Binder must be working reasonably (but can iterate)

## Action Items

### Phase 1: Investigation (Ask Gemini First!)
```bash
# MANDATORY - Run this before writing any code
./scripts/ask-gemini.mjs "I need to implement strictPropertyInitialization check for TS2564. What files should I modify and what's the approach?"
```

- [ ] Read TypeScript's implementation of `strictPropertyInitialization`
  - Check how tsc implements this in `src/compiler/checker.ts`
  - Understand the definite assignment analysis algorithm
- [ ] Read `wasm/specs/WASM_ARCHITECTURE.md` checker section
- [ ] Study `wasm/src/checker/thin_checker.rs` structure
- [ ] Identify where control flow analysis (CFA) is or should be
- [ ] Run conformance tests to get baseline report:
  ```bash
  ./wasm/differential-test/run-conformance.sh --all
  ```

### Phase 2: Implementation
- [ ] Add TS2564 check to `wasm/src/checker/thin_checker.rs`:
  - Detect class properties without initializers
  - Implement definite assignment analysis:
    - Property is assigned in all constructor paths
    - Property has definite assignment assertion (!)
    - Property is declared with `declare` keyword
    - Property type includes `undefined`
  - Report TS2564 when property is not definitely assigned
- [ ] Add control flow analysis for constructors if needed:
  - Track all code paths in constructor
  - Verify property is assigned on all paths
- [ ] Add tests for TS2564 scenarios

### Phase 3: Validation
- [ ] Run `./wasm/test.sh` (Docker-only!)
- [ ] Run conformance tests: `./wasm/differential-test/run-conformance.sh --all`
- [ ] Compare to baseline report
- [ ] Verify Missing TS2564 reduced from 413 to <20
- [ ] Check for false positives (valid code flagged incorrectly)
- [ ] Check for false negatives (invalid code not flagged)

## Success Metrics
- **Missing TS2564:** Reduce from 413 to <20
- **Exact Match:** Should increase significantly
- **No regressions:** Don't break existing working tests
- **Accuracy:** Minimize false positives/negatives

## Deliverables
1. Code changes in `wasm/src/checker/thin_checker.rs`
2. Control flow analysis implementation (if needed)
3. Tests for TS2564 scenarios
4. Conformance test report showing improvement
5. Set `Ready for Merge: Yes` in your plan when complete

## Workflow
1. Sync: `git fetch origin && git merge origin/rust --no-edit`
2. **ASK GEMINI FIRST** (see Phase 1)
3. Write code following Gemini's guidance
4. Test: `./wasm/test.sh`
5. Commit: `[wasm] checker: implement strictPropertyInitialization (TS2564)`
6. Push to worker-3 branch
7. Run conformance tests and analyze report
8. Mark `Ready for Merge: Yes` in your plan

## Status
- **Ready for Merge:** No
- **Last Updated:** 2026-01-14
