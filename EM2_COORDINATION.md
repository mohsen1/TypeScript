# EM-2 Team Coordination: Type Checker & Symbol Resolution

## Team Mission
Lead the **Type Checker & Symbol Resolution Team** to achieve 95%+ conformance with TypeScript compiler for **Tier 2 (Type Checker Accuracy)** and **Tier 3 (Symbol Resolution)**.

## Team Structure
- **Manager**: Engineering Manager 2 (EM-2)
- **Workers**: worker-1, worker-2, worker-3, worker-4

## Current Sprint: Type System Core Fixes

### Priority Order
1. **CRITICAL**: Worker 3 - TS2304 Symbol Resolution
2. **HIGH**: Worker 1 - TS2683/TS2571 'this' Handling
3. **MEDIUM**: Worker 2 - TS2322 Type Assignability
4. **MEDIUM**: Worker 4 - TS2507/TS2348 Constructor/Invocation

### Rationale
Worker 3's fix is critical because unresolved symbols default to `Any`, masking all downstream type errors. This must be fixed first to unblock accurate type checking.

## Worker Assignments

### Worker 1: TS2683/TS2571 - 'this' Type Handling (HIGH PRIORITY)
**Branch**: `worker-1-ts2683-this-handling`

**Task**: Fix `this` keyword typing in regular functions to emit TS2683 instead of TS2571

**Key Files**:
- `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/thin_checker.rs` (lines 646-679, 9950-9975, 14091)

**Success Criteria**:
- TS2683 emitted for implicit `this` in regular functions
- TS2571 only for actual `unknown` typed values
- No regressions

**Status**: ASSIGNED
**Task Document**: `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/TASK_WORKER_1_TS2683.md`

### Worker 2: TS2322 Type Assignability (Tier 2)
**Branch**: `worker-2-ts2322-assignability`

**Task**: Make type assignability stricter - reduce false negatives

**Key Files**:
- `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/solver/subtype.rs`
- `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/solver/compat.rs`
- `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/thin_checker.rs`

**Success Criteria**:
- 10-30% reduction in "Missing TS2322" errors
- Solver fails on unclear type relationships
- Document behavioral changes

**Status**: ASSIGNED
**Task Document**: `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/TASK_WORKER_2_TS2322.md`

### Worker 3: TS2304 Symbol Resolution (Tier 3 - CRITICAL)
**Branch**: `worker-3-ts2304-symbol-resolution`

**Task**: Fix global symbol resolution (lib.d.ts loading and merging)

**Key Files**:
- `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/lib_loader.rs`
- `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/thin_checker.rs`
- Binder files (to be identified)

**Success Criteria**:
- Standard lib symbols (console, Promise, Array) resolve correctly
- Reduction in TS2304 "Missing Errors"
- Unblocks accurate type checking (no Any poisoning)

**Status**: ASSIGNED - **START FIRST**
**Task Document**: `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/TASK_WORKER_3_TS2304.md`

### Worker 4: TS2507/TS2348 - Constructor & Invocation (Tier 2)
**Branch**: `worker-4-ts2507-ts2348-invocation`

**Task**: Fix extends clause validation and call expression checking

**Key Files**:
- `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/src/thin_checker.rs` (lines 7440, 14239, 16616+)

**Success Criteria**:
- TS2507 for non-constructor extends
- TS2348 reduction (proper callable detection)
- No regressions

**Status**: ASSIGNED
**Task Document**: `/Users/mohsenazimi/code/orchestrator-config/workspace/repo/TASK_WORKER_4_TS2507_TS2348.md`

## Dependencies

### Worker 3 → Worker 2
Worker 3's TS2304 fix will affect Worker 2's TS2322 testing:
- Once symbols resolve correctly, more type mismatches will be visible
- Worker 2 should coordinate with Worker 3 before final testing

### EM-1 → EM-2
EM-1's parser fixes affect our work:
- Broken ASTs lead to broken type checking
- Application type expansion affects TS2322 work
- Monitor EM-1's progress

## Workflow

### Phase 1: Analysis & Planning (Day 1)
1. ✅ EM-2 analyzes codebase
2. ✅ EM-2 creates task documents for workers
3. ✅ EM-2 assigns priorities
4. Workers read their task documents
5. Workers investigate their assigned areas

