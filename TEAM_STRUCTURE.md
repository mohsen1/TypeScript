# Team Structure - Project Zang Orchestrator

**Last Updated:** 2026-01-15 13:20
**Director:** Claude Code Orchestrator
**Target Branch:** rust

---

## ✅ FINAL TEAM STRUCTURE: THREE EMs ESTABLISHED

Project Zang has **3 Engineering Managers (EMs)** and **8 Workers**.

### Team Size Policy
- **Maximum team size: 4 workers per EM**
- Teams are resized dynamically after every merge
- If team size exceeds 4, spin up new EM and reassign workers

---

## Team Assignments (FINAL - 2026-01-15 13:20)

### EM-1: "Syntax Squad" 🟢 ACTIVE
**Focus:** Tier 1 (Parser Accuracy)

| Worker | Branch | Squad | Assigned Focus | Status | Throughput |
|--------|--------|-------|----------------|--------|------------|
| Worker 3 | worker-3 | Syntax | TS1109 suppression | 🔵 Week 1 mentorship | TBD |
| Worker 4 | worker-4 | Syntax | TS1005 suppression | 🔵 Week 1 mentorship | TBD |
| Worker 5 | worker-5 | Syntax | Parser recovery ✅ Complete | 🟢 Mentor | High |

**EM Branch:** em-team-1

**Leadership:** Worker 5 (exceptional throughput, mentor for Workers 3-4)

**Status:**
- ✅ **Workers 3-4 restarted** on 2026-01-15 13:45 under Worker 5's mentorship
- ✅ Worker 5 complete - leading Syntax Squad
- 📅 **Week 1 Checkpoint:** 2026-01-22
- 📅 **Decision point:** 2026-02-05

**Worker 3 Assessment:**
- Assigned: TS1109 "Expression expected" suppression (reduce from 262 to <40 errors)
- Actual: Documentation only (claimed it was "already implemented")
- Reality: TS1109 suppression is NOT implemented
- Throughput: None - needs reassignment or reset

