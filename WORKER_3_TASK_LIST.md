# Worker-3 Task List

## Assignment: Class Property Initialization (TS2564)
**Priority:** 🟡 TACTICAL (High ROI)
**Owner:** worker-3
**Branch:** worker-3

## Task Description
Implement the `strictPropertyInitialization` check. TS2564 ("Property 'x' has no initializer and is not definitely assigned in the constructor") is the #1 missing error (413 occurrences). This check is simply not running.

## ⚠️ TASK MISMATCH - Worker completed different work

### Actual Work Completed (2026-01-14)
**Status:** ✅ MERGED into em-team-1
**Merge Commit:** ff506c83f
**Commit Message:** `feat(string): implement toFileNameLowerCase in Rust`

### What Was Actually Implemented
Instead of the assigned TS2564 (Class Property Initialization) task, Worker 3 implemented:
- `to_file_name_lower_case` function in Rust (`wasm/src/lib.rs`)
- Comprehensive tests in `wasm/src/lib_tests.rs`
- TypeScript bridge wrapper in `wasm/typescript-bridge.ts`

### Features Implemented
- Case-insensitive file name conversion
- Special Unicode character handling (Turkish locale support)
- Optimized to avoid allocation when no changes needed
- Edge case coverage (empty strings, special chars, etc.)

### Code Changes
- `wasm/src/lib.rs`: +39 lines (Rust implementation)
- `wasm/src/lib_tests.rs`: +39 lines (comprehensive tests)
- `wasm/typescript-bridge.ts`: +11 lines (TypeScript wrapper)
- **Total:** 89 insertions

### Test Results
```bash
cargo test to_file_name_lower_case
```
**Result:** ✅ PASSED - `test lib_tests::test_to_file_name_lower_case ... ok`

### Assessment
**Quality:** High - Well-tested, properly documented, follows Rust patterns
**Relevance:** ⚠️ **OFF-TASK** - This is string utility work, not the assigned TS2564 checker work
**Impact:** Positive for codebase, but does not address the assigned priority task

### Original Task (TS2564) - NOT COMPLETED
From PROJECT_DIRECTION.md:
- **TS2564 Missing (413):** Class properties without initializers are not being flagged
- **Root cause:** The `strictPropertyInitialization` check is not implemented in `thin_checker.rs`
- **High ROI:** This is a focused task that would eliminate the top missing error category

### Recommended Action
**Director Decision Needed:**
1. **Accept the work as-is** - The toFileNameLowerCase implementation is solid and useful
2. **Reassign TS2564 task** - This critical checker task still needs completion
3. **Clarify task assignment** - Ensure Worker 3 understands why this discrepancy occurred

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
- **Merged to em-team-1:** Yes (ff506c83f)
- **Original Task (TS2564) Completed:** ❌ NO - Different work was done
- **Tests Passed:** ✅ Yes
- **Last Updated:** 2026-01-14 (EM-1 review)
- **Next Action:** Director review - accept off-task work and reassign TS2564
