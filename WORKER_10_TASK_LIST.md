# Worker 10 Task List

**Maintained by**: EM-3
**Worker**: Worker 10
**Worktree**: /var/folders/57/xp3brw212ygckhkk_ml783fr0000gn/T/cco-workspace-TypeScript-1768508073410/worktrees/worker-10
**Target Branch**: rust
**EM Branch**: em-team-3

---

## Mission (from EM-3)

Worker 10 is part of **EM-3: Type Checking Squad**. Our mission is to ensure type checker accuracy:
1. Type assignability is correctly enforced (TS2322)
2. `this` type handling is accurate (TS2683, TS2571)
3. Generic type constraints are properly checked
4. Object-oriented type checking works correctly (super(), extends, etc.)

**Current Priority**: Reduce TS2322 extra errors from ~548 to <200

---

## Tasks

### [x] Task 1: Fix TS2571 Over-reporting (Should be TS2683)
Investigate and fix cases where TypeScript emits TS2571 ("Object is of type 'unknown'") when it should emit TS2683 ("'this' implicitly has type 'any'").

**Problem**:
- TS2571 is over-reported in WASM type checking
- Many cases should be TS2683 instead (implicit `this` in non-class methods)
- These are the same underlying issue in different contexts

**Action**:
1. Search for TS2571 emission points in `wasm/src/thin_checker.rs`
2. Identify cases where the error should be TS2683 instead
3. Understand the difference:
   - TS2571: Object is of type 'unknown' (used when type cannot be inferred)
   - TS2683: 'this' implicitly has type 'any' (used when `this` is used in non-method functions)
4. Fix type inference for `this` in non-class methods
5. Test with arrow functions, callbacks, event handlers
6. Run conformance tests: `./wasm/differential-test/run-conformance.sh --max=100 --workers=2`
7. Commit with message: "fix: replace TS2571 with TS2683 for implicit this in functions"
8. Push to origin worker-10
9. Update task status and notify EM-3

**Target Metrics**:
- TS2571 extra errors: <50
- TS2683 missing errors: Fill gaps

**Status**: Completed

---

### [ ] Task 2: Reduce TS2322 Extra Errors (Type Assignability)
Investigate and reduce TS2322 ("Type X is not assignable to type Y") extra errors in WASM type checking.

**Problem**:
- TS2322 is over-reported (~548 extra errors)
- WASM type checker is stricter than TypeScript in some cases
- Need to reduce from ~548 to <200 extra errors

**Action**:
1. Run baseline conformance test: `./wasm/differential-test/run-conformance.sh --max=500 --workers=4`
2. Analyze TS2322 extra errors - categorize by pattern:
   - Union type assignability issues
   - Generic type constraint issues
   - Structural vs nominal typing differences
   - Literal type widening issues
3. Identify top 3-5 most common TS2322 error patterns
4. Focus on one pattern at a time:
   - Reproduce the issue with a minimal test case
   - Find the type checking logic in `wasm/src/thin_checker.rs` or `wasm/src/checker/`
   - Fix the assignability check
   - Verify fix doesn't introduce missing errors
5. Test each fix with conformance tests
6. Commit with descriptive message for each fix
7. Push to origin worker-10 after each fix
8. Update this task with progress

**Target Metrics**:
- TS2322 extra errors: reduce from ~548 to <200
- Exact Match Rate: improve from ~30% to 40%+

**Status**: In Progress

---

## Notes

**Key Files**:
- `wasm/src/thin_checker.rs` - Type checker implementation
- `wasm/src/checker/` - Type checking modules

**Conformance Testing**:
- Baseline: Run `./wasm/differential-test/run-conformance.sh --max=500 --workers=4` on em-team-3 branch
- Quick test: `./wasm/differential-test/run-conformance.sh --max=100 --workers=2`

**EM-3 Success Metrics**:
| Metric | Current | Target |
|--------|---------|--------|
| TS2322 extra errors | ~548 | <200 |
| TS2571 extra errors | Unknown | <50 |
| TS2683 missing errors | Unknown | Fill gaps |
| Exact Match Rate | ~30% | 40%+ |

---

## Workflow

1. **Stay in worktree**: Never touch other teams' directories
2. **Work on task branch**: All commits go to `worker-10` branch
3. **Commit frequently**: Use descriptive commit messages with `[wasm]` prefix
4. **Push when complete**: `git push origin worker-10`
5. **Notify EM-3**: When ready for merge, update this file and notify EM-3
6. **STOP after push**: Wait for EM-3 to merge and assign next task

**Important Rules**:
- Never force push
- All work must stay in `wasm/` directory
- TypeScript source files (`src/compiler/`) are READ-ONLY
- Test before committing

---

## Completed Tasks

### Task 1: Fix TS2571 Over-reporting (Should be TS2683) ✅
**Completed**: 2025-01-15
**Commit**: `b9fe445f3 fix: push this_type to stack before checking function body`

**Summary**:
Fixed TS2571 over-reporting where TS2683 should be emitted for implicit `this` in non-class methods.

**Changes Made**:
- Modified `wasm/src/thin_checker.rs` to push `this_type` to stack before checking function bodies
- Ensures proper type context for `this` references in all function types:
  - Functions with explicit `this` parameter: uses that type
  - Arrow functions: uses outer `this` type (lexical scoping)
  - Regular functions without explicit `this`: triggers TS2683 when `this` is used

**Testing Results**:
- Conformance tests: 44.4% exact match, 0 crashes
- No regressions introduced

**Push**: Successfully pushed to `origin worker-10`
