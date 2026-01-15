# Worker-4 Task List

## Assignment: Recursion Guards
**Priority:** 🟢 STABILITY
**Owner:** worker-4
**Branch:** worker-4

## Task Description
Add recursion depth guards to prevent stack overflow crashes. The test `types/typeRelationships/recursiveTypes` currently causes a panic. We need to detect deep recursion and return a proper error (TS2589) instead of crashing.

## Problem Analysis
From PROJECT_DIRECTION.md:
- **Crashes:** 2 stack overflow panics on recursive types
- **Root cause:** Unbounded recursion in `solve_subtype` and/or `check_expression`
- **TypeScript behavior:** Returns "Type instantiation is excessively deep" error (TS2589)
- **Our behavior:** Crashes the WASM process
- **Fix:** Add recursion depth counters with configurable limit (e.g., 100)

## Action Items

### Phase 1: Investigation (Ask Gemini First!)
```bash
# MANDATORY - Run this before writing any code
./scripts/ask-gemini.mjs "I need to add recursion guards to solve_subtype and check_expression to prevent stack overflow. What files should I modify and what's the approach?"
```

- [ ] Reproduce the crash:
  ```bash
  ./wasm/test.sh  # Run specific test that crashes
  ```
- [ ] Read `wasm/specs/WASM_ARCHITECTURE.md` solver section
- [ ] Study `wasm/src/solver/` for recursion patterns
- [ ] Study `wasm/src/checker/` for recursion patterns
- [ ] Identify all functions that may recurse deeply:
  - `solve_subtype` (likely culprit)
  - `check_expression` (possible)
  - Type instantiation functions
- [ ] Understand TypeScript's TS2589 error handling

### Phase 2: Implementation
- [ ] Add recursion depth counter to `wasm/src/solver/`:
  - Add `recursion_depth: usize` field to solver context
  - Increment counter on recursive calls
  - Check limit before recursing
  - Return TS2589 error when limit exceeded
- [ ] Add recursion guards to `wasm/src/checker/` if needed:
  - Same pattern as solver
  - Check for expression checking recursion
- [ ] Make limit configurable (default: 100)
- [ ] Add tests for deep recursion scenarios

### Phase 3: Validation
- [ ] Run `./wasm/test.sh` (Docker-only!)
- [ ] Run recursiveTypes test specifically:
  ```bash
  # Should return TS2589 instead of crashing
  ```
- [ ] Run conformance tests: `./wasm/differential-test/run-conformance.sh --all`
- [ ] Verify zero crashes (no panics)
- [ ] Verify TS2589 errors appear for deeply recursive types
- [ ] Check that normal recursive types still work

## Success Metrics
- **Crashes:** Zero (no panics)
- **TS2589 errors:** Appear for excessively deep recursion
- **Normal recursion:** Still works correctly
- **No regressions:** Don't break existing working tests

## Deliverables
1. Code changes in `wasm/src/solver/` and/or `wasm/src/checker/`
2. Recursion depth counter implementation
3. TS2589 error reporting
4. Tests for deep recursion
5. Conformance test report showing zero crashes
6. Set `Ready for Merge: Yes` in your plan when complete

## Workflow
1. Sync: `git fetch origin && git merge origin/rust --no-edit`
2. **ASK GEMINI FIRST** (see Phase 1)
3. Write code following Gemini's guidance
4. Test: `./wasm/test.sh`
5. Commit: `[wasm] solver: add recursion guards to prevent stack overflow`
6. Push to worker-4 branch
7. Run conformance tests and analyze report
8. Mark `Ready for Merge: Yes` in your plan

## Status
- **Ready for Merge:** No
- **Last Updated:** 2026-01-14
