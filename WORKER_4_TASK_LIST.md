# Worker 4 Task List

## Squad: Parser
**Target Branch:** `rust`
**Worker:** 4
**Status:** In Progress

---

## Current Assignment: Parser False Positive Reduction (Priority 1)

### Context
Project Zang is in Phase 8 (Convergence). The highest priority conformance gap for Parser Squad is **False Positive Reduction**:
- **TS1005**: "expected X" - likely over-triggering (701 total parser false positives)
- **TS1109**: "expression expected" - false positives on edge cases
- **Combined Target**: Reduce to <100 false positives

### Architecture Understanding
From `WASM_ARCHITECTURE.md` and current codebase:
- Parser uses `thin_parser.rs` with error recovery mechanisms
- Flow graph improvements from previous work provide better context
- Need better "resynchronization" after syntax errors
- Parser should continue after minor syntax errors

---

## Tasks

### Task 1: Audit TS1005 Emission
**Status:** PENDING
**Estimated Impact:** Foundation for reducing false positives

**Sub-tasks:**
- [ ] Read `src/thin_parser.rs` focusing on TS1005 emission points
- [ ] Identify all locations where "expected X" errors are raised
- [ ] Run `node wasm/differential-test/find-ts1005.mjs` to get baseline
- [ ] Identify patterns: When is TS1005 a false positive?
- [ ] Document common false positive scenarios

**Deliverable:** Analysis document documenting:
1. All TS1005 emission locations in thin_parser.rs
2. Current false positive baseline count
3. Common false positive patterns
4. Proposed fix strategies

**Acceptance Criteria:**
- Clear understanding of TS1005 emission logic
- Documented list of false positive patterns
- Baseline metrics established

---

### Task 2: Audit TS1109 Emission
**Status:** BLOCKED (waiting for Task 1)
**Estimated Impact:** Complements TS1005 work

**Sub-tasks:**
- [ ] Read `src/thin_parser.rs` focusing on TS1109 emission points
- [ ] Identify all locations where "expression expected" errors are raised
- [ ] Run `node wasm/differential-test/find-ts1109.mjs` to get baseline
- [ ] Identify patterns: When is TS1109 a false positive?
- [ ] Document common edge cases (e.g., trailing commas, missing semicolons)

**Deliverable:** Analysis document documenting:
1. All TS1109 emission locations
2. Current false positive baseline count
3. Common edge case patterns
4. Proposed fix strategies

**Acceptance Criteria:**
- Clear understanding of TS1109 emission logic
- Documented list of edge case patterns
- Baseline metrics established

---

### Task 3: Implement Better Error Recovery
**Status:** BLOCKED (waiting for Task 2)
**Estimated Impact:** Core infrastructure improvement

**Sub-tasks:**
- [ ] Implement "resynchronization" logic in `thin_parser.rs`
- [ ] Add lookahead to detect recovery points (statement boundaries, blocks, etc.)
- [ ] Modify error emission to suppress cascading errors after recovery
- [ ] Ensure parser state is consistent after recovery
- [ ] Add unit tests for error recovery scenarios

**Deliverable:** Working error recovery mechanism

**Acceptance Criteria:**
- Parser continues after minor syntax errors
- No cascading false positives after recovery
- Unit tests for recovery scenarios pass
- Integration into `WasmProgram` pipeline

---

### Task 4: Validate with Conformance Suite
**Status:** BLOCKED (waiting for Task 3)
**Estimated Impact:** Confirms real-world improvement

**Sub-tasks:**
- [ ] Run `./wasm/differential-test/run-conformance.sh --max=10000`
- [ ] Measure TS1005 false positive reduction (target: 300+)
- [ ] Measure TS1109 false positive reduction (target: 200+)
- [ ] Check for regressions in other error types
- [ ] Fix any regressions or false negatives

**Deliverable:** Documented conformance improvement

**Acceptance Criteria:**
- Combined TS1005+TS1109 false positives reduced to <100 (from 701)
- No regressions in other error types
- All existing unit tests pass

---

## Anti-Priorities (DO NOT WORK ON)
- New Emitter transforms
- LSP features
- CLI argument parsing
- Performance optimizations (unless regression)

---

## Workflow
1. Complete Task 1 → Write TS1005 analysis
2. Complete Task 2 → Write TS1109 analysis
3. Implement Task 3 → Error recovery with unit tests
4. Validate Task 4 → Document conformance results

## Commit Protocol
- After each task: `git add -A && git commit -m "Worker4: Complete Task N" && git push origin worker-4 --force`
- After full completion: Report to EM-1 for merge review

---

## Progress Tracking

| Task | Status | Last Updated |
|------|--------|--------------|
| Task 1: TS1005 Audit | ⏳ PENDING | 2026-01-14 |
| Task 2: TS1109 Audit | 🔒 BLOCKED | 2026-01-14 |
| Task 3: Error Recovery | 🔒 BLOCKED | 2026-01-14 |
| Task 4: Validation | 🔒 BLOCKED | 2026-01-14 |

---

## Previous Work: Control Flow Analysis (COMPLETE ✅)

**Date:** 2025-01-14
**Merged to:** em-team-1
**Status:** ✅ APPROVED FOR DIRECTOR REVIEW

### Key Findings

**Task 1 Investigation Result:** Flow infrastructure EXISTS and is INTEGRATED
- No new construction needed
- Infrastructure in `thin_binder.rs`, `thin_checker.rs`, `checker/control_flow.rs`
- Pivoted from building new infrastructure to fixing existing code

**Task 2-3 Results:** Bug fixes in existing flow analysis
- `wasm/src/checker/control_flow.rs`: +33 lines
- `wasm/src/thin_binder.rs`: +18 lines
- `wasm/src/binder.rs`: +1 line

**Task 4 Validation:** Stable test pass rate
- 7,923 PASSED (98.1%)
- 154 FAILED (1.9%)
- No regressions introduced

**Impact:**
- Improved flow graph accuracy
- Better error recovery during parsing
- Foundation for TS1005/TS1109 false positive reduction

**See:** `WORKER_4_MERGE_RESULTS.md` for full details