### Phase 2: Implementation (Day 1-2)
1. **Worker 3 starts first** (critical path)
2. Workers 1, 2, 4 begin in parallel
3. Workers create branches and commit incrementally
4. Workers report progress/blockers to EM-2

### Phase 3: Code Review (Day 2)
1. Workers submit branches for review
2. EM-2 reviews code changes
3. EM-2 provides feedback
4. Workers address feedback

### Phase 4: Integration (Day 2)
1. EM-2 merges approved branches
2. EM-2 resolves any merge conflicts
3. EM-2 runs full test suite
4. EM-2 reports to director

## Communication Protocol

### Worker → EM-2
- Report progress every 4-6 hours
- Report blockers immediately
- Submit branch for review when ready
- Ask questions if unclear

### EM-2 → Director
- Daily progress update
- Immediate escalation of blockers
- Architecture decision requests
- Final integration report

## Testing Strategy

### Individual Worker Testing
Each worker must:
1. Build WASM after changes: `cd wasm && wasm-pack build --target web --out-dir pkg`
2. Run unit tests: `cd wasm && cargo test`
3. Run relevant conformance tests: `cd wasm/differential-test && bash run-conformance.sh --max=200`

### Integration Testing (EM-2)
After merging all branches:
1. Full conformance test run: `bash run-conformance.sh --max=1000 --workers=8`
2. Compare error metrics before/after
3. Verify success criteria for all tasks
4. Document any regressions

## Metrics to Track

### Before Fixes (Baseline)
- TS2683 missing count: TBD
- TS2571 extra count (implicit this cases): TBD
- TS2322 missing count: TBD
- TS2304 missing count (lib.d.ts symbols): TBD
- TS2507 missing count: TBD
- TS2348 extra count: TBD

### After Fixes (Target)
- TS2683: Properly emitted for implicit this
- TS2571: No false positives for implicit this
- TS2322: 10-30% reduction in missing
- TS2304: 50%+ reduction for lib symbols
- TS2507: Reduction in missing
- TS2348: Reduction in extra

### How to Measure
```bash
# Run conformance tests and capture output
cd /Users/mohsenazimi/code/orchestrator-config/workspace/repo/wasm/differential-test
bash run-conformance.sh --max=1000 --workers=8 > results.txt 2>&1

# Analyze results
grep -c "Missing.*TS2322" results.txt
grep -c "Missing.*TS2304" results.txt
grep -c "Extra.*TS2348" results.txt
# etc.
```

## Risk Management

### Risk: Worker 3 blocked on binder understanding
**Mitigation**: EM-2 provides additional guidance, pair with Worker 3

### Risk: Worker 2 changes too aggressive (breaks tests)
**Mitigation**: Incremental changes, test after each modification

### Risk: Merge conflicts between workers
**Mitigation**: EM-2 manages merge order, resolves conflicts

### Risk: EM-1 parser changes conflict with our work
**Mitigation**: Coordinate with EM-1, may need to rebase

## Success Definition

### Minimum Success
- TS2304 fix working (lib symbols resolve)
- TS2683 fix working (implicit this detection)
- No regressions in existing tests
- Clean code reviews

### Target Success
- All 4 tasks completed
- Measurable reduction in error mismatches
- Full test suite passing
- Clear documentation

### Stretch Success
- >30% reduction in TS2322 missing
- >60% reduction in TS2304 missing
- Additional edge cases discovered and fixed
- Contribution to overall 95% conformance goal

## Timeline

### Day 1
- ✅ EM-2 analysis and planning
- ✅ Task document creation
- Worker 3 starts investigation (critical path)
- Workers 1, 2, 4 start analysis

### Day 2
- All workers implement fixes
- Workers submit branches for review
- EM-2 reviews and provides feedback

### Day 3
- EM-2 merges approved branches
- Integration testing
- Final report to director

## Notes
- This is a foundational sprint - quality over speed
- Type system fixes have cascading effects
- Test thoroughly before submitting
- Document all behavioral changes from TypeScript
- Coordinate frequently to avoid conflicts