**Worker 4 Assessment:**
- Assigned: TS1005 "X expected" suppression (extend Worker 5's work, reduce from 345 to <50)
- Actual: Implemented "Invert Solver Defaults" (semantic task, not parser)
- Issue: Did wrong task - this was Worker 7's assignment (solver defaults)
- Throughput: Low - completed wrong task, needs redirection

---

### EM-2: "Semantics Squad" ✅ HEALTHY
**Focus:** Tier 2 (Type Checker) + Tier 3 (Symbol Resolution) + Tier 5 (Async)

| Worker | Branch | Squad | Current Focus | Status | Throughput |
|--------|--------|-------|---------------|--------|------------|
| Worker 6 | worker-6 | Semantics | Module resolution (re-exports, TS2792) | 🔴 Not Working | Medium |
| Worker 7 | worker-7 | Semantics | Module resolution (namespace, defaults) | 🔴 Stalled | Medium |
| Worker 8 | worker-8 | LSP | TypeScript config integration | ✅ Complete | High |

**EM Branch:** em-team-2

**Status:**
- ⚠️ Worker 6 reassigned to help Worker 7 (2026-01-15 13:50) - ZERO commits since
- 🔴 Worker 7 stalled for 2+ weeks on module resolution - NO recent commits
- 🔴 Two-pronged approach NOT working - daily sync not happening
- ✅ Worker 8 complete - LSP TypeScript config integration merged

**Priority Issues:**
- 🔴 CRITICAL: Module resolution (800+ errors) - NO progress from Workers 6-7
- Worker 6: re-exports, TS2792 - NOT STARTED
- Worker 7: namespace imports, default imports - STALLED

---

### EM-3: "Type Checking Squad" 🟢 NEW
**Focus:** Tier 2 (Type Checker Accuracy)

| Worker | Branch | Squad | Focus Area | Status | Throughput |
|--------|--------|-------|------------|--------|------------|
| Worker 1 | worker-1 | Type | TS2683 (implicit this) ✅ Complete | 🟢 Ready | High |
| Worker 2 | worker-2 | Type | super() handling ✅ Complete | 🟢 Ready | High |

**EM Branch:** em-team-3

**Leadership:** Workers 1-2 (both high-throughput, type checking expertise)

**Completed Work:**
- ✅ Worker 1: TS2683 (implicit this in functions) - commit c958fc9cb
- ✅ Worker 2: super() call handling - commit dc7519914

**Status:**
- Both workers ready for new assignments
- See EM_3_TASKS.md for full mission statement and priorities

---

## Restructuring History (2026-01-15)

### Initial State ❌
- **EM-1:** 4 workers (inactive)
- **EM-2:** 8 workers (active, over capacity)

### First Restructure ❌
- **EM-1:** 5 workers (Workers 1, 2, 3, 4, 5)
- **EM-2:** 3 workers (Workers 6, 7, 8)
- **Problem:** EM-1 still over capacity (5 workers)

### Final Restructure ✅
- **EM-1:** 3 workers (Workers 3, 4, 5) - Syntax Squad
- **EM-2:** 3 workers (Workers 6, 7, 8) - Semantics Squad
- **EM-3:** 2 workers (Workers 1, 2) - Type Checking Squad (NEW)

### Rationale for EM-3 Creation
1. **Workers 1-2 completed type checking work** (TS2683, super()) - distinct from parser work
2. **Type checker accuracy** is critical priority (548 TS2322 errors)
3. **Workers 1-2 have high throughput** - can lead new EM
4. **EM-1 needs to focus on syntax** - Workers 3-4 need parser-focused mentorship from Worker 5
5. **Balances workload** - 3 teams, each with 2-3 workers

---

## Priority Order by Team

### EM-1 (Syntax Squad) Priority Queue
1. **Restart Workers 3-4** - Under Worker 5's mentorship
2. **TS1109 suppression** - Reduce from 262 to <40 errors (Worker 3 or 5)
3. **TS1005 suppression** - Reduce from 345 to <50 errors (Worker 4 or 5)
4. **ASI handling** - Edge cases in automatic semicolon insertion

### EM-2 (Semantics Squad) Priority Queue
1. **Module Symbol Resolution** - 800+ errors (Worker 7)
   - TS7005: "Symbol cannot be referenced from module" (489 extra)
   - TS7008: "Module has no exported member" (336 extra)
   - TS2792: `import()` type resolution (161 missing)

2. **LSP Config Integration** (Worker 8) - Approved, ready to start

3. **Type Checker Accuracy** (Worker 6 after current task)
   - Remaining TS2304 issues
   - Other semantic errors

### EM-3 (Type Checking Squad) Priority Queue
1. **TS2322 Type Assignability** - 548 extra errors (Worker 1 or 2)
   - Categorize: legitimate vs false positive
   - Fix assignability logic for false positives
   - Handle union/intersection type edge cases

2. **TS2571 Over-reporting** (Worker 1 or 2)
   - Should be TS2683 in many cases
   - Fix `this` type inference in non-class methods

3. **Generic Type Constraints** (Future)
   - Type parameter enforcement
   - Variance and covariance

---

## Merge Workflow

1. **Workers** commit to their worker branches (worker-1 through worker-8)
2. **EMs** merge from worker branches to EM branches (em-team-1, em-team-2, em-team-3)
3. **Director** merges from EM branches to `rust` branch
4. **After merge:** Director evaluates team sizes and reassigns if needed

---

## Current Blocking Issues

| Issue | Workers Blocked | Resolution Path |
|-------|-----------------|-----------------|
| Workers 3-4 restart | Workers 3-4 | EM-1 must assign tasks under Worker 5's mentorship |
| Module resolution | Workers 6-7 (EM-2) | Worker 7 actively working |
| EM-3 activation | Workers 1-2 | EM-3 must assign TS2322 categorization task |

---

## Documentation

- **Director Planning:** `.orchestrator/` directory
- **EM-1 Tasks:** `EM_1_TASKS.md`
- **EM-2 Tasks:** `EM_2_TASKS.md`
- **EM-3 Tasks:** `EM_3_TASKS.md`
- **EM-4 Tasks:** `EM_4_TASKS.md` ✅ Created (2025-01-15)
- **Project Direction:** `PROJECT_DIRECTION.md` (root)
- **Workflow:** `AGENTS.md` (root)

---

## Success Metrics

| Team | Exact Match Target | Current | Priority |
|------|-------------------|---------|----------|
| **EM-1** | Reduce parser noise by 80% | TS1109: ~262, TS1005: ~345 | 🔴 High |
| **EM-2** | Fix 800+ module errors | TS7005: 489, TS7008: 336 | 🔴 Critical |
| **EM-3** | Reduce type checker noise by 70% | TS2322: ~548 | 🟡 High |
| **EM-4** | Fix Tier 0 cross-cutting gaps | Not measured | 🟡 High |

**Overall Project Goal:** 95%+ exact match rate before production

---

## Capacity Planning

**Current State (Post-Rebalancing):** ✅ BALANCED
- EM-1: 3 workers (at capacity)
- EM-2: 3 workers (at capacity, 🔴 at risk)
- EM-3: 4 workers (at capacity, Workers 1-2, 11-12)
- EM-4: 4 workers (at capacity, Workers 9-10, 13-14)

**Rebalancing Actions (2025-01-15):**
- ✅ Workers 9-10 moved from EM-3 to EM-4 (Tier 0 quality)
- ✅ EM-3 now has 4 workers (within capacity)
- ✅ EM-4 established with 4 workers (Tier 0 focus)

**Future Considerations:**
- EM-2 needs intervention: Workers 6-7 have zero commits on module resolution
- EM-4 needs EM assignment (currently TBD)
- Monitor EM-1 mentorship progress (checkpoint: 2025-01-22)
