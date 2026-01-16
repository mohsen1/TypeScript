# Worker 14 Task List

**Maintained by**: EM-4
**Worker**: Worker 14
**Worktree**: /var/folders/57/xp3brw212ygckhkk_ml783fr0000gn/T/cco-workspace-TypeScript-1768508073410/worktrees/worker-14
**Target Branch**: rust
**Squad**: Quality & Stability Squad (EM-4)

---

## Current Task

### [ ] Task: Fix Symbol Resolution Errors (TS2304)

**Priority:** 🟡 HIGH (Tier 3 - Symbol Resolution)
**Assigned:** 2026-01-15
**Status:** 🔄 IN PROGRESS

### Problem

The thin checker has issues with symbol resolution:
- **TS2304:** "Cannot find name 'X'" - 7 missing, 5 extra errors

**Current Impact:**
- 7 missing TS2304 errors (global/local symbol lookup gaps)
- 5 extra TS2304 errors (false positives)

**Note:** EM-4 task description mentioned TS2524, but TS2524 is about "'await' expressions cannot be used in a parameter initializer" (async/await issue), not module exports. TS2524 should be handled by Worker 13 (async/await) or as part of TS1109 fixes. My scope is TS2304 only.

### Root Cause

1. **TS2304 Missing:** Symbol table lookup may not be checking all scopes correctly
2. **TS2304 Extra:** Symbols being found when they shouldn't be (scope leakage)

### Action Items

#### Phase 1: Investigation

1. **Examine test failures**
   ```bash
   # Find symbol-related test failures
   cd wasm/differential-test
   grep -r "TS2304" output/ --include="*.json" | head -50
   ```

2. **Study symbol resolution code**
   - `wasm/src/binder/` - symbol table and scope management
   - `wasm/src/thin_checker.rs` - symbol resolution logic (line 16198+)
   - `wasm/src/parallel.rs` - module handling

3. **Find missing diagnostics**
   - Check TS2304 code exists in `diagnostic_codes` ✓ (confirmed: code 2304)
   - Identify where these should be emitted
   - Find specific test cases showing missing/extra errors

#### Phase 2: Implementation

1. **Fix TS2304 missing errors**
   - Ensure global symbols are checked
   - Ensure local symbols are in correct scope
   - Fix scope chain traversal

2. **Fix TS2304 extra errors**
   - Identify why extra TS2304s are emitted
   - Fix scope boundaries to prevent leakage

#### Phase 3: Validation

1. **Run targeted tests**
   ```bash
   cd wasm/differential-test
   # Test symbol resolution specifically
   bash run-conformance.sh --category="Symbols" --max=100
   bash run-conformance.sh --category="ambient" --max=100
   ```

2. **Run conformance suite**
   ```bash
   cd wasm/differential-test
   bash run-conformance.sh --max=500 --workers=4
   ```
   - Track TS2304 count (target: reduce missing to <3)
   - Track TS2304 extra count (target: reduce to <2)
   - Ensure no regression

3. **Compare with tsc output**
   ```bash
   # Verify our errors match TypeScript compiler
   ```

### Files to Work On

- **Primary:** `wasm/src/binder/` - Symbol table management
- **Primary:** `wasm/src/thin_checker.rs` - Symbol resolution logic (line 16198+ for `emit_cannot_find_name`)
- **Tests:** Create test cases for symbol resolution edge cases

### Success Criteria

| Metric | Current | Target |
|--------|---------|--------|
| TS2304 missing errors | 7 | <3 |
| TS2304 extra errors | 5 | <2 |

### Reference

- **EM_4_TASKS.md:** Tier 3 Symbol Resolution priorities
- **PROJECT_DIRECTION.md:** Symbol resolution guidelines
- **wasm/src/checker/types/diagnostics.rs:** Diagnostic code definitions
- **TS2524_ANALYSIS.md:** TS2524 is async/await issue, not module exports (out of scope)

### Instructions

1. Sync with rust branch: `git fetch origin && git pull origin rust`
2. Work on TS2304 symbol resolution ONLY
3. Commit frequently with descriptive messages:
   - `fix(wasm): add TS2304 for undefined global symbols`
   - `fix(wasm): fix TS2304 for scope lookup gaps`
   - `fix(wasm): remove TS2304 false positives from scope leakage`
4. Push to worker-14 branch: `git push origin worker-14`
5. Run tests locally before considering complete
6. Update this task list with status
7. Notify EM-4 when ready for review

### Validation Checklist Before Merge

- [ ] TS2304 missing errors reduced to target (<3)
- [ ] TS2304 extra errors reduced to target (<2)
- [ ] Global symbol lookup working
- [ ] Local symbol lookup working
- [ ] No regression in valid error detection
- [ ] Test cases added for new functionality

---

## Completed Tasks

### ✅ Task 1: Fix TS2322 False Positives in Union Type Assignability (2026-01-15)
**Status:** Merged to em-team-4
**Result:** Investigation found union type logic is correct - 548 extra TS2322s are legitimate errors from Worker 7's "Invert Solver Defaults" fix

**Created:**
- UNION_ASSIGNABILITY_ANALYSIS.md - Deep dive into union type checking
- test_union_assignability.rs - Unit tests for union types
- test_union_assignability.ts - TypeScript test cases

**Key Finding:** No implementation needed in subtype checker - logic already correct

---

## Progress Log

**2026-01-15 - Tier 0 Progress 🔵:**
- ✅ Previous task (TS2322 union types) investigation complete
- ✅ TS2304 Symbol Resolution investigation complete (Phase 1)
- ✅ TS2304 implementation plan and test cases merged (368 lines)
- 🔄 Reassigned to Tier 0: AST Child Enumeration
- 🔄 New task: Implement `get_children` for parser arenas

**2026-01-15 - TS2304 Investigation Merge:**
- **Files Created:**
  - TS2304_IMPLEMENTATION_PLAN.md (220 lines) - Detailed fix strategy
  - ts2304-test-cases.ts (133 lines) - Comprehensive test coverage
  - test-symbols.ts (15 lines) - Basic symbol resolution tests
- **Validation:** 44.4% exact match (20/45 tests) - baseline maintained
- **Merge Commit:** 3a24ef248 "Merge branch 'worker-14' into em-team-4"
- **Status:** Phase 1 complete, ready for EM-4 validation

**2026-01-15 - Earlier:**
- ✅ Previous task (TS2322 union types) investigation complete
- ✅ Merged to em-team-4
- 🔄 Reassigned to Symbol Resolution (TS2304/TS2524)
- 🔄 Synced with rust branch

---

## Notes

- **EM-4 Status:** Active - worker-14 reassigned to Symbol Resolution
- **Team Alignment:** Tier 3 Symbol Resolution focus
- **Next Steps:** Investigate TS2304/TS2524 failures and implement fixes
- **Priority:** Symbol resolution is critical for accurate error reporting
