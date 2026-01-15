# Worker-4 Task List

## Previous Assignment: Recursion Guards ✅ COMPLETED
- **Status:** Complete and merged
- **Summary:** Recursion guards were already implemented in SubtypeChecker and TypeInstantiator. Fixed syntax error in thin_parser.rs and updated test expectation in subtype_tests.rs. All 15 recursion tests pass, no stack overflow panics.

---

## Assignment: Fix Flow Analysis Tests
**Priority:** 🟡 STABILITY
**Owner:** worker-4
**Branch:** worker-4

## Task Description
Multiple flow analysis tests are failing with panics. These tests are related to closure capture and flow graph analysis. The failures appear to be pre-existing issues, not related to recent changes.

## Problem Analysis
From test failures:
- test_closure_capture_with_array_filter - panics at line 1827
- test_closure_capture_with_array_map - panics at line 1410
- test_closure_with_conditional_capture - panics at line 1703
- test_multiple_closures_capture_same_variable - panics at line 1631
- test_flow_graph_captures_* - multiple flow graph capture tests failing

## Action Items

### Phase 1: Investigation
- [ ] Run the failing tests and capture full panic messages
- [ ] Read wasm/src/checker/flow_analyzer.rs to understand flow analysis
- [ ] Read wasm/src/checker/control_flow_tests.rs to understand test expectations
- [ ] Identify the root cause of the panics

### Phase 2: Fix Implementation
- [ ] Fix the flow analysis logic to handle closure captures correctly
- [ ] Ensure flow graph properly captures variable access in closures
- [ ] Fix any assertion failures in the tests

### Phase 3: Validation
- [ ] Run all flow analysis tests: cargo test control_flow
- [ ] Run full test suite: ./wasm/test.sh
- [ ] Verify no regressions in other tests

## Success Metrics
- Zero panics in flow analysis tests
- All control_flow tests passing
- No regressions in other test suites

## Deliverables
1. Code changes in wasm/src/checker/flow_analyzer.rs or related files
2. All flow analysis tests passing
3. Test report showing all tests green

## Workflow
1. Sync: git fetch origin && git merge origin/rust --no-edit
2. Investigate the failing tests
3. Fix the implementation
4. Test: cargo test control_flow
5. Commit: [wasm] checker: fix flow analysis closure capture tests
6. Push to worker-4 branch
7. Mark Ready for Merge: Yes in your plan

## Status
- Ready for Merge: No
- Last Updated: 2026-01-14
