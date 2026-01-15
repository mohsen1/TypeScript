# WORKER-13 Task List

**EM:** EM-4
**Focus:** Solver Test Coverage (Tier 0) - Re-enable commented-out tests

## Mission

Re-enable solver tests that are currently commented out due to API drift. This will unblock validation of the type solver and ensure correctness of type inference, subtype checking, and evaluation logic.

## Current Baseline

- **Solver Tests Status:** `infer/subtype/evaluate` tests are commented out
- **Reason:** API drift - test APIs no longer match implementation
- **Impact:** Cannot validate solver correctness, blocking confidence in type checking

## Problem Analysis

**Current Behavior:**
- Solver tests in `wasm/src/solver/` are commented out
- Tests reference functions/signatures that have changed
- Without tests, solver changes cannot be validated

**Why This Matters:**
- Solver is core to type checking (inference, subtyping, evaluation)
- Tests ensure refactoring doesn't break correctness
- Unblocks other Tier 0 work (e.g., Application type expansion)

## Key Files

| File | Purpose |
|------|---------|
| `wasm/src/solver/infer.rs` | Type inference - tests commented out |
| `wasm/src/solver/subtype.rs` | Subtype checking - tests commented out |
| `wasm/src/solver/evaluate.rs` | Type evaluation - tests commented out |
| `wasm/src/solver/` | Main solver directory |

## Task Breakdown

### Phase 1: Inventory
- [ ] Find all commented-out solver tests
- [ ] Document what each test validates
- [ ] Identify API changes that caused test failures

### Phase 2: API Mapping
- [ ] Map old test APIs to current implementation
- [ ] Find equivalent functions in current codebase
- [ ] Document API drift patterns

### Phase 3: Test Updates
- [ ] Update `infer.rs` tests to use current APIs
- [ ] Update `subtype.rs` tests to use current APIs
- [ ] Update `evaluate.rs` tests to use current APIs
- [ ] Uncomment and fix each test module

### Phase 4: Validation
- [ ] Run `cargo test` in wasm directory
- [ ] Ensure all solver tests pass
- [ ] Document test coverage achieved

## Success Criteria

| Metric | Target |
|--------|--------|
| Solver tests uncommented | ✅ All 3 modules |
| Tests pass | ✅ 100% |
| Test coverage | Baseline established |

## Progress Log

### 2026-01-15 - New Assignment 🔵
- **Previous:** Completed Async/Await checks (TS2705/TS1359)
- **New:** Solver Test Coverage (Tier 0)
- **Status:** Starting inventory phase

### Previous Completed Tasks
- ✅ Async/Await checks (TS2705/TS1359) - 2026-01-15
- ✅ Strict null checks support - 2026-01-15
- ✅ TS1359 await expression improvement - 2026-01-15

## Workflow

1. **Sync with EM-4:**
   ```bash
   git fetch origin
   git pull origin rust --rebase
   ```

2. **Work on task:**
   - Focus on `wasm/src/solver/` directory
   - Commit frequently: `git commit -m "[wasm] solver: <description>"`
   - Push to worker-13: `git push origin worker-13`

3. **Validation:**
   - Run `cargo test` in wasm directory
   - Ensure all solver tests pass

4. **When complete:**
   - Update this task list
   - Notify EM-4 for merge review
