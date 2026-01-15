# Worker 1 Task List

Maintained by EM-1

## 🔴 CURRENT TASK: Missing Error Categories Investigation

**Last Updated:** 2026-01-15
**Status:** 🔄 ASSIGNED
**Priority:** 🟡 MEDIUM
**Timeline:** 1-2 days (parallel with worker-2's validation)

### Task Description

While worker-2 runs the full conformance baseline validation, investigate the top missing error categories to prepare a prioritized task backlog. This work can proceed in parallel and will accelerate the next planning cycle.

### Context

From previous analysis, we know:
- Exact match is at 44.2% (target: 80%+)
- Many missing error categories identified but not fully investigated
- worker-11 identified TS2322 as critical (105 missing errors)
- Other categories need similar investigation

### Deliverables

1. **Run Quick Conformance Sample**
   ```bash
   cd wasm/differential-test
   ./run-conformance.sh --samples 200 2>&1 | tee quick-sample-$(date +%Y%m%d).log
   ```

2. **Analyze Missing Errors**
   - Extract top 10 missing error categories by frequency
   - For each top 5 categories, provide:
     - Error code (TSXXXX)
     - Frequency (count and percentage)
     - 2-3 representative test cases
     - Hypothesized root cause
     - Estimated fix complexity (low/medium/high)
     - Suggested owner (worker or team)

3. **Create Investigation Report**
   - Document findings in `MISSING_ERRORS_INVESTIGATION.md`
   - Include summary table of top categories
   - Provide recommendations for priority order

### Example Output Format

```markdown
# Missing Errors Investigation

## Top 5 Missing Error Categories

| Error Code | Count | % | Complexity | Description |
|------------|-------|---|------------|-------------|
| TS2322 | 105 | 15% | High | Type mismatch - Abstract Constructor Assignability |
| TS7006 | 87 | 12% | Medium | Implicit Any |
| TS2575 | 52 | 7% | Low | Parameter duplication |
| ... | ... | ... | ... | ... |

## Detailed Analysis

### TS2322: Type Mismatch
**Frequency:** 105 occurrences (15%)
**Root Cause:** typeof AbstractClass not detected as non-assignable in TypeQuery expressions
**Test Cases:**
- testAbstractConstructorAssignability1
- testAbstractConstructorAssignability2
**Complexity:** HIGH - requires subtype checker changes
**Suggested Owner:** worker-11 (has analysis ready)

### TS7006: Implicit Any
...
```

### Success Criteria

- [ ] Quick conformance sample run (200 tests)
- [ ] Top 5 missing error categories identified
- [ ] Each category has 2-3 test cases documented
- [ ] Root cause hypotheses provided
- [ ] Complexity estimates provided
- [ ] `MISSING_ERRORS_INVESTIGATION.md` created
- [ ] Priority order recommendations provided

### Contingency: If TS1005/TS1109 Target Missed

If worker-2's validation shows TS1005/TS1109 did not meet the <40 target:
1. **PAUSE** this investigation immediately
2. **PIVOT** to TS1005/TS1109 Phase 2 refinements
3. Focus on remaining parser error recovery issues
4. Coordinate with worker-5 (EM-2) who achieved 96% reduction

### Timeline

- **Day 1:** Run conformance sample, extract error data
- **Day 2:** Analyze patterns, document findings, create report

### Impact

**MEDIUM** - Provides critical data for next planning cycle:
- Accelerates task assignment for all workers
- Identifies quick wins vs. strategic investments
- Helps balance workload across teams
- Enables data-driven prioritization

---

## Completed Tasks

### Task 1: Fix checker/expr.rs Optimistic Defaults (P0) ✅

**Status:** @ COMPLETE (2026-01-15)
**Priority:** 🔴 CRITICAL
**Commits:**
- b8697565779 Task 1: Fix checker/expr.rs optimistic defaults (P0)
- e1ecaa8ecd2 docs: EM-1 task reassignment notification

**Summary:**
Fixed optimistic type defaults in expression type checker to return UNKNOWN instead of ANY for error cases, improving type error detection.

**Changes:**
- Updated `wasm/src/checker/expr.rs` to return TypeId::UNKNOWN for missing nodes
- Updated `wasm/src/checker/expr.rs` to return TypeId::UNKNOWN for parsing failures
- Added detailed comments explaining the stricter type checking approach

**Impact:**
- Exposes type errors that were previously hidden by permissive ANY defaults
- Improves error reporting accuracy
- Aligns with worker-9's solver defaults inversion work

**Build Status:** ✅ Passed (cargo build --release: 2m 49s, 64 warnings)

**Merge Status:** ✅ Merged to em-team-1 (commit: f0debb590c3)

---

### TS1005/TS1109 Parser Noise Reduction ✅

**Status:** @ COMPLETE (2026-01-15)
**Priority:** 🔴 CRITICAL
**Commits:**
- 9596bd4f1bb [wasm] parser: allow reserved keywords in dotted module names
- 72ec386a349 [wasm] parser: fix await identifier allowed in static blocks
- 2c8b88308b0 [wasm] parser: comprehensive error suppression for TS1005/TS1109

**Summary:**
Enhanced parser error recovery to reduce false positive TS1005 and TS1109 errors.

**Improvements:**
1. Module names with reserved keywords: `declare namespace test.class {}` now valid
2. Await in static blocks: `static { let await = 1; }` now correctly parsed
3. Error recovery suppression: More lenient parsing at recovery boundaries

**Target:** Reduce TS1005/TS1109 from ~700 to <40
**Status:** Implementation complete, validation pending conformance tests

**Merge Status:** ✅ Merged to em-team-1

---

## Notes

- Work in: /tmp/orchestrator-workspace/worktrees/worker-1
- Push to worker-1 branch when complete
- Coordinate with worker-2 (running parallel validation)
- If TS1005/TS1109 target missed, pivot immediately to Phase 2
