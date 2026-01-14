# Worker 4 Task List

## Squad: Forge
**Target Branch:** `rust`
**Worker:** 4
**Status:** Ready

---

## Current Assignment: Control Flow Analysis (Priority 1)

### Context
Project Zang is in Phase 8 (Convergence). The highest priority conformance gap is **Control Flow Analysis**:
- **TS2454**: Variable used before assignment (573 missing errors)
- **TS2564**: Property not initialized (443 missing errors)
- **Combined Impact**: +1,016 tests

### Architecture Understanding
From `WASM_ARCHITECTURE.md` and current codebase:
- Parser uses `ThinNode` (Struct-of-Arrays) for performance
- Current `src/checker/control_flow.rs` exists but appears disconnected from checking
- `thin_checker.rs:check_identifier` needs to query Flow Graph for every identifier usage
- Must default to "Unassigned" (not "Assigned") when flow state is unknown

---

## Tasks

### Task 1: Investigate Current Control Flow Implementation
**Status:** PENDING
**Estimated Impact:** Foundation for all subsequent work

**Sub-tasks:**
- [ ] Read `src/checker/control_flow.rs` completely
- [ ] Read `src/checker/thin_checker.rs` focusing on `check_identifier`
- [ ] Read `WASM_ARCHITECTURE.md` section on Flow Graph design
- [ ] Identify: Does Flow Graph construction exist? Is it called?
- [ ] Identify: Is `check_identifier` querying flow state?

**Deliverable:** Brief analysis document documenting:
1. Current state of Flow Graph infrastructure
2. Current state of identifier checking
3. Gap analysis: What's missing between the two

**Acceptance Criteria:**
- Clear understanding of existing code
- Documented list of missing components

---

### Task 2: Implement Flow Graph Side-Table
**Status:** BLOCKED (waiting for Task 1)
**Estimated Impact:** Core infrastructure

**Sub-tasks:**
- [ ] Design side-table structure: `HashMap<NodeId, FlowState>` or similar
- [ ] Implement Flow Graph construction pass (runs after binding, before checking)
- [ ] Add FlowNode types: Uninitialized, DefinitelyAssigned, MaybeAssigned
- [ ] Implement basic control flow tracking for:
  - Variable declarations (`let`, `const`, `var`)
  - Assignment expressions
  - Return statements
  - Control flow branches (`if`, `for`, `while`)

**Deliverable:** Working Flow Graph construction that can be queried by the checker

**Acceptance Criteria:**
- Flow Graph pass compiles
- Unit tests for basic scenarios (declared-before-use, used-before-decl)
- Integration into `WasmProgram` pipeline

---

### Task 3: Connect Flow Graph to Checker
**Status:** BLOCKED (waiting for Task 2)
**Estimated Impact:** Enables TS2454/TS2564 detection

**Sub-tasks:**
- [ ] Modify `thin_checker.rs:check_identifier` to query Flow Graph
- [ ] Add TS2454 error emission when variable used before assignment
- [ ] Add TS2564 error emission for class properties not initialized in constructor
- [ ] Handle edge cases:
  - Destructuring patterns
  - Conditional declarations
  - Union of control paths

**Deliverable:** Checker that reports TS2454/TS2564 errors

**Acceptance Criteria:**
- `node wasm/differential-test/find-ts2454.mjs` shows reduction in missing errors
- `node wasm/differential-test/find-ts2564.mjs` shows reduction in missing errors
- Zero regressions in Extra Errors

---

### Task 4: Validate with Conformance Suite
**Status:** BLOCKED (waiting for Task 3)
**Estimated Impact:** Confirms real-world improvement

**Sub-tasks:**
- [ ] Run `./wasm/differential-test/run-conformance.sh --max=10000`
- [ ] Measure TS2454 missing error reduction (target: 400+)
- [ ] Measure TS2564 missing error reduction (target: 300+)
- [ ] Check Extra Errors for regressions
- [ ] Fix any regressions or false positives

**Deliverable:** Documented conformance improvement

**Acceptance Criteria:**
- Combined TS2454+TS2564 missing errors reduced by 500+
- Extra Errors increase < 50
- All existing unit tests pass

---

## Anti-Priorities (DO NOT WORK ON)
- New Emitter transforms
- LSP features
- CLI argument parsing
- Performance optimizations (unless regression)

---

## Workflow
1. Complete Task 1 → Write analysis
2. Implement Task 2 → Unit test
3. Implement Task 3 → Conformance test
4. Validate Task 4 → Document results

## Commit Protocol
- After each task: `git add -A && git commit -m "Worker4: Complete Task N" && git push origin worker-4 --force`
- After full completion: Report to EM-1 for merge review

---

## Progress Tracking

| Task | Status | Last Updated |
|------|--------|--------------|
| Task 1: Investigation | PENDING | 2026-01-14 |
| Task 2: Flow Graph | BLOCKED | - |
| Task 3: Checker Integration | BLOCKED | - |
| Task 4: Validation | BLOCKED | - |
