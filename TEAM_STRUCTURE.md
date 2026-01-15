# Team Structure - Project Zang Orchestrator

**Last Updated:** 2026-01-15 12:30
**Director:** Claude Code Orchestrator
**Target Branch:** rust

---

## Overview

Project Zang has 2 Engineering Managers (EMs) and 8 Workers. **RESTRUCTURED 2026-01-15** to balance workload.

### Team Size Policy
- **Maximum team size: 4 workers per EM**
- Teams are resized dynamically after every merge
- If team size exceeds 4, spin up new EM and reassign workers

---

## Team Assignments (UPDATED 2026-01-15 12:57)

### EM-1: "Syntax Squad" ⚠️ RESTRUCTURING REQUIRED
**Focus:** Tier 1 (Parser Accuracy) + Tier 3 (Symbol Resolution)

| Worker | Branch | Squad | Assigned Focus | Status | Throughput |
|--------|--------|-------|----------------|--------|------------|
| Worker 1 | worker-1 | Syntax | TS2683 (implicit this) ✅ Complete | 🟢 Ready | High |
| Worker 2 | worker-2 | Syntax | super() ✅ Complete | 🟢 Ready | High |
| Worker 3 | worker-3 | Syntax | TS1109 suppression | ❌ **Not done** | None |
| Worker 4 | worker-4 | Syntax | TS1005 suppression | ❌ **Wrong task** | Low |
| Worker 5 | worker-5 | Syntax | Parser recovery ✅ Complete | 🟢 Complete | High |

**EM Branch:** em-team-1

**⚠️ CRITICAL ISSUE:** Workers 3-4 did not complete their Syntax Squad tasks

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

**Priority Issues:**
- ⚠️ **Reassign Workers 3-4** - Did not complete Syntax Squad tasks
- ✅ Worker 5 complete - transfer to EM-1 (parser work aligns)
- 🔄 Workers 1-2 ready for next assignments

---

### EM-2: "Semantics Squad" ✅ HEALTHY
**Focus:** Tier 2 (Type Checker) + Tier 3 (Symbol Resolution) + Tier 5 (Async)

| Worker | Branch | Squad | Current Focus | Status | Throughput |
|--------|--------|-------|---------------|--------|------------|
| Worker 6 | worker-6 | Binder | Global scope / lib injection | 🟢 Ready | Medium |
| Worker 7 | worker-7 | Semantics | Module symbol resolution (partial) | 🔵 Active | Medium |
| Worker 8 | worker-8 | LSP | TypeScript config integration | 🟢 Approved | TBD |

**EM Branch:** em-team-2

**Priority Issues:**
- Module resolution (TS7005, TS7008, TS2792) - 800+ combined errors (Worker 7 - partial implementation)
- LSP TypeScript config integration (Worker 8 - ready to start)
- Remaining type checker accuracy issues

**Worker 7 Status:**
- ✅ Partial implementation merged (named imports work)
- ⚠️ Incomplete: namespace, default, re-exports, TS2792
- 🔄 Continue task to completion

---

## Restructuring History (2026-01-15)

### Before Restructure ❌
- **EM-1:** 4 workers (inactive)
- **EM-2:** 8 workers (active, over capacity)

### After Restructure ✅
- **EM-1:** 5 workers (Workers 1, 2, 3, 4, 5 transferring)
- **EM-2:** 3 workers (Workers 6, 7, 8)

### Rationale
1. **Workers 1-2 completed syntax work** (TS2683, super()) - belongs under EM-1
2. **Worker 5 doing parser work** - belongs under EM-1's Syntax Squad
3. **EM-2 overloaded** with 8 workers, needs streamlining
4. **EM-1 inactive** with no tasks assigned, needs activation

### Transition Plan
- **Immediate:** Workers 1-2 now managed by EM-1
- **After current task:** Worker 5 transfers to EM-1
- **EM-1 action:** Activate Workers 3-4 with first assignments
- **EM-2 action:** Continue focus on semantics (Workers 6, 7, 8)

---

## Priority Order by Team

### EM-1 (Syntax Squad) Priority Queue
1. **Activate Workers 3-4** - Assign first tasks (TS1109, TS1005, ASI)
2. **Review Workers 1-2 completed work** - Assign next tasks
3. **Complete Worker 5 transfer** - After parser task completes
4. **Parser Accuracy (Tier 1)** - Foundation for all other work
   - TS1109 extra (Parser expression expected errors)
   - TS1005 extra (Parser token expected errors)
   - ASI handling edge cases

### EM-2 (Semantics Squad) Priority Queue
1. **Module Symbol Resolution** - 800+ errors (Worker 7)
   - TS7005: "Symbol cannot be referenced from module" (489 extra)
   - TS7008: "Module has no exported member" (336 extra)
   - TS2792: `import()` type resolution (161 missing)

2. **LSP Config Integration** (Worker 8) - Approved, ready to start

3. **Type Checker Accuracy** (Worker 6 after current task)
   - Remaining TS2304 issues
   - Other semantic errors

---

## Merge Workflow

1. **Workers** commit to their worker branches (worker-1 through worker-8)
2. **EMs** merge from worker branches to EM branches (em-team-1, em-team-2)
3. **Director** merges from EM branches to `rust` branch
4. **After merge:** Director evaluates team sizes and reassigns if needed

---

## Current Blocking Issues

| Issue | Workers Blocked | Resolution Path |
|-------|-----------------|-----------------|
| EM-1 activation | Workers 3-4 | EM-1 must assign first tasks |
| Worker 5 transfer | None | Complete current task, then transfer |
| Module resolution | Workers 6-7 (EM-2) | Worker 7 actively working |

---

## Documentation

- **Director Planning:** `.orchestrator/` directory
- **EM-1 Tasks:** `EM_1_TASKS.md`
- **EM-2 Tasks:** `EM_2_TASKS.md`
- **Project Direction:** `PROJECT_DIRECTION.md` (root)
- **Workflow:** `AGENTS.md` (root)

---

## Success Metrics

| Team | Exact Match Target | Current | Priority |
|------|-------------------|---------|----------|
| **EM-1** | Reduce parser noise by 80% | TS1109: ~262, TS1005: ~439 | 🔴 High |
| **EM-2** | Fix 800+ module errors | TS7005: 489, TS7008: 336 | 🔴 Critical |

**Overall Project Goal:** 95%+ exact match rate before production
