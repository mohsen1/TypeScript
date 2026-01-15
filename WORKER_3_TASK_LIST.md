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

### Next Steps for TS2564 (Original Assignment)
The TS2564 task still requires:
- [ ] Read TypeScript's implementation of `strictPropertyInitialization`
- [ ] Study `wasm/src/checker/thin_checker.rs` structure
- [ ] Implement definite assignment analysis
- [ ] Add TS2564 check to `wasm/src/checker/thin_checker.rs`
- [ ] Run conformance tests to verify Missing TS2564 reduced from 413 to <20

## Status
- **Merged to em-team-1:** Yes (ff506c83f)
- **Original Task (TS2564) Completed:** ❌ NO - Different work was done
- **Tests Passed:** ✅ Yes
- **Last Updated:** 2026-01-14 (EM-1 review)
- **Next Action:** Director review - accept off-task work and reassign TS2564

---

## Original Task Details (For Reference)

### Action Items

#### Phase 1: Investigation
```bash
# MANDATORY - Run this before writing any code
./scripts/ask-gemini.mjs "I need to implement strictPropertyInitialization check for TS2564. What files should I modify and what's the approach?"
```

#### Phase 2: Implementation
- [ ] Add TS2564 check to `wasm/src/checker/thin_checker.rs`
- [ ] Implement definite assignment analysis
- [ ] Add control flow analysis for constructors
- [ ] Add tests for TS2564 scenarios

#### Phase 3: Validation
- [ ] Run `./wasm/test.sh` (Docker-only!)
- [ ] Run conformance tests
- [ ] Verify Missing TS2564 reduced from 413 to <20

### Success Metrics
- **Missing TS2564:** Reduce from 413 to <20
- **Exact Match:** Should increase significantly
- **No regressions:** Don't break existing working tests
