# EM-3 TASKS

## Team
- **Engineering Manager:** EM-3
- **Workers:** worker-9, worker-10, worker-11, worker-12
- **Branch:** em-team-3
- **Parent Branch:** rust

## Mission Statement
Keep em-team-3 synchronized with rust, own task assignment for the team, merge worker branches locally after validation, and escalate to the Director only when stable.

## Current Focus: Project Zang Stability & Support

Based on PROJECT_DIRECTION.md analysis, EM-3 team owns the following critical areas:

### Priority 5: Recursion Guards - 2 Crashes
**Owner:** worker-9
- Add recursion depth counter to `solve_subtype` and `check_expression`
- Return TS2589 error ("Type instantiation is excessively deep") at limit (100)
- Prevent stack overflow crashes
- Target: Zero crashes in conformance tests

### Supporting Priority: Parser Edge Cases
**Owner:** worker-10
- Fix remaining ASI (Automatic Semicolon Insertion) edge cases
- Handle complex syntactic constructs that produce TS1005/TS1109
- Support worker-1/worker-5 parser noise reduction efforts
- Target: Reduce parser edge case failures by 50%

### Supporting Priority: Test Infrastructure
**Owner:** worker-11
- Improve test runner lib.d.ts injection robustness
- Add debug logging for symbol resolution failures
- Create conformance test categorization (parser/binder/solver)
- Target: Better visibility into error sources

### Supporting Priority: Validation & Metrics
**Owner:** worker-12
- Build automated error tracking dashboard
- Implement daily regression detection
- Track exact/equivalent/extra/missing metrics over time
- Target: Real-time quality metrics visibility

## Workflow

### Daily Operations
1. **Morning Sync:** Pull rust branch, check for updates
2. **Task Assignment:** Update WORKER_<id>_TASK_LIST.md files
3. **Validation:** Review worker PRs, run conformance tests
4. **Integration:** Merge worker branches to em-team-3 locally
5. **Escalation:** Push to rust only after validation passes

### Quality Gates
Before merging any worker branch:
- [ ] Conformance tests run without crashes
- [ ] Error counts move in the right direction
- [ ] No new TypeScript test failures introduced
- [ ] Code reviewed for correctness

### Success Metrics
- **Stability:** Zero stack overflow crashes
- **Test Infrastructure:** All tests categorizable by component
- **Metrics Dashboard:** Real-time error tracking available
- **Exact Match:** Support increase from 30.1% to 80%+ (team contribution)

## Status Log

### 2026-01-14 - Initial Setup
- Created em-team-3 branch from rust (commit: 17b30883e)
- Assigned focused tasks to workers 9-12
- Ready to receive worker branches for validation

## Notes
- EM-3 is a stability and infrastructure support team
- Focus on preventing crashes and improving developer productivity
- Enable EM-1 and EM-2 teams to move faster with better tooling
- Director may resize or reassign teams at any time
