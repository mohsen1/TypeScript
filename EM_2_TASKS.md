# EM-2 Task Master

## Team: em-team-2
**Workers:** worker-5, worker-6, worker-7, worker-8
**Branch:** em-team-2

## Mission
Own task assignment, maintain worker task lists, merge locally when stable, escalate to Director only when validated.

## Current Phase: Phase 8 - Conformance, Convergence, and Hardening
**Focus:** Stop the "Any" poisoning, fix parser false positives, harden semantic checking.

### Status Overview
| Metric | Current | Target |
|--------|---------|--------|
| Exact Match | 30.1% | 40% |
| Missing Errors | 60.0% | <50% |
| Parser false positives | 701 | <100 |
| TS2304 extra errors | 343 | <50 |

## Squad Assignments

### 🔴 CRITICAL: Binder Squad (2 workers)
**Problem:** TS2304 (Cannot find name) - global scope and lib.d.ts binding failures causing "Any" poisoning
**Assigned:** worker-5, worker-6
**Goal:** Reduce TS2304 extra errors from 343 to <50

**Tasks:**
1. Debug why `console.log`, `Promise`, `Array` fail to resolve
2. Verify `lib_loader.rs` correctly merges `lib.d.ts` symbols into root `SymbolTable`
3. Fix module augmentation resolution (merging `interface Window` across files)
4. Debug `src/thin_binder.rs` `file_locals` population from library context

### 🟠 PRIORITY: Parser Squad (1 worker)
**Problem:** 701 false positive errors (TS1005: 439, TS1109: 262) polluting measurements
**Assigned:** worker-7
**Goal:** Reduce parser false positives from 701 to <100

**Tasks:**
1. Continue TS1005 "expected X" fixes (Worker 1 has patterns 1-5 done, continue remaining)
2. Audit and fix TS1109 "expression expected" false positives (262 occurrences)
3. Improve error recovery/resynchronization in `src/thin_parser.rs`

### 🟡 SUPPORT: Solver Squad (1 worker)
**Problem:** Solver too permissive, defaults to `Any` instead of `Unknown`/`Error`
**Assigned:** worker-8
**Goal:** Expose real errors by stopping silent `Any` fallback

**Tasks:**
1. Change `lower_type` to return `Error` instead of `Any` when resolution fails
2. Harden `solve_subtype` logic
3. Implement "Lawyer" layer for TypeScript quirks (function bivariance, void return exceptions)

## Merge Protocol
1. Workers push to their feature branches
2. EM-2 merges worker branches locally to em-team-2
3. Run validation: `cargo test && npm run conformance`
4. Only escalate to Director when metrics improve and tests pass

## Blocking Issues
None - all workers can start in parallel

## Next Actions
1. Assign tasks to worker-5 through worker-8
2. Monitor progress via WORKER_*_TASK_LIST.md updates
3. Merge completed work to em-team-2
4. Run conformance to measure impact
