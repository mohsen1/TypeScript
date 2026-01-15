# EM-2 TASKS

## Team
- **Engineering Manager:** EM-2
- **Workers:** worker-5, worker-6, worker-7, worker-8
- **Branch:** em-team-2
- **Parent Branch:** rust

## Mission Statement
Keep em-team-2 synchronized with rust, own task assignment for the team, merge worker branches locally after validation, and escalate to the Director only when stable.

## Current Focus: Project Zang Priority Items

Based on PROJECT_DIRECTION.md analysis, EM-2 team owns the following critical areas:

### Priority 2: Global Scope Fix (TS2304) - 343 Extra Errors
**Owner:** worker-5
- Fix lib.d.ts injection in test runner
- Ensure lib.d.ts is correctly merged into root SymbolTable for every test
- Target: Reduce extra TS2304 from 343 to <10 errors

### Priority 2: Global Merging Across Files
**Owner:** worker-6
- Fix global merging (interfaces like Window, types)
- Ensure interface merging works correctly across multiple files
- Verify symbol table updates propagate correctly
- Target: Zero missing TS2304 errors for known globals

### Priority 3: Solver Strictness (Support EM-1)
**Owner:** worker-7
- Invert solver defaults from ANY to UNKNOWN
- Support EM-1's semantic analysis work
- Update type resolution to return UNKNOWN/ERROR on failure
- Accept temporary regression in extra errors

### Priority 4: Class Property Initialization (Support)
**Owner:** worker-8
- Support EM-1's TS2564 implementation work
- Verify strictPropertyInitialization checks work correctly
- Add additional test coverage for edge cases
- Target: Help reduce TS2564 from 413 to <20 missing errors

## Workflow

### Daily Operations
1. **Morning Sync:** Pull rust branch, check for updates
2. **Task Assignment:** Update WORKER_<id>_TASK_LIST.md files
3. **Validation:** Review worker PRs, run conformance tests
4. **Integration:** Merge worker branches to em-team-2 locally
5. **Escalation:** Push to rust only after validation passes

### Quality Gates
Before merging any worker branch:
- [ ] Conformance tests run without crashes
- [ ] Error counts move in the right direction
- [ ] No new TypeScript test failures introduced
- [ ] Code reviewed for correctness

### Success Metrics
- **Exact Match:** Increase from 30.1% to 80%+
- **TS2304 Extra:** <10 extra errors
- **TS2304 Missing:** <10 missing errors
- **Global Merging:** All known globals resolve correctly

## Status Log

### 2026-01-14 - Initial Setup
- Created em-team-2 branch from rust
- Assigned focused tasks to workers 5-8
- Ready to receive worker branches for validation

## Notes
- EM-2 owns the binder/lib injection critical path
- This is the root cause of "Error Poisoning" - fixes here unlock semantic analysis
- Work closely with EM-3 on test infrastructure improvements
- Director may resize or reassign teams at any time
