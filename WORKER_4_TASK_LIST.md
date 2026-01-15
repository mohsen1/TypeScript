# Worker-4 Task List

## Previous Assignment: Flow Analysis Tests ✅ PARTIALLY COMPLETED
- **Status:** Partially completed and merged
- **Summary:** Fixed flow recording for unary/binary expressions in closures. Fixed AST navigation in test_closure_capture_with_array_map. 44/54 control_flow tests now passing.
- **Remaining:** Type narrowing issues, AST navigation in other tests, flow graph construction

---

## Assignment: Investigate and Fix Type Narrowing
**Priority:** 🟡 STABILITY
**Owner:** worker-4
**Branch:** worker-4

## Task Description
The flow analysis tests are failing at the type narrowing stage. The `FlowAnalyzer::get_flow_type` method is not correctly narrowing types based on flow nodes. This prevents proper type narrowing in closures even when flow is correctly recorded.

## Problem Analysis
From test failures:
- `test_closure_capture_with_array_filter` - fails at type narrowing (line 1830)
- `test_closure_capture_with_array_map` - fails at type narrowing (line 1450)
- The flow is recorded correctly, but `FlowAnalyzer::get_flow_type` returns wrong type
- Expected: `TypeId::STRING` (narrowed from `string | number`)
- Actual: `TypeId(130)` (some other type, possibly the union itself)

## Action Items

### Phase 1: Investigation
- [ ] Read `wasm/src/checker/flow_analyzer.rs` to understand `get_flow_type` logic
- [ ] Add debug output to understand what `TypeId(130)` represents
- [ ] Check if flow nodes are correctly connected to type narrowing logic
- [ ] Compare with working tests to understand expected flow graph structure

### Phase 2: Fix Implementation
- [ ] Fix `FlowAnalyzer::get_flow_type` to correctly narrow types
- [ ] Ensure flow conditions properly narrow union types
- [ ] Fix any flow graph construction issues

### Phase 3: Validation
- [ ] Run all flow analysis tests: `cargo test control_flow`
- [ ] Verify type narrowing works correctly in closures
- [ ] Fix remaining AST navigation issues in other tests

## Success Metrics
- All control_flow tests passing
- Type narrowing works correctly in closures
- No regressions in other tests

## Deliverables
1. Code changes in `wasm/src/checker/flow_analyzer.rs` or related files
2. All control_flow tests passing
3. Test report showing all tests green

## Workflow
1. Sync: `git fetch origin && git merge origin/rust --no-edit`
2. Investigate FlowAnalyzer logic
3. Fix type narrowing implementation
4. Test: `cargo test control_flow`
5. Commit: `[wasm] checker: fix type narrowing in flow analysis`
6. Push to worker-4 branch
7. Mark `Ready for Merge: Yes` in your plan

## Status
- **Ready for Merge:** No
- **Last Updated:** 2026-01-14
