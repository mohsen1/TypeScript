# Worker 8 Task List

**Maintained by:** EM-2
**Branch:** worker-8 → rust
**Worktree:** /tmp/orchestrator-workspace/worktrees/worker-8

---

## Completed Tasks

### ✅ Task 1: TS2454 - Variable Use Before Assignment Detection

**Status:** COMPLETED (2026-01-14)
**Commit:** c0c454e33

Fixed critical bug where TS2454 errors were not reported for variables used as arguments in call expressions when the callee returned ANY/ERROR type.

### ✅ Task 2: TS2564 - Property Initialization Detection

**Status:** ALREADY IMPLEMENTED
**All 12 unit tests pass**

The TS2564 functionality was already fully implemented in the codebase with comprehensive coverage for all edge cases.

---

## In Progress / Requires Investigation

### 🔍 Task 3: Fix TS2322 - Solver Strictness Improvements

**Priority:** MEDIUM (Priority #2 from README)
**Status:** REQUIRES DEEPER INVESTIGATION
**Impact:** 310 missing errors in conformance tests

#### Investigation Findings
- **Scope:** Extensive - changing `Any` fallback to `Unknown/Error` affects core solver behavior
- **Locations:** `wasm/src/solver/infer.rs`, `wasm/src/solver/subtype.rs`, `wasm/src/solver/compat.rs`, `wasm/src/solver/operations.rs`
- **Risk:** HIGH - changes could cause widespread test failures and type checking regressions
- **Current State:** TS2322 errors ARE being emitted in many test cases (comprehensive test coverage exists)

#### Key Challenge
The "310 missing errors" likely comes from conformance test comparisons with tsc. However:
1. Changing solver fallback from `Any` to `Unknown/Error` is a architectural change
2. Could break existing valid code patterns
3. Requires extensive regression testing
4. May need to be done incrementally with careful validation

#### Recommendation
This task requires:
1. Detailed conformance test analysis to identify SPECIFIC missing TS2322 cases
2. Targeted fixes rather than wholesale solver changes
3. Incremental approach with validation at each step
4. Possible coordination with solver architecture team

**Note:** This is a complex task that benefits from having complete test infrastructure and possibly more context from the original TypeScript implementation.

---

## Queue (Future Tasks)

### Task 4: Reduce TS2339 False Positives
**Priority:** MEDIUM (Priority #3 from README)
**Error Code:** TS2339 ("Property '{0}' does not exist on type '{1}'")
**Impact:** 292 extra errors (false positives)
**Approach:** Improve type narrowing, implement apparent members for primitives

---

## Anti-Priorities (DO NOT WORK ON)
- New emitter transforms (ES3, obscure module formats)
- LSP features (semantic tokens, code actions)
- CLI argument parsing
- Performance micro-optimizations

---

## Status Updates
- **Created:** 2026-01-14
- **Last Updated:** 2026-01-14
- **Current Focus:** Task 3 requires deeper investigation
- **Progress:** 2/4 tasks complete (50%), 1 task blocked on investigation

---

## Summary of Work Session
### Completed (2026-01-14)
1. **TS2454 Fix (Task 1)**: Fixed critical bug - definite assignment now checked for call arguments even when callee is ANY/ERROR
2. **TS2564 Verification (Task 2)**: Confirmed already fully implemented with comprehensive test coverage

### Remaining
- **Task 3**: Requires detailed conformance test analysis and targeted fixes
- **Task 4**: Ready to start when Task 3 is resolved or deferred

**Total commits:** 3 (c0c454e33, 54d47ecf9, 88324d5f8)
