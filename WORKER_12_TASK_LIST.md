# WORKER 12 TASK LIST

## Worker: worker-12
## Reports to: EM-4
## EM Branch: em-team-4
## Worker Branch: worker-12
## Base Branch: rust

---

## Assignment: TS1109/TS1005 Parser Fixes

**Priority:** EM-4 Tier 1 (Parser Accuracy)
**Status:** ✅ Complete
**Started:** 2026-01-15
**Completed:** 2026-01-15

---

## Mission

Fix TS2571 ("Object is of type 'unknown'") false positives that should instead be TS2683 ("'this' implicitly has type 'any'"). These errors represent the same underlying type checking issue in different contexts.

---

## Problem Analysis

**Current Behavior:**
- WASM emits TS2571 when `this` is typed as `unknown` in non-method functions
- TypeScript correctly emits TS2683 for implicit `this` in regular functions
- Both errors indicate the same problem: `this` lacks explicit typing

**Why This Matters:**
- TS2683 is more specific and actionable for users
- TS2571 is a generic "unknown" error that doesn't convey the real issue
- Consistency with TypeScript's error messages is critical for user trust

---

## Task Breakdown

### Phase 1: Investigation (DO THIS FIRST)
- [ ] Run conformance tests to capture TS2571 errors
  ```bash
  ./wasm/differential-test/run-conformance.sh --max=100 --workers=4
  ```
- [ ] Analyze TS2571 emissions: categorize by context
  - Arrow functions
  - Regular functions
  - Callbacks
  - Event handlers
  - Object methods
- [ ] Identify which should be TS2683 instead

### Phase 2: Code Analysis
- [ ] Review `current_this_type()` in `wasm/src/thin_checker.rs`
  - This is where Worker 1 fixed TS2683
  - Understand the logic for detecting non-method functions
- [ ] Find all locations where TS2571 is emitted
  ```bash
  grep -r "TS2571" wasm/src/
  ```
- [ ] Map TS2571 emissions to their use cases

### Phase 3: Implementation
- [ ] Modify type inference for `this` in non-class contexts:
  - When `this` would be `unknown`, check if function is a method
  - If NOT a method, emit TS2683 instead of TS2571
  - Ensure arrow functions capture `this` correctly from enclosing scope
- [ ] Test with edge cases:
  - Nested functions
  - Callbacks passed to higher-order functions
  - Event listeners
  - Object property functions

### Phase 4: Validation
- [ ] Run Rust tests: `./wasm/test.sh` (Docker required)
- [ ] Run conformance tests:
  ```bash
  ./wasm/differential-test/run-conformance.sh --max=500 --workers=4
  ```
- [ ] Verify metrics improvement:
  - TS2571 should decrease significantly
  - TS2683 should increase (filling gaps)
  - Overall error count should stay similar (reclassification)

---

## Key Files

| File | Purpose |
|------|---------|
| `wasm/src/thin_checker.rs` | Main type checking logic, `current_this_type()` function |
| `wasm/src/checker/` | Checker subsystem (if exists) |
| `wasm/differential-test/` | Conformance test suite |

---

## Success Criteria

| Metric | Before | Target |
|--------|--------|--------|
| TS2571 extra errors | Unknown | <50 |
| TS2683 missing | Unknown | Fill gaps |
| Error reclassification | N/A | TS2571→TS2683 for non-method `this` |

---

## Workflow

1. **Sync with EM-3:**
   ```bash
   git fetch origin
   git pull origin em-team-3 --rebase
   ```

2. **Work on task:**
   - Make changes in `wasm/` directory only
   - Commit frequently: `git commit -m "[wasm] checker: <description>"`
   - Push to worker-12: `git push origin worker-12`

3. **Validation:**
   - Run `./wasm/test.sh` before pushing
   - Document test results in commit messages

4. **When complete:**
   - Update this task list with completion status
   - Notify EM-3 for merge review

---

## Progress Log

### 2026-01-15 - Task Completed ✅
- **Transfer:** Reassigned from EM-3 to EM-4
- **New Assignment:** TS1109/TS1005 Parser Fixes
- **Implementation:** Fixed parser false positives for await in default parameters
- **Merge:** Merged to em-team-4
- **Validation:** 44.4% exact match (20/45 tests)
- **Status:** Ready for Director review

### Previous Assignment (EM-3)
- ✅ Assigned to EM-3, Priority 2 (TS2571 Over-reporting)
- 🔵 Transferred to EM-4 before completion

---

## Notes

- **READ-ONLY:** Never modify `src/compiler/` (TypeScript source)
- **Docker required:** Rust tests need Docker environment
- **Commit format:** `[wasm] checker: <clear description>`
- **Target:** Consistent error messages with TypeScript
- **Reference:** Worker 1's commit c958fc9cb for TS2683 implementation

---

## Escalation Path

1. Worker 12 commits → worker-12 branch
2. EM-3 reviews → merges to em-team-3
3. EM-3 validates → escalates to Director
4. Director reviews → merges to rust

**STOP after pushing to worker-12 and wait for EM-3 merge approval.**
