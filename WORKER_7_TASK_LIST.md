# Worker 7 Task List

**Maintained by:** EM-2
**Branch:** worker-7 → rust
**Worktree:** /tmp/orchestrator-workspace/worktrees/worker-7

---

## Current Task (PENDING)

### Task 1: Switch Solver from `Any` to `Unknown`/`Error` Defaults

**Priority:** STRATEGIC (Priority #3 from PROJECT_DIRECTION.md)
**Focus:** Stop "Error Poisoning" by being stricter
**Impact:** TS2322 missing errors (310 occurrences) + TS7006 missing errors
**Location:** `src/solver/mod.rs`, `src/solver/subtype.rs`

#### Background
The compiler currently defaults to `Any` when it encounters something it doesn't understand. In TypeScript, `Any` disables all type checking, causing missing errors throughout the codebase. By switching to `Unknown` or `Error`, we expose these issues instead of silencing them.

#### Requirements
1. Change `lower_type` to return `Error` instead of `Any` on resolution failure
2. Implement "Lawyer" layer from `specs/SOLVER.md` for TypeScript quirks
3. Harden `solve_subtype` logic (function bivariance, void return exceptions)
4. Convert "Missing TS2322" into either "Exact Match" or "Extra TS2322"

#### Acceptance Criteria
- [ ] `lower_type` returns `TypeId::ERROR` or `TypeId::UNKNOWN` on failure (not `TypeId::ANY`)
- [ ] Running `./wasm/differential-test/run-conformance.sh --max=10000` shows:
  - Reduction in missing TS2322 errors
  - Possible temporary increase in extra errors (acceptable - we're exposing real bugs)
- [ ] `cargo test --lib` passes in wasm/ directory
- [ ] Unit tests for new strictness behavior

#### Testing Strategy
```bash
# Run conformance to see impact
./wasm/differential-test/run-conformance.sh --max=10000

# Find specific TS2322 issues
node wasm/differential-test/find-ts2322.mjs

# Run unit tests
cd wasm && cargo test solver
```

#### Implementation Notes
- Look at `src/solver/mod.rs` for the `lower_type` function
- The `solve_subtype` function in `src/solver/subtype.rs` is where type compatibility is checked
- Read `specs/SOLVER.md` for the "Lawyer" layer specification
- TypeScript quirks to handle:
  - Function bivariance: function types are covariant in return, contravariant in parameters
  - Void return exceptions: functions returning void can be used where non-void is expected in some cases
  - Optional properties: `undefined` is allowed in optional property types
- **Expectation:** This will temporarily increase error count as we expose hidden bugs

#### Strategy
The goal is not to be "correct" immediately, but to be "stricter" so we can see where the real problems are. Better to have extra errors that we can fix than missing errors that hide bugs.

---

## Queue (Future Tasks)

### Task 2: Implement "Lawyer" Layer for TypeScript Quirks
**Priority:** HIGH
**Focus:** TypeScript-specific type system behaviors
**Dependencies:** Task 1

### Task 3: Harden Subtype Solving Logic
**Priority:** HIGH
**Focus:** Generic inference, union/intersection types
**Dependencies:** Task 1

---

## Anti-Priorities (DO NOT WORK ON)
- New emitter transforms
- LSP features
- CLI argument parsing
- Performance micro-optimizations

---

## Status Updates
- **Created:** 2026-01-14
- **Last Updated:** 2026-01-14
- **Current Focus:** Switch Solver to `Unknown`/`Error` Defaults
